/// Automatic priority tagger for telemetry chunks
/// 
/// Analyzes chunk content and automatically assigns priority tags
/// based on content patterns, keywords, and semantic analysis
/// 
/// Supports all data formats:
/// - Text/JSON/XML (keyword-based detection)
/// - Binary data (pattern-based detection)
/// - Structured data (format-aware detection)
/// - Any format (embedding-based fallback)

use crate::telemetry_ai::Severity;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Priority levels for telemetry chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ChunkPriority {
    Critical = 0,   // Emergency, immediate action required
    High = 64,      // Important, time-sensitive
    Normal = 128,   // Standard priority
    Low = 192,      // Background, can be delayed
    Bulk = 255,     // Bulk transfers, lowest priority
}

impl From<u8> for ChunkPriority {
    fn from(value: u8) -> Self {
        match value {
            0..=31 => ChunkPriority::Critical,
            32..=95 => ChunkPriority::High,
            96..=159 => ChunkPriority::Normal,
            160..=223 => ChunkPriority::Low,
            _ => ChunkPriority::Bulk,
        }
    }
}

impl From<ChunkPriority> for u8 {
    fn from(priority: ChunkPriority) -> Self {
        priority as u8
    }
}

/// Data format detection for format-aware priority tagging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Text,       // Plain text, logs
    Json,       // JSON format
    Xml,        // XML format
    Image,      // Image formats (JPEG, PNG, GIF, WebP, etc.)
    Video,      // Video formats (MP4, AVI, MOV, WebM, etc.)
    Audio,      // Audio formats (MP3, AAC, OGG, etc.)
    Medical,    // Medical data (HL7, DICOM, FHIR, etc.)
    Disaster,   // Disaster response data (emergency alerts, etc.)
    Engineering, // Engineering data (CAD, sensor data, etc.)
    Binary,     // Binary data
    Structured, // Structured binary (protobuf, msgpack, etc.)
    Unknown,    // Unknown format
}

/// Scenario/use case classification for priority tagging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataScenario {
    MediaStudio,      // Media production, broadcasting
    RuralLab,         // Rural laboratory, research
    MobileClinic,     // Mobile medical clinic
    RemoteEngineering, // Remote engineering sites
    DisasterSite,    // Disaster response, emergency
    MedicalFacility,  // Hospital, clinic, medical facility
    FieldOps,        // Field operations, remote sites
    Unknown,         // Unknown scenario
}

/// Priority tagger that analyzes telemetry chunks
/// Works with ALL data formats: text, JSON, binary, structured, etc.
pub struct PriorityTagger {
    // Critical keywords that indicate high priority
    critical_keywords: Vec<&'static [u8]>,
    high_keywords: Vec<&'static [u8]>,
    low_keywords: Vec<&'static [u8]>,
}

impl PriorityTagger {
    /// Create a new priority tagger with default keyword patterns
    pub fn new() -> Self {
        Self {
            critical_keywords: vec![
                // General critical
                b"Alert",
                b"Critical",
                b"Emergency",
                b"Fire",
                b"Failure",
                b"Down",
                b"Error",
                b"Fatal",
                b"Crash",
                b"Outage",
                b"Exception",
                b"Panic",
                // Medical critical
                b"Code Blue",
                b"Code Red",
                b"Cardiac Arrest",
                b"Respiratory Failure",
                b"Critical Patient",
                b"Life Threatening",
                b"Emergency Surgery",
                b"Trauma",
                // Disaster critical
                b"Evacuate",
                b"Immediate Danger",
                b"Structural Collapse",
                b"Chemical Spill",
                b"Radiation Alert",
                b"Natural Disaster",
            ],
            high_keywords: vec![
                // General high
                b"Warning",
                b"Degraded",
                b"Slow",
                b"High",
                b"Exceeded",
                b"Threshold",
                b"Anomaly",
                b"Timeout",
                // Medical high
                b"Patient Status",
                b"Vital Signs",
                b"Lab Results",
                b"Diagnosis",
                b"Treatment",
                b"Medication",
                b"Patient Data",
                // Engineering high
                b"Sensor Alert",
                b"Equipment Status",
                b"System Health",
                b"Performance",
            ],
            low_keywords: vec![
                b"Status",
                b"OK",
                b"Normal",
                b"Idle",
                b"Info",
                b"Log",
                b"Debug",
                b"Heartbeat",
            ],
        }
    }

    /// Create a priority tagger with custom keywords
    pub fn with_keywords(
        critical: Vec<&'static [u8]>,
        high: Vec<&'static [u8]>,
        low: Vec<&'static [u8]>,
    ) -> Self {
        Self {
            critical_keywords: critical,
            high_keywords: high,
            low_keywords: low,
        }
    }

    /// Detect data format from chunk content
    /// Includes detection for images, videos, and audio files
    pub fn detect_format(&self, chunk_data: &[u8]) -> DataFormat {
        if chunk_data.is_empty() {
            return DataFormat::Unknown;
        }

        // Check for image formats (magic numbers)
        if chunk_data.len() >= 4 {
            let magic = &chunk_data[0..4.min(chunk_data.len())];
            
            // JPEG: FF D8 FF
            if chunk_data.len() >= 3 && chunk_data[0] == 0xFF && chunk_data[1] == 0xD8 && chunk_data[2] == 0xFF {
                return DataFormat::Image;
            }
            
            // PNG: 89 50 4E 47
            if magic == &b"\x89PNG"[..] {
                return DataFormat::Image;
            }
            
            // GIF: 47 49 46 38 (GIF8)
            if chunk_data.len() >= 6 && 
               (chunk_data.starts_with(b"GIF89a") || chunk_data.starts_with(b"GIF87a")) {
                return DataFormat::Image;
            }
            
            // WebP: RIFF...WEBP
            if chunk_data.len() >= 12 && chunk_data.starts_with(b"RIFF") && chunk_data[8..12].starts_with(b"WEBP") {
                return DataFormat::Image;
            }
            
            // BMP: 42 4D
            if magic.starts_with(b"BM") {
                return DataFormat::Image;
            }
            
            // TIFF: 49 49 2A 00 (little-endian) or 4D 4D 00 2A (big-endian)
            if magic == &b"II*\x00"[..] || magic == &b"MM\x00*"[..] {
                return DataFormat::Image;
            }
        }
        
        // Check for video formats
        if chunk_data.len() >= 12 {
            // MP4: ftyp box at offset 4
            if chunk_data.len() >= 12 && chunk_data[4..8].starts_with(b"ftyp") {
                // Check for MP4 brand signatures
                if chunk_data.len() >= 12 {
                    let brand = &chunk_data[8..12];
                    if brand == b"mp41" || brand == b"mp42" || brand == b"isom" || brand == b"avc1" {
                        return DataFormat::Video;
                    }
                }
            }
            
            // AVI: RIFF...AVI
            if chunk_data.starts_with(b"RIFF") && chunk_data.len() >= 8 && chunk_data[8..11].starts_with(b"AVI") {
                return DataFormat::Video;
            }
            
            // MOV/QuickTime: ftyp at offset 4
            if chunk_data.len() >= 12 && chunk_data[4..8].starts_with(b"ftyp") {
                let brand = &chunk_data[8..12];
                if brand == b"qt  " || brand == b"moov" {
                    return DataFormat::Video;
                }
            }
            
            // WebM: 1A 45 DF A3
            if chunk_data.len() >= 12 && chunk_data.starts_with(b"\x1A\x45\xDF\xA3") {
                return DataFormat::Video;
            }
            
            // MKV: Same as WebM (Matroska)
            if chunk_data.len() >= 12 && chunk_data.starts_with(b"\x1A\x45\xDF\xA3") {
                return DataFormat::Video;
            }
        }
        
        // Check for audio formats
        if chunk_data.len() >= 12 {
            // WAV: RIFF...WAVE
            if chunk_data.starts_with(b"RIFF") && chunk_data.len() >= 12 && chunk_data[8..12].starts_with(b"WAVE") {
                return DataFormat::Audio;
            }
            
            // MP3: FF FB or FF F3 (MPEG-1 Layer 3)
            if chunk_data.len() >= 2 && chunk_data[0] == 0xFF && (chunk_data[1] & 0xE0) == 0xE0 {
                return DataFormat::Audio;
            }
            
            // OGG: OggS
            if chunk_data.len() >= 12 && chunk_data.starts_with(b"OggS") {
                return DataFormat::Audio;
            }
            
            // FLAC: fLaC
            if chunk_data.len() >= 12 && chunk_data.starts_with(b"fLaC") {
                return DataFormat::Audio;
            }
        }

        // Check for medical formats
        // HL7: Starts with MSH| (Message Header)
        if chunk_data.len() >= 12 && chunk_data.starts_with(b"MSH|") {
            return DataFormat::Medical;
        }
        
        // DICOM: Starts with DICM (at offset 128) or has DICM tag
        if chunk_data.len() >= 132 {
            if &chunk_data[128..132] == b"DICM" {
                return DataFormat::Medical;
            }
        }
        
        // FHIR JSON: Contains "resourceType" and medical keywords
        if chunk_data.starts_with(b"{") {
            let json_str = String::from_utf8_lossy(chunk_data);
            if json_str.contains("resourceType") && 
               (json_str.contains("Patient") || json_str.contains("Observation") || 
                json_str.contains("DiagnosticReport") || json_str.contains("Medication")) {
                return DataFormat::Medical;
            }
        }
        
        // Check for disaster/emergency data
        let text_lower = chunk_data.to_ascii_lowercase();
        if text_lower.windows(8).any(|w| w == b"evacuate") || text_lower.windows(8).any(|w| w == b"disaster") ||
           text_lower.windows(18).any(|w| w == b"emergency response") || text_lower.windows(6).any(|w| w == b"crisis") {
            return DataFormat::Disaster;
        }
        
        // Check for engineering data (CAD, sensor data)
        if text_lower.windows(3).any(|w| w == b"cad") || text_lower.windows(6).any(|w| w == b"sensor") ||
           text_lower.windows(9).any(|w| w == b"telemetry") || text_lower.windows(11).any(|w| w == b"measurement") {
            return DataFormat::Engineering;
        }

        // Check for JSON (starts with { or [)
        if chunk_data.starts_with(b"{") || chunk_data.starts_with(b"[") {
            // Verify it's likely JSON (contains quotes, colons, etc.)
            let json_chars = chunk_data.iter()
                .filter(|&&b| b == b'"' || b == b':' || b == b',' || b == b'{' || b == b'[')
                .count();
            if json_chars > chunk_data.len() / 10 {
                return DataFormat::Json;
            }
        }

        // Check for XML (starts with <)
        if chunk_data.starts_with(b"<") {
            // Check for HL7 XML
            let xml_str = String::from_utf8_lossy(&chunk_data[0..chunk_data.len().min(200)]);
            if xml_str.contains("urn:hl7-org") || xml_str.contains("ClinicalDocument") {
                return DataFormat::Medical;
            }
            return DataFormat::Xml;
        }

        // Check if it's mostly ASCII text
        let ascii_count = chunk_data.iter()
            .filter(|&b| b.is_ascii() && (b.is_ascii_alphanumeric() || b.is_ascii_whitespace() || b.is_ascii_punctuation()))
            .count();
        let ascii_ratio = ascii_count as f32 / chunk_data.len() as f32;

        if ascii_ratio > 0.8 {
            DataFormat::Text
        } else if ascii_ratio > 0.3 {
            // Mixed - might be structured binary with text headers
            DataFormat::Structured
        } else {
            // Mostly binary
            DataFormat::Binary
        }
    }

    /// Tag priority based on chunk content analysis
    /// 
    /// Works with ALL data formats:
    /// - Text/JSON/XML: Keyword-based detection
    /// - Binary: Pattern-based detection (size, entropy)
    /// - Structured: Format-aware detection
    /// 
    /// # Arguments
    /// * `chunk_data` - Raw telemetry chunk bytes (any format)
    /// * `severity` - AI-determined severity (optional, can override)
    /// 
    /// # Returns
    /// Priority level for the chunk
    pub fn tag_priority(&self, chunk_data: &[u8], severity: Option<Severity>) -> ChunkPriority {
        // If severity is provided and is High, prioritize accordingly
        if let Some(Severity::High) = severity {
            return ChunkPriority::Critical;
        }

        // Detect format for format-aware processing
        let format = self.detect_format(chunk_data);

        match format {
            DataFormat::Text | DataFormat::Json | DataFormat::Xml => {
                self.tag_text_format(chunk_data)
            }
            DataFormat::Image => {
                self.tag_image_format(chunk_data)
            }
            DataFormat::Video => {
                self.tag_video_format(chunk_data)
            }
            DataFormat::Audio => {
                self.tag_audio_format(chunk_data)
            }
            DataFormat::Medical => {
                self.tag_medical_format(chunk_data)
            }
            DataFormat::Disaster => {
                self.tag_disaster_format(chunk_data)
            }
            DataFormat::Engineering => {
                self.tag_engineering_format(chunk_data)
            }
            DataFormat::Binary => {
                self.tag_binary_format(chunk_data)
            }
            DataFormat::Structured => {
                // Try text-based first, fallback to binary heuristics
                let text_priority = self.tag_text_format(chunk_data);
                if text_priority != ChunkPriority::Normal {
                    text_priority
                } else {
                    self.tag_binary_format(chunk_data)
                }
            }
            DataFormat::Unknown => {
                ChunkPriority::Normal
            }
        }
    }

    /// Tag priority for text-based formats (Text, JSON, XML)
    fn tag_text_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        // Convert to lowercase for case-insensitive matching
        let chunk_lower = chunk_data.to_ascii_lowercase();

        // Check for critical keywords
        for keyword in &self.critical_keywords {
            if chunk_lower.windows(keyword.len()).any(|w| {
                w.eq_ignore_ascii_case(keyword)
            }) {
                return ChunkPriority::Critical;
            }
        }

        // Check for high priority keywords
        for keyword in &self.high_keywords {
            if chunk_lower.windows(keyword.len()).any(|w| {
                w.eq_ignore_ascii_case(keyword)
            }) {
                return ChunkPriority::High;
            }
        }

        // Check for low priority keywords
        let mut low_count = 0;
        for keyword in &self.low_keywords {
            if chunk_lower.windows(keyword.len()).any(|w| {
                w.eq_ignore_ascii_case(keyword)
            }) {
                low_count += 1;
            }
        }

        // If multiple low-priority indicators, likely low priority
        if low_count >= 2 {
            return ChunkPriority::Low;
        }

        // Pattern-based analysis for text formats
        // Check for numeric patterns that might indicate metrics
        let numeric_ratio = chunk_data.iter()
            .filter(|&&b| b.is_ascii_digit() || b == b'.' || b == b',' || b == b':')
            .count() as f32 / chunk_data.len().max(1) as f32;

        // High numeric ratio might indicate bulk metrics data
        if numeric_ratio > 0.7 && chunk_data.len() > 1000 {
            return ChunkPriority::Bulk;
        }

        // Check for JSON arrays (often bulk data)
        if chunk_data.starts_with(b"[") && chunk_data.len() > 5000 {
            return ChunkPriority::Bulk;
        }

        // Default to normal priority
        ChunkPriority::Normal
    }

    /// Tag priority for image formats
    /// Images are typically:
    /// - Small images (< 1MB): Normal priority (can be sent quickly)
    /// - Medium images (1-10MB): Normal priority
    /// - Large images (> 10MB): Low priority (bulk transfer)
    /// - Thumbnails (< 100KB): High priority (preview/thumbnail)
    fn tag_image_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        let size = chunk_data.len();
        
        // Thumbnails/small previews - high priority
        if size < 100_000 {  // < 100KB
            return ChunkPriority::High;
        }
        
        // Large images - low priority (bulk)
        if size > 10_000_000 {  // > 10MB
            return ChunkPriority::Low;
        }
        
        // Very large images - bulk priority
        if size > 50_000_000 {  // > 50MB
            return ChunkPriority::Bulk;
        }
        
        // Normal images - standard priority
        ChunkPriority::Normal
    }

    /// Tag priority for video formats
    /// Videos are typically:
    /// - Small clips (< 10MB): High priority (important content)
    /// - Medium videos (10-100MB): Normal priority
    /// - Large videos (> 100MB): Low/Bulk priority (streaming-friendly)
    /// - Live streams: Critical/High priority (real-time)
    fn tag_video_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        let size = chunk_data.len();
        
        // Small video clips - high priority (important content)
        if size < 10_000_000 {  // < 10MB
            return ChunkPriority::High;
        }
        
        // Medium videos - normal priority
        if size < 100_000_000 {  // < 100MB
            return ChunkPriority::Normal;
        }
        
        // Large videos - low priority (streaming-friendly, can be chunked)
        if size < 500_000_000 {  // < 500MB
            return ChunkPriority::Low;
        }
        
        // Very large videos - bulk priority
        ChunkPriority::Bulk
    }

    /// Tag priority for audio formats
    /// Audio files are typically:
    /// - Small audio (< 5MB): High priority (voice messages, alerts)
    /// - Medium audio (5-50MB): Normal priority
    /// - Large audio (> 50MB): Low priority
    fn tag_audio_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        let size = chunk_data.len();
        
        // Small audio files - high priority (voice messages, alerts)
        if size < 5_000_000 {  // < 5MB
            return ChunkPriority::High;
        }
        
        // Medium audio - normal priority
        if size < 50_000_000 {  // < 50MB
            return ChunkPriority::Normal;
        }
        
        // Large audio - low priority
        ChunkPriority::Low
    }

    /// Tag priority for medical data formats
    /// Medical data is typically:
    /// - Critical alerts: Critical priority (Code Blue, Code Red, etc.)
    /// - Patient data: High priority (vital signs, lab results)
    /// - Routine data: Normal priority (scheduled reports)
    fn tag_medical_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        let text_lower = chunk_data.to_ascii_lowercase();
        
        // Critical medical alerts - always critical
        if text_lower.windows(9).any(|w| w == b"code blue") || text_lower.windows(8).any(|w| w == b"code red") ||
           text_lower.windows(14).any(|w| w == b"cardiac arrest") || text_lower.windows(19).any(|w| w == b"respiratory failure") ||
           text_lower.windows(16).any(|w| w == b"life threatening") || text_lower.windows(17).any(|w| w == b"emergency surgery") {
            return ChunkPriority::Critical;
        }
        
        // Patient critical data - high priority
        if text_lower.windows(16).any(|w| w == b"critical patient") || text_lower.windows(6).any(|w| w == b"trauma") ||
           text_lower.windows(11).any(|w| w == b"vital signs") || text_lower.windows(11).any(|w| w == b"lab results") {
            return ChunkPriority::High;
        }
        
        // DICOM images - high priority (medical imaging)
        if chunk_data.len() >= 12 && &chunk_data[128..132] == b"DICM" {
            return ChunkPriority::High;
        }
        
        // Routine medical data - normal priority
        ChunkPriority::Normal
    }

    /// Tag priority for disaster/emergency data
    /// Disaster data is typically:
    /// - Immediate threats: Critical priority
    /// - Emergency alerts: High priority
    /// - Status updates: Normal priority
    fn tag_disaster_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        let text_lower = chunk_data.to_ascii_lowercase();
        
        // Immediate danger - critical priority
        if text_lower.windows(8).any(|w| w == b"evacuate") || text_lower.windows(16).any(|w| w == b"immediate danger") ||
           text_lower.windows(19).any(|w| w == b"structural collapse") || text_lower.windows(14).any(|w| w == b"chemical spill") ||
           text_lower.windows(15).any(|w| w == b"radiation alert") {
            return ChunkPriority::Critical;
        }
        
        // Emergency alerts - high priority
        if text_lower.windows(9).any(|w| w == b"emergency") || text_lower.windows(8).any(|w| w == b"disaster") ||
           text_lower.windows(6).any(|w| w == b"crisis") || text_lower.windows(5).any(|w| w == b"alert") {
            return ChunkPriority::High;
        }
        
        // Status updates - normal priority
        ChunkPriority::Normal
    }

    /// Tag priority for engineering data
    /// Engineering data is typically:
    /// - Critical alerts: High priority (equipment failure, sensor alerts)
    /// - Sensor data: Normal priority
    /// - Routine logs: Low priority
    fn tag_engineering_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        let text_lower = chunk_data.to_ascii_lowercase();
        
        // Critical engineering alerts - high priority
        if text_lower.windows(12).any(|w| w == b"sensor alert") || text_lower.windows(17).any(|w| w == b"equipment failure") ||
           text_lower.windows(14).any(|w| w == b"system failure") || text_lower.windows(8).any(|w| w == b"critical") {
            return ChunkPriority::High;
        }
        
        // Sensor data - normal priority
        if text_lower.windows(6).any(|w| w == b"sensor") || text_lower.windows(9).any(|w| w == b"telemetry") ||
           text_lower.windows(11).any(|w| w == b"measurement") || text_lower.windows(16).any(|w| w == b"equipment status") {
            return ChunkPriority::Normal;
        }
        
        // Routine logs - low priority
        ChunkPriority::Low
    }

    /// Detect scenario/use case from chunk content
    pub fn detect_scenario(&self, chunk_data: &[u8]) -> DataScenario {
        let text_lower = chunk_data.to_ascii_lowercase();
        
        // Medical scenarios
        if text_lower.windows(7).any(|w| w == b"patient") || text_lower.windows(7).any(|w| w == b"medical") ||
           text_lower.windows(8).any(|w| w == b"hospital") || text_lower.windows(6).any(|w| w == b"clinic") ||
           text_lower.windows(9).any(|w| w == b"diagnosis") || text_lower.windows(9).any(|w| w == b"treatment") {
            if text_lower.windows(6).any(|w| w == b"mobile") || text_lower.windows(5).any(|w| w == b"field") {
                return DataScenario::MobileClinic;
            }
            return DataScenario::MedicalFacility;
        }
        
        // Disaster scenarios
        if text_lower.windows(8).any(|w| w == b"disaster") || text_lower.windows(9).any(|w| w == b"emergency") ||
           text_lower.windows(8).any(|w| w == b"evacuate") || text_lower.windows(6).any(|w| w == b"crisis") {
            return DataScenario::DisasterSite;
        }
        
        // Engineering scenarios
        if text_lower.windows(11).any(|w| w == b"engineering") || text_lower.windows(11).any(|w| w == b"remote site") ||
           text_lower.windows(9).any(|w| w == b"field ops") || text_lower.windows(12).any(|w| w == b"construction") {
            return DataScenario::RemoteEngineering;
        }
        
        // Media scenarios
        if text_lower.windows(5).any(|w| w == b"media") || text_lower.windows(9).any(|w| w == b"broadcast") ||
           text_lower.windows(10).any(|w| w == b"production") || text_lower.windows(6).any(|w| w == b"studio") {
            return DataScenario::MediaStudio;
        }
        
        // Lab scenarios
        if text_lower.windows(3).any(|w| w == b"lab") || text_lower.windows(10).any(|w| w == b"laboratory") ||
           text_lower.windows(8).any(|w| w == b"research") || text_lower.windows(5).any(|w| w == b"rural") {
            return DataScenario::RuralLab;
        }
        
        // Field operations
        if text_lower.windows(5).any(|w| w == b"field") || text_lower.windows(6).any(|w| w == b"remote") {
            return DataScenario::FieldOps;
        }
        
        DataScenario::Unknown
    }

    /// Tag priority for binary formats
    fn tag_binary_format(&self, chunk_data: &[u8]) -> ChunkPriority {
        // Binary format heuristics
        
        // Very small chunks might be control messages (higher priority)
        if chunk_data.len() < 64 {
            return ChunkPriority::High;
        }

        // Very large chunks are likely bulk data
        if chunk_data.len() > 1_000_000 {
            return ChunkPriority::Bulk;
        }

        // Calculate entropy (randomness measure)
        // High entropy = encrypted/compressed data (likely bulk)
        // Low entropy = structured data (might be important)
        let entropy = self.calculate_entropy(chunk_data);
        
        if entropy > 7.5 {
            // Very high entropy - likely encrypted/compressed bulk data
            return ChunkPriority::Bulk;
        } else if entropy < 3.0 {
            // Low entropy - structured data, might be important
            // Check size - small structured = control message (high priority)
            if chunk_data.len() < 1000 {
                return ChunkPriority::High;
            }
        }

        // Check for common binary patterns
        // Magic numbers for common formats
        if chunk_data.len() >= 4 {
            let magic = &chunk_data[0..4];
            
            // Compressed formats (likely bulk)
            // Note: PNG is handled in Image format, not here
            if magic == &b"PK\x03\x04"[..] || // ZIP
               magic == &b"\x1f\x8b"[..] || // GZIP
               magic == &b"BZ"[..] { // BZIP2
                return ChunkPriority::Bulk;
            }
        }

        // Default for binary: Normal priority
        ChunkPriority::Normal
    }

    /// Calculate Shannon entropy of data
    /// Higher entropy = more random/compressed data
    fn calculate_entropy(&self, data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        let mut frequency = [0u32; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let len = data.len() as f32;
        let mut entropy = 0.0;

        for &count in &frequency {
            if count > 0 {
                let probability = count as f32 / len;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// Tag priority based on embedding similarity to known patterns
    /// 
    /// This method can be extended to use ML-based classification
    /// Works with any data format (embeddings are format-agnostic)
    pub fn tag_priority_from_embedding(
        &self,
        embedding: &[f32],
        severity: Option<Severity>,
    ) -> ChunkPriority {
        // If severity is provided, use it
        if let Some(Severity::High) = severity {
            return ChunkPriority::Critical;
        }

        // Simple heuristic: check embedding variance
        // High variance might indicate important/unique data
        let mean = embedding.iter().sum::<f32>() / embedding.len() as f32;
        let variance = embedding.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / embedding.len() as f32;

        if variance > 0.1 {
            ChunkPriority::High
        } else if variance < 0.01 {
            ChunkPriority::Low
        } else {
            ChunkPriority::Normal
        }
    }
}

impl Default for PriorityTagger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_priority_text() {
        let tagger = PriorityTagger::new();
        let chunk = b"Alert: Fire detected in building A";
        let priority = tagger.tag_priority(chunk, None);
        assert_eq!(priority, ChunkPriority::Critical);
    }

    #[test]
    fn test_critical_priority_json() {
        let tagger = PriorityTagger::new();
        let chunk = br#"{"type":"Alert","message":"Critical error detected"}"#;
        let priority = tagger.tag_priority(chunk, None);
        assert_eq!(priority, ChunkPriority::Critical);
    }

    #[test]
    fn test_image_format_detection() {
        let tagger = PriorityTagger::new();
        // PNG magic number
        let png_data = b"\x89PNG\r\n\x1a\n";
        let format = tagger.detect_format(png_data);
        assert_eq!(format, DataFormat::Image);
        
        // JPEG magic number
        let jpeg_data = b"\xFF\xD8\xFF\xE0";
        let format = tagger.detect_format(jpeg_data);
        assert_eq!(format, DataFormat::Image);
    }

    #[test]
    fn test_video_format_detection() {
        let tagger = PriorityTagger::new();
        // MP4 magic number (ftyp box)
        let mut mp4_data = vec![0u8; 20];
        mp4_data[4..8].copy_from_slice(b"ftyp");
        mp4_data[8..12].copy_from_slice(b"mp41");
        let format = tagger.detect_format(&mp4_data);
        assert_eq!(format, DataFormat::Video);
    }

    #[test]
    fn test_image_priority_tagging() {
        let tagger = PriorityTagger::new();
        // Small image (thumbnail) - high priority
        let small_image = vec![0xFF, 0xD8, 0xFF]; // JPEG header + small data
        let mut thumbnail = small_image.clone();
        thumbnail.extend(vec![0u8; 50_000]); // 50KB total
        let priority = tagger.tag_priority(&thumbnail, None);
        assert_eq!(priority, ChunkPriority::High);
        
        // Large image - low priority
        let mut large_image = small_image.clone();
        large_image.extend(vec![0u8; 15_000_000]); // 15MB
        let priority = tagger.tag_priority(&large_image, None);
        assert_eq!(priority, ChunkPriority::Low);
    }

    #[test]
    fn test_video_priority_tagging() {
        let tagger = PriorityTagger::new();
        // Small video clip - high priority
        let mut small_video = vec![0u8; 20];
        small_video[4..8].copy_from_slice(b"ftyp");
        small_video[8..12].copy_from_slice(b"mp41");
        small_video.extend(vec![0u8; 5_000_000]); // 5MB total
        let priority = tagger.tag_priority(&small_video, None);
        assert_eq!(priority, ChunkPriority::High);
        
        // Large video - bulk priority
        let mut large_video = small_video.clone();
        large_video.truncate(20);
        large_video.extend(vec![0u8; 600_000_000]); // 600MB
        let priority = tagger.tag_priority(&large_video, None);
        assert_eq!(priority, ChunkPriority::Bulk);
    }

    #[test]
    fn test_binary_format_detection() {
        let tagger = PriorityTagger::new();
        let binary_data = vec![0u8; 1000];
        let format = tagger.detect_format(&binary_data);
        assert_eq!(format, DataFormat::Binary);
    }

    #[test]
    fn test_json_format_detection() {
        let tagger = PriorityTagger::new();
        let json_data = br#"{"key":"value","array":[1,2,3]}"#;
        let format = tagger.detect_format(json_data);
        assert_eq!(format, DataFormat::Json);
    }

    #[test]
    fn test_binary_bulk_detection() {
        let tagger = PriorityTagger::new();
        let large_binary = vec![0u8; 2_000_000]; // 2MB
        let priority = tagger.tag_priority(&large_binary, None);
        assert_eq!(priority, ChunkPriority::Bulk);
    }

    #[test]
    fn test_binary_small_high_priority() {
        let tagger = PriorityTagger::new();
        let small_binary = vec![0u8; 32]; // Small control message
        let priority = tagger.tag_priority(&small_binary, None);
        assert_eq!(priority, ChunkPriority::High);
    }

    #[test]
    fn test_severity_override() {
        let tagger = PriorityTagger::new();
        let chunk = b"Regular telemetry data";
        let priority = tagger.tag_priority(chunk, Some(Severity::High));
        assert_eq!(priority, ChunkPriority::Critical);
    }
}
