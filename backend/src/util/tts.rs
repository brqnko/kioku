pub struct SynthesizedAudio {
    pub content_type: String,
    pub audio: Vec<u8>,
}

pub struct SynthesizeDialogueInput {
    pub script: String,
    pub voice: String,
    pub audio_profile: String,
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
    primary_model: String,
    fallback_model: String,
}

impl GeminiTtsImpl {
    pub fn new(api_key: String) -> Result<Self, anyhow::Error> {
        Ok(Self {
            http_client: reqwest::Client::builder().build()?,
            api_key,
            primary_model: "gemini-3.1-flash-tts-preview".to_string(),
            fallback_model: "gemini-2.5-flash-preview-tts".to_string(),
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
            role: &'static str,
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
            temperature: f32,
            speech_config: SpeechConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct SpeechConfig<'a> {
            voice_config: VoiceConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct VoiceConfig<'a> {
            prebuilt_voice_config: PrebuiltVoiceConfig<'a>,
        }

        #[derive(serde::Serialize)]
        struct PrebuiltVoiceConfig<'a> {
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

        let prompt = format!(
            "Read the following transcript based on the audio profile and director's note.\n\
            \n\
            # Audio Profile\n\
            {audio_profile}\n\
            \n\
            # Director's note\n\
            Style: Conversational podcast. Pace: Varied. Accent: Natural.\n\
            \n\
            ## Scene:\n\
            Two podcast hosts in a cozy recording studio, speaking directly into microphones.\n\
            \n\
            ## Sample Context:\n\
            Two co-hosts having a genuine, engaging conversation. Natural back-and-forth with \
            moments of discovery, laughter, and intellectual curiosity. Tone is warm, accessible, \
            and authentic. Hosts feed off each other's energy naturally.\n\
            \n\
            ## Transcript:\n\
            {script}",
            audio_profile = input.audio_profile,
            script = input.script,
        );

        let req = Request {
            contents: vec![Content {
                role: "user",
                parts: vec![Part { text: &prompt }],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["audio"],
                temperature: 1.0,
                speech_config: SpeechConfig {
                    voice_config: VoiceConfig {
                        prebuilt_voice_config: PrebuiltVoiceConfig {
                            voice_name: &input.voice,
                        },
                    },
                },
            },
        };

        use base64::Engine as _;
        let models = [self.primary_model.as_str(), self.fallback_model.as_str()];
        let mut last_err: Option<anyhow::Error> = None;
        for model in models {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent",
            );
            let result = self
                .http_client
                .post(&url)
                .header("x-goog-api-key", &self.api_key)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .json(&req)
                .send()
                .await
                .and_then(|r| r.error_for_status());
            let resp_bytes = match result {
                Ok(r) => r.bytes().await?,
                Err(e) => {
                    tracing::warn!(model, error = %e, "tts model failed, trying fallback");
                    last_err = Some(e.into());
                    continue;
                }
            };
            let responses = match serde_json::from_slice::<Vec<Response>>(&resp_bytes) {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(model, error = %e, "tts model response parse failed, trying fallback");
                    last_err = Some(e.into());
                    continue;
                }
            };
            let mut mime_type = String::new();
            let mut audio = Vec::<u8>::new();
            for response in responses {
                for part in response
                    .candidates
                    .into_iter()
                    .flat_map(|c| c.content.parts)
                {
                    if let Some(inline) = part.inline_data {
                        if mime_type.is_empty() {
                            mime_type = inline.mime_type;
                        }
                        let chunk = base64::engine::general_purpose::STANDARD
                            .decode(inline.data.as_bytes())?;
                        audio.extend(chunk);
                    }
                }
            }
            if mime_type.is_empty() {
                let e = anyhow::anyhow!("no audio data in tts response from {model}");
                tracing::warn!(model, "{e}");
                last_err = Some(e);
                continue;
            }
            return Ok(SynthesizedAudio {
                content_type: mime_type,
                audio,
            });
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("all tts models failed")))
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
