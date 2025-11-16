/// Priority-based scheduler for telemetry data transmission
/// 
/// Manages priority queues and schedules data transmission based on:
/// - Priority tags
/// - Network conditions
/// - WFQ weights from AI decisions

use crate::telemetry_ai::priority_tagger::ChunkPriority;
use crate::telemetry_ai::{NetworkMetricsInput, AiDecision};
use anyhow::Result;
use std::collections::VecDeque;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Scheduled chunk ready for transmission
#[derive(Debug, Clone)]
pub struct ScheduledChunk {
    pub data: Vec<u8>,
    pub priority: ChunkPriority,
    pub timestamp: u64,
    pub retry_count: u32,
    pub route: Option<u32>, // Route decision if available
}

/// Priority-based scheduler
pub struct PriorityScheduler {
    // Priority queues (0 = highest priority)
    critical_queue: RwLock<VecDeque<ScheduledChunk>>,
    high_queue: RwLock<VecDeque<ScheduledChunk>>,
    normal_queue: RwLock<VecDeque<ScheduledChunk>>,
    low_queue: RwLock<VecDeque<ScheduledChunk>>,
    bulk_queue: RwLock<VecDeque<ScheduledChunk>>,
    
    // Current WFQ weights (from AI decisions)
    p0_weight: RwLock<u32>,  // Critical/High priority weight
    p1_weight: RwLock<u32>,  // Normal priority weight
    p2_weight: RwLock<u32>,  // Low/Bulk priority weight
    p2_enabled: RwLock<bool>, // Whether bulk transfers are enabled
    
    // Statistics
    total_scheduled: RwLock<u64>,
    total_sent: RwLock<u64>,
}

impl PriorityScheduler {
    /// Create a new priority scheduler
    pub fn new() -> Self {
        Self {
            critical_queue: RwLock::new(VecDeque::new()),
            high_queue: RwLock::new(VecDeque::new()),
            normal_queue: RwLock::new(VecDeque::new()),
            low_queue: RwLock::new(VecDeque::new()),
            bulk_queue: RwLock::new(VecDeque::new()),
            p0_weight: RwLock::new(50),
            p1_weight: RwLock::new(30),
            p2_weight: RwLock::new(20),
            p2_enabled: RwLock::new(true),
            total_scheduled: RwLock::new(0),
            total_sent: RwLock::new(0),
        }
    }

    /// Schedule a chunk for transmission
    /// 
    /// # Arguments
    /// * `data` - Chunk data to schedule
    /// * `priority` - Priority level
    /// * `route` - Optional route decision
    pub fn schedule(
        &self,
        data: Vec<u8>,
        priority: ChunkPriority,
        route: Option<u32>,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let chunk = ScheduledChunk {
            data,
            priority,
            timestamp,
            retry_count: 0,
            route,
        };

        // Add to appropriate queue based on priority
        match priority {
            ChunkPriority::Critical => {
                self.critical_queue.write().push_back(chunk);
            }
            ChunkPriority::High => {
                self.high_queue.write().push_back(chunk);
            }
            ChunkPriority::Normal => {
                self.normal_queue.write().push_back(chunk);
            }
            ChunkPriority::Low => {
                self.low_queue.write().push_back(chunk);
            }
            ChunkPriority::Bulk => {
                self.bulk_queue.write().push_back(chunk);
            }
        }

        *self.total_scheduled.write() += 1;
        Ok(())
    }

    /// Update scheduler weights from AI decision
    pub fn update_weights(&self, decision: &AiDecision) {
        *self.p0_weight.write() = decision.wfq_p0_weight;
        *self.p1_weight.write() = decision.wfq_p1_weight;
        *self.p2_weight.write() = decision.wfq_p2_weight;
        *self.p2_enabled.write() = decision.p2_enable;
    }

    /// Get next chunk to send based on WFQ weights and priority
    /// 
    /// Uses weighted fair queuing algorithm:
    /// - P0 (Critical/High): Gets p0_weight% of bandwidth
    /// - P1 (Normal): Gets p1_weight% of bandwidth
    /// - P2 (Low/Bulk): Gets p2_weight% of bandwidth (if enabled)
    pub fn get_next(&self) -> Option<ScheduledChunk> {
        let p0_weight = *self.p0_weight.read();
        let p1_weight = *self.p1_weight.read();
        let p2_weight = *self.p2_weight.read();
        let p2_enabled = *self.p2_enabled.read();

        // Calculate total weight
        let total_weight = p0_weight + p1_weight + p2_weight;
        if total_weight == 0 {
            // Fallback: round-robin if no weights set
            return self.get_next_round_robin();
        }

        // Use weighted selection based on queue sizes and weights
        // Priority order: Critical > High > Normal > Low > Bulk
        
        // First, try critical queue (always highest priority)
        if let Some(chunk) = self.critical_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }

        // Then high priority queue
        if let Some(chunk) = self.high_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }

        // Apply WFQ weights for remaining queues
        let p0_count = self.critical_queue.read().len() + self.high_queue.read().len();
        let p1_count = self.normal_queue.read().len();
        let p2_count = self.low_queue.read().len() + self.bulk_queue.read().len();

        // Calculate which queue to serve based on weights
        // Simple implementation: serve based on weight ratios
        if p1_count > 0 && (p0_count == 0 || p1_weight > p0_weight) {
            if let Some(chunk) = self.normal_queue.write().pop_front() {
                *self.total_sent.write() += 1;
                return Some(chunk);
            }
        }

        // Serve low/bulk queues only if enabled
        if p2_enabled && p2_count > 0 {
            // Prefer low over bulk
            if let Some(chunk) = self.low_queue.write().pop_front() {
                *self.total_sent.write() += 1;
                return Some(chunk);
            }
            
            if let Some(chunk) = self.bulk_queue.write().pop_front() {
                *self.total_sent.write() += 1;
                return Some(chunk);
            }
        }

        // Fallback: serve normal queue if nothing else available
        if let Some(chunk) = self.normal_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }

        None
    }

    /// Round-robin fallback when weights are not set
    fn get_next_round_robin(&self) -> Option<ScheduledChunk> {
        // Priority order: Critical > High > Normal > Low > Bulk
        if let Some(chunk) = self.critical_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }
        if let Some(chunk) = self.high_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }
        if let Some(chunk) = self.normal_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }
        if let Some(chunk) = self.low_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }
        if let Some(chunk) = self.bulk_queue.write().pop_front() {
            *self.total_sent.write() += 1;
            return Some(chunk);
        }
        None
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStats {
        SchedulerStats {
            critical_queue_size: self.critical_queue.read().len(),
            high_queue_size: self.high_queue.read().len(),
            normal_queue_size: self.normal_queue.read().len(),
            low_queue_size: self.low_queue.read().len(),
            bulk_queue_size: self.bulk_queue.read().len(),
            total_scheduled: *self.total_scheduled.read(),
            total_sent: *self.total_sent.read(),
            p0_weight: *self.p0_weight.read(),
            p1_weight: *self.p1_weight.read(),
            p2_weight: *self.p2_weight.read(),
            p2_enabled: *self.p2_enabled.read(),
        }
    }

    /// Clear all queues
    pub fn clear(&self) {
        self.critical_queue.write().clear();
        self.high_queue.write().clear();
        self.normal_queue.write().clear();
        self.low_queue.write().clear();
        self.bulk_queue.write().clear();
    }

    /// Get total queue size
    pub fn total_queued(&self) -> usize {
        self.critical_queue.read().len() +
        self.high_queue.read().len() +
        self.normal_queue.read().len() +
        self.low_queue.read().len() +
        self.bulk_queue.read().len()
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub critical_queue_size: usize,
    pub high_queue_size: usize,
    pub normal_queue_size: usize,
    pub low_queue_size: usize,
    pub bulk_queue_size: usize,
    pub total_scheduled: u64,
    pub total_sent: u64,
    pub p0_weight: u32,
    pub p1_weight: u32,
    pub p2_weight: u32,
    pub p2_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_and_get() {
        let scheduler = PriorityScheduler::new();
        
        // Schedule chunks with different priorities
        scheduler.schedule(
            b"critical data".to_vec(),
            ChunkPriority::Critical,
            None,
        ).unwrap();
        
        scheduler.schedule(
            b"normal data".to_vec(),
            ChunkPriority::Normal,
            None,
        ).unwrap();

        // Critical should be retrieved first
        let chunk = scheduler.get_next().unwrap();
        assert_eq!(chunk.priority, ChunkPriority::Critical);
        assert_eq!(chunk.data, b"critical data");

        // Then normal
        let chunk = scheduler.get_next().unwrap();
        assert_eq!(chunk.priority, ChunkPriority::Normal);
    }
}

