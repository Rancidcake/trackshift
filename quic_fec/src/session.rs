//! Session Management
//!
//! Tracks client sessions, active transfers, and session state

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Instant, Duration};
use uuid::Uuid;
use anyhow::Result;

/// Session ID type
pub type SessionId = String;

/// Session
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: SessionId,
    pub connection_id: u64,
    pub client_id: String,
    pub user_id: String,
    pub connected_at: Instant,
    pub last_activity: Arc<RwLock<Instant>>,
    pub active_transfers: Arc<RwLock<Vec<String>>>, // transfer_ids
    pub network_metrics: Option<crate::handover::PathMetrics>,
}

impl Session {
    /// Create new session
    pub fn new(connection_id: u64, client_id: String, user_id: String) -> Self {
        let now = Instant::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            connection_id,
            client_id,
            user_id,
            connected_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            active_transfers: Arc::new(RwLock::new(Vec::new())),
            network_metrics: None,
        }
    }

    /// Update last activity
    pub fn update_activity(&self) {
        *self.last_activity.write() = Instant::now();
    }

    /// Check if session is expired
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.read().elapsed() > timeout
    }
}

/// Session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    session_timeout: Duration,
}

impl SessionManager {
    /// Create new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_timeout: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Create a new session
    pub async fn create_session(&self, session: Session) -> Result<()> {
        let session_id = session.session_id.clone();
        self.sessions.write().insert(session_id.clone(), session);
        Ok(())
    }

    /// Get session
    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().get(session_id).cloned()
    }

    /// Update session activity
    pub fn update_activity(&self, session_id: &str) {
        if let Some(session) = self.sessions.read().get(session_id) {
            session.update_activity();
        }
    }

    /// Add transfer to session
    pub async fn add_transfer(&self, session_id: &str, transfer_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.read().get(session_id) {
            session.active_transfers.write().push(transfer_id.to_string());
            session.update_activity();
        }
        Ok(())
    }

    /// Remove transfer from session
    pub async fn remove_transfer(&self, session_id: &str, transfer_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.read().get(session_id) {
            session.active_transfers.write().retain(|id| id != transfer_id);
        }
        Ok(())
    }

    /// Remove session
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        self.sessions.write().remove(session_id);
        Ok(())
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired(&self) -> Result<()> {
        let mut sessions = self.sessions.write();
        let mut to_remove = Vec::new();

        for (session_id, session) in sessions.iter() {
            if session.is_expired(self.session_timeout) {
                to_remove.push(session_id.clone());
            }
        }

        for session_id in to_remove {
            sessions.remove(&session_id);
        }

        Ok(())
    }

    /// Get active session count
    pub fn active_count(&self) -> usize {
        self.sessions.read().len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

