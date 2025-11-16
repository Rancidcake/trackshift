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
            .service(web::resource("/api/status").route(web::get().to(api::status)))
            .service(web::resource("/api/metrics/current").route(web::get().to(api::metrics_current)))
            .service(web::resource("/api/transfers").route(web::get().to(api::transfers)))
            .service(web::resource("/api/network").route(web::get().to(api::network)))
            .service(web::resource("/api/health").route(web::get().to(api::health)))
            .service(web::resource("/api/config").route(web::get().to(api::config)).route(web::post().to(api::config_update)))
            .service(web::resource("/api/control").route(web::post().to(api::control)))
            .service(web::resource("/api/methods").route(web::get().to(api::methods)))
            .service(web::resource("/api/stats").route(web::get().to(api::stats)))
            .service(Files::new("/", "./dashboard/static").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
