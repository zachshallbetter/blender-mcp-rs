use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Timeout waiting for Blender response")]
    Timeout,

    #[error("Protocol error: {0}")]
    Protocol(String),
}

/// Control transport client communicating with the Blender addon instance.
pub struct BlenderIpcClient {
    host: String,
    port: u16,
    timeout: Duration,
}

impl BlenderIpcClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn send_command(&self, command: &serde_json::Value) -> Result<serde_json::Value, IpcError> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = tokio::time::timeout(self.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| IpcError::Timeout)??;

        let payload = serde_json::to_vec(command)?;
        stream.write_all(&payload).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 8192];

        loop {
            let bytes_read = tokio::time::timeout(self.timeout, stream.read(&mut chunk))
                .await
                .map_err(|_| IpcError::Timeout)??;

            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);

            // Attempt to parse complete JSON
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&buffer) {
                return Ok(val);
            }
        }

        if buffer.is_empty() {
            return Err(IpcError::Protocol("Empty response from Blender".to_string()));
        }

        let resp: serde_json::Value = serde_json::from_slice(&buffer)?;
        Ok(resp)
    }
}

/// Shared memory ring buffer reader for zero-copy viewport framebuffer streaming.
pub struct SharedMemoryViewport {
    path: PathBuf,
}

impl SharedMemoryViewport {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
