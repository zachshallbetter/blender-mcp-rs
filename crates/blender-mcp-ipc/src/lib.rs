use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

    #[error(
        "Shared memory buffer overflow: capacity {capacity} bytes, requested {requested} bytes"
    )]
    BufferOverflow { capacity: usize, requested: usize },

    #[error("Invalid frame header magic (expected 0x4652414D, got 0x{0:08X})")]
    InvalidMagic(u32),
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

    pub async fn send_command(
        &self,
        command: &serde_json::Value,
    ) -> Result<serde_json::Value, IpcError> {
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
            return Err(IpcError::Protocol(
                "Empty response from Blender".to_string(),
            ));
        }

        let resp: serde_json::Value = serde_json::from_slice(&buffer)?;
        Ok(resp)
    }
}

pub const SHM_MAGIC: u32 = 0x4652414D; // 'FRAM' in ASCII

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
    pub magic: u32,
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub timestamp_ns: u64,
    pub data_size: u64,
}

pub const HEADER_SIZE: usize = std::mem::size_of::<FrameHeader>();

#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub sequence: u64,
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub timestamp_ns: u64,
    pub pixels: Vec<u8>,
}

/// Shared memory ring buffer for zero-copy viewport framebuffer streaming.
pub struct SharedMemoryViewport {
    path: PathBuf,
    mmap: MmapMut,
    sequence: u64,
}

impl SharedMemoryViewport {
    pub fn create_or_open(path: impl AsRef<Path>, capacity: usize) -> Result<Self, IpcError> {
        let path_buf = path.as_ref().to_path_buf();
        if let Some(parent) = path_buf.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path_buf)?;

        let current_len = file.metadata()?.len() as usize;
        if current_len < capacity {
            file.set_len(capacity as u64)?;
        }

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            path: path_buf,
            mmap,
            sequence: 0,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write_frame(
        &mut self,
        width: u32,
        height: u32,
        channels: u32,
        data: &[u8],
    ) -> Result<(), IpcError> {
        let total_required = HEADER_SIZE + data.len();
        if total_required > self.mmap.len() {
            return Err(IpcError::BufferOverflow {
                capacity: self.mmap.len(),
                requested: total_required,
            });
        }

        self.sequence += 1;
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let header = FrameHeader {
            magic: SHM_MAGIC,
            sequence: self.sequence,
            width,
            height,
            channels,
            timestamp_ns,
            data_size: data.len() as u64,
        };

        unsafe {
            let header_ptr = self.mmap.as_mut_ptr() as *mut FrameHeader;
            std::ptr::write_unaligned(header_ptr, header);
        }

        self.mmap[HEADER_SIZE..HEADER_SIZE + data.len()].copy_from_slice(data);
        self.mmap.flush()?;

        Ok(())
    }

    pub fn read_latest_frame(&self) -> Result<Option<FrameSnapshot>, IpcError> {
        if self.mmap.len() < HEADER_SIZE {
            return Ok(None);
        }

        let header = unsafe {
            let header_ptr = self.mmap.as_ptr() as *const FrameHeader;
            std::ptr::read_unaligned(header_ptr)
        };

        if header.magic == 0 {
            return Ok(None);
        }

        if header.magic != SHM_MAGIC {
            return Err(IpcError::InvalidMagic(header.magic));
        }

        let data_size = header.data_size as usize;
        let end = HEADER_SIZE + data_size;
        if end > self.mmap.len() {
            return Err(IpcError::BufferOverflow {
                capacity: self.mmap.len(),
                requested: end,
            });
        }

        let pixels = self.mmap[HEADER_SIZE..end].to_vec();

        Ok(Some(FrameSnapshot {
            sequence: header.sequence,
            width: header.width,
            height: header.height,
            channels: header.channels,
            timestamp_ns: header.timestamp_ns,
            pixels,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_memory_viewport_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let shm_path = temp_dir.join("test_viewport.shm");

        let mut viewport = SharedMemoryViewport::create_or_open(&shm_path, 1024 * 1024)
            .expect("Failed to create shm viewport");

        let fake_pixels = vec![128u8; 100 * 100 * 4];
        viewport
            .write_frame(100, 100, 4, &fake_pixels)
            .expect("Failed to write frame");

        let snapshot = viewport
            .read_latest_frame()
            .expect("Failed to read frame")
            .expect("Expected frame snapshot");

        assert_eq!(snapshot.sequence, 1);
        assert_eq!(snapshot.width, 100);
        assert_eq!(snapshot.height, 100);
        assert_eq!(snapshot.channels, 4);
        assert_eq!(snapshot.pixels.len(), 100 * 100 * 4);
        assert_eq!(snapshot.pixels[0], 128);

        let _ = std::fs::remove_file(shm_path);
    }
}
