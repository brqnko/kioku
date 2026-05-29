
pub enum MdConvertInput {
    Pdf(Vec<u8>),
    Url(String),
}

pub struct MdConvertOutput {
    pub markdown: String,
}

#[async_trait::async_trait]
pub trait MdConvertService: Send + Sync {
    async fn convert(&self, input: MdConvertInput) -> Result<MdConvertOutput, anyhow::Error>;
}

pub struct MdConvertServiceImpl {
    client: reqwest::Client,
}

impl Default for MdConvertServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MdConvertServiceImpl {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl MdConvertService for MdConvertServiceImpl {
    async fn convert(&self, input: MdConvertInput) -> Result<MdConvertOutput, anyhow::Error> {
        let markdown = match input {
            MdConvertInput::Pdf(pdf) => {
                tokio::task::spawn_blocking(move || -> Result<String, anyhow::Error> {
                    let doc = pdf_oxide::PdfDocument::from_bytes(pdf)
                        .map_err(|e| anyhow::anyhow!("failed to parse pdf: {e}"))?;
                    let options = pdf_oxide::converters::ConversionOptions::default();
                    doc.to_markdown_all(&options)
                        .map_err(|e| anyhow::anyhow!("failed to convert pdf to markdown: {e}"))
                })
                .await??
            }
            MdConvertInput::Url(url) => {
                let response = self
                    .client
                    .get(&url)
                    .send()
                    .await?
                    .error_for_status()?;
                let html = response.text().await?;
                html2md_rs::to_md::safe_from_html_to_md(html)
                    .map_err(|e| anyhow::anyhow!("failed to convert html to markdown: {e:?}"))?
            }
        };

        Ok(MdConvertOutput { markdown })
    }
}
