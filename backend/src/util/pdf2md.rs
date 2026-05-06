pub struct Pdf2MdInput {
    pub pdf: Vec<u8>,
}

pub struct Pdf2MdOutput {
    pub markdown: String,
}

#[async_trait::async_trait]
pub trait Pdf2MdService: Send + Sync {
    async fn convert(&self, input: Pdf2MdInput) -> Result<Pdf2MdOutput, anyhow::Error>;
}

pub struct Pdf2MdServiceImpl {}

impl Default for Pdf2MdServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdf2MdServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl Pdf2MdService for Pdf2MdServiceImpl {
    async fn convert(&self, input: Pdf2MdInput) -> Result<Pdf2MdOutput, anyhow::Error> {
        let markdown = tokio::task::spawn_blocking(move || -> Result<String, anyhow::Error> {
            let doc = pdf_oxide::PdfDocument::from_bytes(input.pdf)
                .map_err(|e| anyhow::anyhow!("failed to parse pdf: {e}"))?;
            let options = pdf_oxide::converters::ConversionOptions::default();
            doc.to_markdown_all(&options)
                .map_err(|e| anyhow::anyhow!("failed to convert pdf to markdown: {e}"))
        })
        .await??;

        Ok(Pdf2MdOutput { markdown })
    }
}
