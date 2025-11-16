//! Authentication and Authorization
//!
//! Handles user authentication, permissions, and rate limiting

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub is_authenticated: bool,
    pub user_id: String,
    pub permissions: Permissions,
    pub rate_limits: RateLimits,
}

/// User permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissions {
    ReadOnly,
    ReadWrite,
    Admin,
}

/// Rate limits
#[derive(Debug, Clone)]
pub struct RateLimits {
    pub max_file_size: u64,
    pub max_transfer_rate: u64, // bytes/sec
    pub max_concurrent_transfers: usize,
    pub max_daily_transfer: u64,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024 * 1024, // 10GB
            max_transfer_rate: 100 * 1024 * 1024, // 100 MB/s
            max_concurrent_transfers: 10,
            max_daily_transfer: 100 * 1024 * 1024 * 1024, // 100GB/day
        }
    }
}

/// User
#[derive(Debug, Clone)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub permissions: Permissions,
    pub rate_limits: RateLimits,
}

/// Authentication manager
pub struct AuthManager {
    users: Arc<RwLock<HashMap<String, User>>>, // token -> user
    tokens: Arc<RwLock<HashMap<String, String>>>, // token -> user_id
}

impl AuthManager {
    /// Create new auth manager
    pub fn new() -> Self {
        let mut manager = Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
        };

        // Add default admin user
        manager.add_user(User {
            user_id: "admin".to_string(),
            username: "admin".to_string(),
            permissions: Permissions::Admin,
            rate_limits: RateLimits {
                max_file_size: u64::MAX,
                max_transfer_rate: u64::MAX,
                max_concurrent_transfers: 100,
                max_daily_transfer: u64::MAX,
            },
        }, Some("admin_token".to_string()));

        manager
    }

    /// Add a user
    pub fn add_user(&mut self, user: User, token: Option<String>) {
        let user_id = user.user_id.clone();
        self.users.write().insert(user_id.clone(), user);

        if let Some(token) = token {
            self.tokens.write().insert(token, user_id);
        }
    }

    /// Authenticate using token
    pub async fn authenticate(&self, token: Option<&String>) -> Result<AuthResult> {
        // Default: allow anonymous access with limited permissions
        if token.is_none() {
            return Ok(AuthResult {
                is_authenticated: true,
                user_id: "anonymous".to_string(),
                permissions: Permissions::ReadWrite,
                rate_limits: RateLimits {
                    max_file_size: 100 * 1024 * 1024, // 100MB
                    max_transfer_rate: 10 * 1024 * 1024, // 10 MB/s
                    max_concurrent_transfers: 3,
                    max_daily_transfer: 10 * 1024 * 1024 * 1024, // 10GB/day
                },
            });
        }

        let token = token.unwrap();
        let tokens = self.tokens.read();
        
        if let Some(user_id) = tokens.get(token) {
            let users = self.users.read();
            if let Some(user) = users.get(user_id) {
                return Ok(AuthResult {
                    is_authenticated: true,
                    user_id: user.user_id.clone(),
                    permissions: user.permissions,
                    rate_limits: user.rate_limits.clone(),
                });
            }
        }

        // Token not found
        Ok(AuthResult {
            is_authenticated: false,
            user_id: String::new(),
            permissions: Permissions::ReadOnly,
            rate_limits: RateLimits::default(),
        })
    }

    /// Check if user has permission
    pub fn has_permission(&self, user_id: &str, required: Permissions) -> bool {
        let users = self.users.read();
        if let Some(user) = users.get(user_id) {
            match (user.permissions, required) {
                (Permissions::Admin, _) => true,
                (Permissions::ReadWrite, Permissions::ReadOnly) => true,
                (Permissions::ReadWrite, Permissions::ReadWrite) => true,
                (Permissions::ReadOnly, Permissions::ReadOnly) => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

