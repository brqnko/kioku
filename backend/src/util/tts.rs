pub struct SynthesizedAudio {
    pub content_type: String,
    pub audio: Vec<u8>,
}

#[derive(Clone)]
pub struct SpeakerVoice {
    pub speaker: String,
    pub voice: String,
}

pub struct SynthesizeDialogueInput {
    pub script: String,
    pub speakers: Vec<SpeakerVoice>,
}

#[async_trait::async_trait]
pub trait TTSClient: Send + Sync {
    async fn synthesize_dialogue(
        &self,
        input: SynthesizeDialogueInput,
    ) -> Result<SynthesizedAudio, anyhow::Error>;
}

// Gemini TTS implementation

pub struct GeminiTtsImpl {
    http_client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiTtsImpl {
    pub fn new(api_key: String) -> Result<Self, anyhow::Error> {
        Ok(Self {
            http_client: reqwest::Client::builder().build()?,
            api_key,
            model: "gemini-2.5-flash-preview-tts".to_string(),
        })
    }
}

#[async_trait::async_trait]
impl TTSClient for GeminiTtsImpl {
    async fn synthesize_dialogue(
        &self,
        input: SynthesizeDialogueInput,
    ) -> Result<SynthesizedAudio, anyhow::Error> {
        #[derive(serde::Serialize)]
        struct Request<'a> {
            contents: Vec<Content<'a>>,
            #[serde(rename = "generationConfig")]
            generation_config: GenerationConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct Content<'a> {
            parts: Vec<Part<'a>>,
        }

        #[derive(serde::Serialize)]
        struct Part<'a> {
            text: &'a str,
        }

        #[derive(serde::Serialize)]
        struct GenerationConfig<'a> {
            #[serde(rename = "responseModalities")]
            response_modalities: Vec<&'static str>,
            #[serde(rename = "speechConfig")]
            speech_config: SpeechConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct SpeechConfig<'a> {
            #[serde(rename = "multiSpeakerVoiceConfig")]
            multi_speaker_voice_config: MultiSpeakerVoiceConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct MultiSpeakerVoiceConfig<'a> {
            #[serde(rename = "speakerVoiceConfigs")]
            speaker_voice_configs: Vec<SpeakerVoiceConfig<'a>>,
        }

        #[derive(serde::Serialize)]
        struct SpeakerVoiceConfig<'a> {
            speaker: &'a str,
            #[serde(rename = "voiceConfig")]
            voice_config: VoiceConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct VoiceConfig<'a> {
            #[serde(rename = "prebuiltVoiceConfig")]
            prebuilt_voice_config: PrebuiltVoiceConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct PrebuiltVoiceConfig<'a> {
            #[serde(rename = "voiceName")]
            voice_name: &'a str,
        }

        #[derive(serde::Deserialize)]
        struct Response {
            candidates: Vec<Candidate>,
        }

        #[derive(serde::Deserialize)]
        struct Candidate {
            content: ResponseContent,
        }

        #[derive(serde::Deserialize)]
        struct ResponseContent {
            parts: Vec<ResponsePart>,
        }

        #[derive(serde::Deserialize)]
        struct ResponsePart {
            #[serde(rename = "inlineData")]
            inline_data: Option<InlineData>,
        }

        #[derive(serde::Deserialize)]
        struct InlineData {
            #[serde(rename = "mimeType")]
            mime_type: String,
            data: String,
        }

        let speaker_voice_configs = input
            .speakers
            .iter()
            .map(|s| SpeakerVoiceConfig {
                speaker: &s.speaker,
                voice_config: VoiceConfig {
                    prebuilt_voice_config: PrebuiltVoiceConfig {
                        voice_name: &s.voice,
                    },
                },
            })
            .collect::<Vec<_>>();

        let req = Request {
            contents: vec![Content {
                parts: vec![Part {
                    text: &input.script,
                }],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["AUDIO"],
                speech_config: SpeechConfig {
                    multi_speaker_voice_config: MultiSpeakerVoiceConfig {
                        speaker_voice_configs,
                    },
                },
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        let resp = self
            .http_client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<Response>()
            .await?;

        let inline = resp
            .candidates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no candidates in gemini tts response"))?
            .content
            .parts
            .into_iter()
            .find_map(|p| p.inline_data)
            .ok_or_else(|| anyhow::anyhow!("no inline_data in gemini tts response"))?;

        use base64::Engine as _;
        let audio = base64::engine::general_purpose::STANDARD.decode(inline.data.as_bytes())?;

        Ok(SynthesizedAudio {
            content_type: inline.mime_type,
            audio,
        })
    }
}

/// Parse the sample rate from a `audio/L16;codec=pcm;rate=24000` style mime string.
pub fn parse_pcm_sample_rate(content_type: &str) -> u32 {
    content_type
        .split(';')
        .find_map(|p| p.trim().strip_prefix("rate="))
        .and_then(|r| r.parse::<u32>().ok())
        .unwrap_or(24000)
}

/// Wrap raw signed 16-bit little-endian mono PCM in a WAV (RIFF) container so
/// the result is directly playable in browsers and OS audio players.
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
