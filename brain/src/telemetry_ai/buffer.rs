/// Data buffer for patchy networks - stores data when network is down
use std::collections::VecDeque;
use parking_lot::RwLock;

/// Buffered telemetry chunk with metadata
#[derive(Debug, Clone)]
pub struct BufferedChunk {
    pub data: Vec<u8>,
    pub priority: u8,  // 0=highest, 255=lowest
    pub timestamp: u64,
    pub retry_count: u32,
}

/// Buffer manager for handling data during network outages
pub struct TelemetryBuffer {
    buffer: RwLock<VecDeque<BufferedChunk>>,
    max_size: usize,
    max_age_seconds: u64,
}

impl TelemetryBuffer {
    /// Create a new buffer with specified limits
    pub fn new(max_size: usize, max_age_seconds: u64) -> Self {
        Self {
            buffer: RwLock::new(VecDeque::new()),
            max_size,
            max_age_seconds,
        }
    }
    
    /// Add chunk to buffer (when network is down)
    pub fn add(&self, data: Vec<u8>, priority: u8) -> Result<()> {
        let mut buffer = self.buffer.write();
        
        // Remove old chunks
        self.cleanup_old(&mut buffer)?;
        
        // Check if buffer is full
        if buffer.len() >= self.max_size {
            // Remove lowest priority chunk
            if let Some(pos) = buffer.iter().position(|c| c.priority == 255) {
                buffer.remove(pos);
            } else {
                // Remove oldest if all same priority
                buffer.pop_front();
            }
        }
        
        let chunk = BufferedChunk {
            data,
            priority,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            retry_count: 0,
        };
        
        // Insert sorted by priority (highest first)
        let insert_pos = buffer.iter()
            .position(|c| c.priority > priority)
            .unwrap_or(buffer.len());
        buffer.insert(insert_pos, chunk);
        
        Ok(())
    }
    
    /// Get next chunk to send (highest priority first)
    pub fn pop(&self) -> Option<BufferedChunk> {
        let mut buffer = self.buffer.write();
        self.cleanup_old(&mut buffer).ok();
        buffer.pop_front()
    }
    
    /// Peek at next chunk without removing
    pub fn peek(&self) -> Option<BufferedChunk> {
        let buffer = self.buffer.read();
        buffer.front().cloned()
    }
    
    /// Get buffer status
    pub fn status(&self) -> BufferStatus {
        let buffer = self.buffer.read();
        BufferStatus {
            size: buffer.len(),
            max_size: self.max_size,
            is_full: buffer.len() >= self.max_size,
        }
    }
    
    /// Cleanup old chunks
    fn cleanup_old(&self, buffer: &mut VecDeque<BufferedChunk>) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        buffer.retain(|chunk| {
            now.saturating_sub(chunk.timestamp) < self.max_age_seconds
        });
        
        Ok(())
    }
    
    /// Clear buffer
    pub fn clear(&self) {
        let mut buffer = self.buffer.write();
        buffer.clear();
    }
}

/// Buffer status information
#[derive(Debug, Clone)]
pub struct BufferStatus {
    pub size: usize,
    pub max_size: usize,
    pub is_full: bool,
}

use anyhow::Result;

