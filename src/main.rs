//! src/main.rs
//!
//! This is the main entry point for the gameshop project.

use gameshop::server::run_server;

#[tokio::main]
/// Starts the application.
///
/// # Returns
///
/// A `Result` indicating success or failure.
async fn main() {
    let _ = run_server().await;
}
