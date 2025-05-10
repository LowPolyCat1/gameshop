//! src/main.rs
//!
//! This is the main entry point for the gameshop project.

use gameshop::server::start;

#[tokio::main]
/// Starts the application.
///
/// # Returns
///
/// A `Result` indicating success or failure.
async fn main() {
    let _ = start().await;
}
