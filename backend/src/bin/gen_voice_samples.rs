use backend::util::tts::{SupertonicTtsImpl, SynthesizeDialogueInput, TTSClient, wrap_pcm_as_wav};

const VOICES: &[&str] = &["F1", "F2", "F3", "F4", "F5", "M1", "M2", "M3", "M4", "M5"];

const SAMPLES: &[(&str, &str)] = &[
    ("ja", "こんにちは。これはサンプル音声です。お好みの声を選んでください。"),
    ("en", "Hello. This is a sample audio. Please select your preferred voice."),
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let out_dir = std::env::args().nth(1).unwrap_or_else(|| "/workspace/frontend/public/voice-samples".to_string());
    std::fs::create_dir_all(&out_dir)?;

    let tts = SupertonicTtsImpl::new()?;

    for (lang, text) in SAMPLES {
        for voice in VOICES {
            let filename = if *lang == "ja" {
                format!("{out_dir}/{voice}.wav")
            } else {
                format!("{out_dir}/{voice}_{lang}.wav")
            };

            if std::path::Path::new(&filename).exists() && *lang == "ja" {
                tracing::info!("skip (exists): {filename}");
                continue;
            }

            tracing::info!("generating {filename}");
            let result = tts.synthesize_dialogue(SynthesizeDialogueInput {
                script: text.to_string(),
                voice: format!("{lang}:{voice}"),
            }).await?;

            let wav = wrap_pcm_as_wav(&result.audio, result.sample_rate);
            std::fs::write(&filename, wav)?;
            tracing::info!("wrote {filename}");
        }
    }

    Ok(())
}
