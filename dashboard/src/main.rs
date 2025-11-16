/// Web Dashboard for PitlinkPQC
/// 
/// Provides:
/// - Real-time monitoring (transfers, network, system health)
/// - Control options (start/stop transfers, change settings)
/// - Method selection (compression, integrity, routing)
/// - Configuration management

use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use actix_files::Files;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use trackshift::*;
use std::collections::HashMap;

mod api;
mod state;

use state::DashboardState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting PitlinkPQC Dashboard...");
    println!("   Access at: http://localhost:8080");
    
    // Initialize dashboard state
    let state = Arc::new(DashboardState::new());
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/api/status", web::get().to(api::status))
            .route("/api/metrics/current", web::get().to(api::metrics_current))
            .route("/api/transfers", web::get().to(api::transfers))
            .route("/api/network", web::get().to(api::network))
            .route("/api/health", web::get().to(api::health))
            .route("/api/config", web::get().to(api::config))
            .route("/api/config", web::post().to(api::config_update))
            .route("/api/control", web::post().to(api::control))
            .route("/api/methods", web::get().to(api::methods))
            .route("/api/stats", web::get().to(api::stats))
            .service(Files::new("/", "./dashboard/static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
