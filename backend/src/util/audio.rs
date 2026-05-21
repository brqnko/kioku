use anyhow::Context as _;
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;

pub async fn wav_to_opus(wav: Vec<u8>, bitrate_kbps: u32) -> anyhow::Result<Vec<u8>> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel", "error",
            "-f", "wav",
            "-i", "pipe:0",
            "-c:a", "libopus",
            "-b:a", &format!("{bitrate_kbps}k"),
            "-application", "voip",
            "-f", "ogg",
            "pipe:1",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn ffmpeg")?;

    let mut stdin = child.stdin.take().context("ffmpeg stdin missing")?;
    let writer = tokio::spawn(async move {
        stdin.write_all(&wav).await?;
        stdin.shutdown().await?;
        Ok::<(), std::io::Error>(())
    });

    let output = child.wait_with_output().await.context("ffmpeg wait failed")?;
    writer.await.context("ffmpeg stdin task panicked")??;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg exited with {}: {stderr}", output.status);
    }
    Ok(output.stdout)
}
