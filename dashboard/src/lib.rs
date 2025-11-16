//! Dashboard for PitlinkPQC System
//! 
//! Provides a web-based dashboard for monitoring:
//! - Telemetry AI decisions
//! - Network metrics and handover events
//! - QUIC-FEC connection status
//! - Compression statistics
//! - System performance

// Only expose modules that are actually used and have dependencies
pub mod api;
pub mod state;

// Comment out unused modules that have missing dependencies
// These can be enabled when dependencies are added to Cargo.toml
// pub mod metrics;
// pub mod server;
// pub mod integration;
// pub mod control;

