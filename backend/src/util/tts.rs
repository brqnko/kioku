use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context as _;
use ndarray::{Array1, Array2, Array3};
use ort::session::Session;
use ort::value::Value;
use serde::Deserialize;
use unicode_normalization::UnicodeNormalization as _;

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

pub struct SynthesizedAudio {
    pub content_type: String,
    pub audio: Vec<u8>,
    pub sample_rate: u32,
}

pub struct SynthesizeDialogueInput {
    pub script: String,
    /// Format: "lang:voice_style"  e.g. "ja:F1", "en:M1"
    pub voice: String,
}

#[async_trait::async_trait]
pub trait TTSClient: Send + Sync {
    async fn synthesize_dialogue(
        &self,
        input: SynthesizeDialogueInput,
    ) -> Result<SynthesizedAudio, anyhow::Error>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Supertonic TTS (ONNX Runtime via ort, models auto-downloaded via hf-hub)
// ─────────────────────────────────────────────────────────────────────────────

pub struct SupertonicTtsImpl {
    dp: Arc<std::sync::Mutex<Session>>,
    te: Arc<std::sync::Mutex<Session>>,
    ve: Arc<std::sync::Mutex<Session>>,
    vc: Arc<std::sync::Mutex<Session>>,
    config: Arc<TtsConfig>,
    proc: Arc<UnicodeProcessor>,
    voices: Arc<HashMap<String, Style>>,
    denoising_steps: usize,
    speed: f32,
}

impl SupertonicTtsImpl {
    pub fn new() -> anyhow::Result<Self> {
        tracing::info!("supertonic: resolving models from HuggingFace (Supertone/supertonic-3)…");
        let repo = hf_hub::api::sync::ApiBuilder::new()
            .with_cache_dir(PathBuf::from("/opt/huggingface"))
            .build()?
            .model("Supertone/supertonic-3".to_string());

        let dp_path = repo.get("onnx/duration_predictor.onnx")?;
        let te_path = repo.get("onnx/text_encoder.onnx")?;
        let ve_path = repo.get("onnx/vector_estimator.onnx")?;
        let vc_path = repo.get("onnx/vocoder.onnx")?;
        let cfg_path = repo.get("onnx/tts.json")?;
        let idx_path = repo.get("onnx/unicode_indexer.json")?;

        let load = |p: &PathBuf| -> anyhow::Result<Arc<std::sync::Mutex<Session>>> {
            fn build(p: &PathBuf) -> Result<Session, String> {
                let b = Session::builder().map_err(|e| e.to_string())?;
                let b = b.with_memory_pattern(false).map_err(|e| e.to_string())?;
                let b = b.with_intra_threads(2).map_err(|e| e.to_string())?;
                let mut b = b.with_inter_threads(1).map_err(|e| e.to_string())?;
                b.commit_from_file(p).map_err(|e| e.to_string())
            }
            let session = build(p).map_err(|e| anyhow::anyhow!("ort session build failed: {e}"))?;
            Ok(Arc::new(std::sync::Mutex::new(session)))
        };

        let dp = load(&dp_path)?;
        let te = load(&te_path)?;
        let ve = load(&ve_path)?;
        let vc = load(&vc_path)?;

        let config: TtsConfig =
            serde_json::from_str(&std::fs::read_to_string(&cfg_path)?)?;
        let proc = UnicodeProcessor::from_path(&idx_path)?;

        const VOICE_NAMES: &[&str] =
            &["F1", "F2", "F3", "F4", "F5", "M1", "M2", "M3", "M4", "M5"];
        let mut voices = HashMap::new();
        for name in VOICE_NAMES {
            let path = repo.get(&format!("voice_styles/{name}.json"))?;
            let style = load_voice_style(&path)?;
            voices.insert(name.to_string(), style);
        }

        tracing::info!("supertonic: ready");
        Ok(Self {
            dp,
            te,
            ve,
            vc,
            config: Arc::new(config),
            proc: Arc::new(proc),
            voices: Arc::new(voices),
            denoising_steps: 8,
            speed: 1.0,
        })
    }
}

#[async_trait::async_trait]
impl TTSClient for SupertonicTtsImpl {
    async fn synthesize_dialogue(
        &self,
        input: SynthesizeDialogueInput,
    ) -> Result<SynthesizedAudio, anyhow::Error> {
        let (lang, style_name) = input
            .voice
            .split_once(':')
            .context("voice must be 'lang:style' e.g. 'ja:F1'")?;

        let style = self
            .voices
            .get(style_name)
            .ok_or_else(|| anyhow::anyhow!("unknown voice style: {style_name}"))?
            .clone();

        let dp = Arc::clone(&self.dp);
        let te = Arc::clone(&self.te);
        let ve = Arc::clone(&self.ve);
        let vc = Arc::clone(&self.vc);
        let config = Arc::clone(&self.config);
        let proc = Arc::clone(&self.proc);
        let script = input.script;
        let lang = lang.to_string();
        let steps = self.denoising_steps;
        let speed = self.speed;

        tokio::task::spawn_blocking(move || {
            let mut dp = dp.lock().unwrap();
            let mut te = te.lock().unwrap();
            let mut ve = ve.lock().unwrap();
            let mut vc = vc.lock().unwrap();
            synthesize_blocking(&mut dp, &mut te, &mut ve, &mut vc, &config, &proc, &style, &script, &lang, steps, speed)
        })
        .await
        .context("TTS task panicked")?
    }
}

#[allow(clippy::too_many_arguments)]
fn synthesize_blocking(
    dp: &mut Session,
    te: &mut Session,
    ve: &mut Session,
    vc: &mut Session,
    cfg: &TtsConfig,
    proc: &UnicodeProcessor,
    voice: &Style,
    script: &str,
    lang: &str,
    steps: usize,
    speed: f32,
) -> anyhow::Result<SynthesizedAudio> {
    let max_chars = if lang == "ja" || lang == "ko" { 120 } else { 300 };
    let chunks = chunk_text(script, Some(max_chars));
    let silence_samps = (0.3 * cfg.ae.sample_rate as f32) as usize;
    let mut all_samples: Vec<f32> = Vec::new();

    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        if chunk.trim().is_empty() {
            continue;
        }
        let (text_ids_vec, text_mask) = proc.call(chunk, lang)?;
        let seq_len = text_ids_vec[0].len();

        let text_ids = Array2::<i64>::from_shape_vec(
            (1, seq_len),
            text_ids_vec.into_iter().flatten().collect(),
        )?;

        // ── Duration predictor ───────────────────────────────────────────────
        let dp_out = dp.run(ort::inputs![
            "text_ids" => Value::from_array(text_ids.clone())?,
            "style_dp" => Value::from_array(voice.dp.clone())?,
            "text_mask" => Value::from_array(text_mask.clone())?,
        ])?;
        let (_, dur_data) = dp_out["duration"].try_extract_tensor::<f32>()?;
        let mut duration: Vec<f32> = dur_data.to_vec();
        for d in &mut duration {
            *d /= speed;
        }

        // ── Text encoder ─────────────────────────────────────────────────────
        let te_out = te.run(ort::inputs![
            "text_ids" => Value::from_array(text_ids)?,
            "style_ttl" => Value::from_array(voice.ttl.clone())?,
            "text_mask" => Value::from_array(text_mask.clone())?,
        ])?;
        let (emb_shape, emb_data) = te_out["text_emb"].try_extract_tensor::<f32>()?;
        let text_emb = Array3::<f32>::from_shape_vec(
            (emb_shape[0] as usize, emb_shape[1] as usize, emb_shape[2] as usize),
            emb_data.to_vec(),
        )?;

        // ── Sample noisy latent ──────────────────────────────────────────────
        let (mut xt, latent_mask) = sample_noisy_latent(
            &duration,
            cfg.ae.sample_rate,
            cfg.ae.base_chunk_size,
            cfg.ttl.chunk_compress_factor,
            cfg.ttl.latent_dim,
        );

        // ── Denoising loop ───────────────────────────────────────────────────
        let total_arr = Array1::<f32>::from_elem(1, steps as f32);
        for step in 0..steps {
            let cur_arr = Array1::<f32>::from_elem(1, step as f32);
            let ve_out = ve.run(ort::inputs![
                "noisy_latent" => Value::from_array(xt.clone())?,
                "text_emb"     => Value::from_array(text_emb.clone())?,
                "style_ttl"    => Value::from_array(voice.ttl.clone())?,
                "latent_mask"  => Value::from_array(latent_mask.clone())?,
                "text_mask"    => Value::from_array(text_mask.clone())?,
                "current_step" => Value::from_array(cur_arr)?,
                "total_step"   => Value::from_array(total_arr.clone())?,
            ])?;
            let (ds, denoised) = ve_out["denoised_latent"].try_extract_tensor::<f32>()?;
            xt = Array3::<f32>::from_shape_vec(
                (ds[0] as usize, ds[1] as usize, ds[2] as usize),
                denoised.to_vec(),
            )?;
        }

        // ── Vocoder ──────────────────────────────────────────────────────────
        let vc_out = vc.run(ort::inputs!["latent" => Value::from_array(xt)?])?;
        let (_, wav_data) = vc_out["wav_tts"].try_extract_tensor::<f32>()?;
        let wav: Vec<f32> = wav_data.to_vec();

        let wav_len = (duration[0] * cfg.ae.sample_rate as f32) as usize;
        let wav_chunk = &wav[..wav_len.min(wav.len())];

        if chunk_idx > 0 {
            all_samples.extend(std::iter::repeat_n(0.0f32, silence_samps));
        }
        all_samples.extend_from_slice(wav_chunk);
    }

    let pcm: Vec<u8> = all_samples
        .iter()
        .flat_map(|&s| ((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes())
        .collect();

    Ok(SynthesizedAudio {
        content_type: "audio/raw-pcm".to_string(),
        audio: pcm,
        sample_rate: cfg.ae.sample_rate as u32,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Config & voice style
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct TtsConfig {
    ae: AEConfig,
    ttl: TTLConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct AEConfig {
    sample_rate: i32,
    base_chunk_size: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct TTLConfig {
    chunk_compress_factor: i32,
    latent_dim: i32,
}

#[derive(Clone)]
struct Style {
    ttl: Array3<f32>,
    dp: Array3<f32>,
}

#[derive(Deserialize)]
struct VoiceStyleData {
    style_ttl: StyleComponent,
    style_dp: StyleComponent,
}

#[derive(Deserialize)]
struct StyleComponent {
    data: Vec<Vec<Vec<f32>>>,
    dims: Vec<usize>,
}

fn load_voice_style(path: &PathBuf) -> anyhow::Result<Style> {
    let raw: VoiceStyleData = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let to_array = |c: StyleComponent| -> anyhow::Result<Array3<f32>> {
        let flat: Vec<f32> = c.data.into_iter().flatten().flatten().collect();
        Ok(Array3::from_shape_vec((c.dims[0], c.dims[1], c.dims[2]), flat)?)
    };
    Ok(Style {
        ttl: to_array(raw.style_ttl)?,
        dp: to_array(raw.style_dp)?,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Text processing
// ─────────────────────────────────────────────────────────────────────────────

struct UnicodeProcessor {
    indexer: Vec<i64>,
}

impl UnicodeProcessor {
    fn from_path(path: &PathBuf) -> anyhow::Result<Self> {
        let indexer: Vec<i64> = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        Ok(Self { indexer })
    }

    fn call(
        &self,
        text: &str,
        lang: &str,
    ) -> anyhow::Result<(Vec<Vec<i64>>, Array3<f32>)> {
        let processed = preprocess_text(text, lang)?;
        let len = processed.chars().count();
        let mut row = vec![0i64; len];
        for (j, c) in processed.chars().enumerate() {
            let v = c as usize;
            row[j] = if v < self.indexer.len() { self.indexer[v] } else { -1 };
        }
        let mask = length_to_mask(&[len], len);
        Ok((vec![row], mask))
    }
}

const AVAILABLE_LANGS: &[&str] = &[
    "en", "ko", "ja", "ar", "bg", "cs", "da", "de", "el", "es", "et", "fi", "fr",
    "hi", "hr", "hu", "id", "it", "lt", "lv", "nl", "pl", "pt", "ro", "ru", "sk",
    "sl", "sv", "tr", "uk", "vi", "na",
];

fn preprocess_text(text: &str, lang: &str) -> anyhow::Result<String> {
    use regex::Regex;
    use std::sync::LazyLock;

    anyhow::ensure!(AVAILABLE_LANGS.contains(&lang), "unsupported lang: {lang}");

    let mut s: String = text.nfkd().collect();

    static EMOJI: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
            r"[\x{1F600}-\x{1F64F}\x{1F300}-\x{1F5FF}\x{1F680}-\x{1F6FF}",
            r"\x{2600}-\x{26FF}\x{2700}-\x{27BF}\x{1F1E6}-\x{1F1FF}]+"
        ))
        .unwrap()
    });
    s = EMOJI.replace_all(&s, "").into_owned();

    for (from, to) in [
        ("–", "-"), ("‑", "-"), ("—", "-"), ("_", " "),
        ("\u{201C}", "\""), ("\u{201D}", "\""),
        ("\u{2018}", "'"), ("\u{2019}", "'"),
        ("´", "'"), ("`", "'"),
        ("[", " "), ("]", " "), ("|", " "), ("/", " "),
        ("#", " "), ("→", " "), ("←", " "),
        ("♥", ""), ("☆", ""), ("♡", ""), ("©", ""), ("\\", ""),
        ("@", " at "), ("e.g.,", "for example, "), ("i.e.,", "that is, "),
    ] {
        s = s.replace(from, to);
    }

    static FIX_SPACE: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        vec![
            (Regex::new(r" ,").unwrap(), ","),
            (Regex::new(r" \.").unwrap(), "."),
            (Regex::new(r" !").unwrap(), "!"),
            (Regex::new(r" \?").unwrap(), "?"),
            (Regex::new(r" ;").unwrap(), ";"),
            (Regex::new(r" :").unwrap(), ":"),
            (Regex::new(r" '").unwrap(), "'"),
        ]
    });
    for (re, rep) in FIX_SPACE.iter() {
        s = re.replace_all(&s, *rep).into_owned();
    }

    for (dbl, single) in [("\"\"", "\""), ("''", "'")] {
        while s.contains(dbl) {
            s = s.replace(dbl, single);
        }
    }

    static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
    s = SPACES.replace_all(&s, " ").into_owned();
    s = s.trim().to_string();

    static ENDS_PUNCT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"[.!?;:,'")\]}…。」』】〉》›»]$"#).unwrap()
    });
    if !s.is_empty() && !ENDS_PUNCT.is_match(&s) {
        s.push('.');
    }

    Ok(format!("<{lang}>{s}</{lang}>"))
}

fn chunk_text(text: &str, max_len: Option<usize>) -> Vec<String> {
    let max_len = max_len.unwrap_or(300);
    let text = text.trim();
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut chunks: Vec<String> = Vec::new();
    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        let mut current = String::new();
        let mut count = 0usize;
        for c in para.chars() {
            current.push(c);
            count += 1;
            if count >= max_len {
                chunks.push(std::mem::take(&mut current));
                count = 0;
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
    }
    if chunks.is_empty() {
        vec![String::new()]
    } else {
        chunks
    }
}

fn length_to_mask(lengths: &[usize], max_len: usize) -> Array3<f32> {
    let bsz = lengths.len();
    let mut mask = Array3::<f32>::zeros((bsz, 1, max_len));
    for (i, &len) in lengths.iter().enumerate() {
        for j in 0..len.min(max_len) {
            mask[[i, 0, j]] = 1.0;
        }
    }
    mask
}

fn sample_noisy_latent(
    duration: &[f32],
    sample_rate: i32,
    base_chunk_size: i32,
    chunk_compress: i32,
    latent_dim: i32,
) -> (Array3<f32>, Array3<f32>) {
    let bsz = duration.len();
    let max_dur = duration.iter().copied().fold(0.0f32, f32::max);
    let wav_len_max = (max_dur * sample_rate as f32) as usize;
    let wav_lengths: Vec<usize> = duration
        .iter()
        .map(|&d| (d * sample_rate as f32) as usize)
        .collect();

    let chunk_size = (base_chunk_size * chunk_compress) as usize;
    let latent_len = wav_len_max.div_ceil(chunk_size).max(1);
    let latent_dim_val = (latent_dim * chunk_compress) as usize;

    let mut noisy = Array3::<f32>::zeros((bsz, latent_dim_val, latent_len));
    let mut rng = rand::rng();
    for b in 0..bsz {
        for d in 0..latent_dim_val {
            for t in 0..latent_len {
                noisy[[b, d, t]] = normal_sample(&mut rng);
            }
        }
    }

    let latent_lengths: Vec<usize> = wav_lengths
        .iter()
        .map(|&l| l.div_ceil(chunk_size).max(1))
        .collect();
    let latent_mask = length_to_mask(&latent_lengths, latent_len);
    for b in 0..bsz {
        for d in 0..latent_dim_val {
            for t in 0..latent_len {
                noisy[[b, d, t]] *= latent_mask[[b, 0, t]];
            }
        }
    }
    (noisy, latent_mask)
}

fn normal_sample<R: rand::RngExt>(rng: &mut R) -> f32 {
    let u1 = rng.random::<f32>().max(f32::EPSILON);
    let u2 = rng.random::<f32>();
    (-2.0f32 * u1.ln()).sqrt() * (2.0f32 * std::f32::consts::PI * u2).cos()
}

// ─────────────────────────────────────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────────────────────────────────────

pub fn wrap_pcm_as_wav(pcm: &[u8], sample_rate: u32) -> Vec<u8> {
    const BITS_PER_SAMPLE: u16 = 16;
    const CHANNELS: u16 = 1;
    let byte_rate = sample_rate * u32::from(CHANNELS) * u32::from(BITS_PER_SAMPLE) / 8;
    let block_align = CHANNELS * BITS_PER_SAMPLE / 8;
    let data_size = pcm.len() as u32;
    let chunk_size = 36 + data_size;
    let mut wav = Vec::<u8>::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&CHANNELS.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

