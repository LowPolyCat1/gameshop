//! src/server.rs
//!
//! This module defines the Actix Web server and its routes for the IAM project.

use crate::database::Database;
use crate::errors::custom_errors::CustomError;
use crate::middleware::AuthenticationMiddlewareFactory;
use actix_files as fs;
use actix_files::NamedFile;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::HttpRequest;
use actix_web::Result;
use actix_web::{App, HttpMessage, HttpResponse, Responder, delete, get, post, put, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env::var;
use std::path::PathBuf;
use tracing_appender::rolling::Rotation;
use validator::Validate;
use validator_derive::Validate;

/// Struct representing the login request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct LoginRequest {
    #[validate(email(message = "Email is invalid"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    password: String,
}
/// Struct representing the register request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct RegisterRequest {
    #[validate(length(min = 1, message = "Firstname is required"))]
    firstname: String,
    #[validate(length(min = 1, message = "Lastname is required"))]
    lastname: String,
    #[validate(length(min = 1, message = "Username is required"))]
    username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    password: String,
    #[validate(email(message = "Email is invalid"))]
    email: String,
}

/// Struct representing the change username request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct ChangeUsernameRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    username: String,
}

/// Struct representing the change password request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct ChangePasswordRequest {
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    password: String,
}

/// Struct representing the create offer request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct CreateOfferRequest {
    #[validate(length(min = 1, message = "Game title is required"))]
    game_title: String,
    #[validate(length(min = 1, message = "Platform is required"))]
    platform: String,
    #[validate(length(min = 1, message = "Condition is required"))]
    condition: String,
    #[validate(range(min = 0.0, message = "Price cannot be negative"))]
    price: f64,
    #[validate(length(min = 1, message = "Description is required"))]
    description: String,
}

/// Struct representing the update offer request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct UpdateOfferRequest {
    #[validate(
        length(min = 1, message = "Game title cannot be empty"),
    )]
    game_title: Option<String>,
    #[validate(
        length(min = 1, message = "Platform cannot be empty"),
    )]
    platform: Option<String>,
    #[validate(
        length(min = 1, message = "Condition cannot be empty"),
    )]
    condition: Option<String>,
    #[validate(range(min = 0.0, message = "Price cannot be negative"))]
    price: Option<f64>,
    #[validate(
        length(min = 1, message = "Description cannot be empty"),
    )]
    description: Option<String>,
}

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    /// Database connection
    pub db: Database,
}

/// Starts the Actix Web server.
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    let rolling = tracing_appender::rolling::Builder::new()
        .rotation(Rotation::DAILY)
        .filename_suffix("log")
        .build("D:/VSC/Rust/Projects/current/IAM/logs")?; // Only here until I get logging middleware to work
    tracing_subscriber::fmt().with_writer(rolling).init();
    tracing::info!("Starting Programm!");

    tracing::info!("Loading env");
    // Load environment variables from .env file
    load_dotenv()?;

    // Create a new database connection
    let database = Database::new().await?;

    // Create the application state
    let app_state = AppState {
        db: database.clone(),
    };

    tracing::info!("Getting IP");
    // Get the server IP address from environment variables
    let server_ip = get_server_ip()?;

    tracing::info!("Getting Port");
    // Get the server port as a string from environment variables
    let server_port_string = get_server_port_string()?;

    tracing::info!("Parsing Port");
    // Parse the server port string into a u16
    let server_port = parse_server_port(&server_port_string)?;
    tracing::info!("Setting up server");

    let governor_conf = match GovernorConfigBuilder::default()
        .requests_per_second(2)
        .seconds_per_request(1)
        .burst_size(10)
        .finish()
    {
        Some(governor) => governor,
        None => {
            return Err(Box::new(CustomError::GovernorCreationError(
                "Unknown".to_string(),
            )));
        }
    };

    // Create the Actix Web server
    actix_web::HttpServer::new(move || {
        App::new()
            // Share the application state with all routes
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Governor::new(&governor_conf))
            .wrap(AuthenticationMiddlewareFactory::new())
            .service(register)
            .service(login)
            .service(change_username)
            .service(change_password)
            .service(create_offer)
            .service(get_all_offers)
            .service(get_offer_by_id)
            .service(get_offers_by_seller_id)
            .service(update_offer)
            .service(delete_offer)
            .service(index)
            .service(fs::Files::new("/web", "./web").show_files_listing())
    })
    // Bind the server to the specified IP address and port
    .bind((server_ip, server_port))?
    .run()
    .await?;

    tracing::info!("Starting server");
    // Start the server
    Ok(())
}

/// Gets the server IP address from environment variables.
///
/// # Returns
///
/// A `Result` containing the server IP address or a `CustomError` if an error occurs.
fn get_server_ip() -> Result<String, CustomError> {
    match var("SERVER_IP") {
        Ok(server_ip) => {
            tracing::info!("Found SERVER_IP = {}", server_ip);
            Ok(server_ip)
        }
        Err(error) => {
            tracing::error!("Couldn't find SERVER_IP | {}", error);
            Err(CustomError::EnvironmentVariableError(error.to_string()))
        }
    }
}

/// Gets the server port as a string from environment variables.
///
/// # Returns
///
/// A `Result` containing the server port as a string or a `CustomError` if an error occurs.
fn get_server_port_string() -> Result<String, CustomError> {
    match var("SERVER_PORT") {
        Ok(server_port) => {
            tracing::info!("Found SERVER_PORT = {}", server_port);
            Ok(server_port)
        }
        Err(error) => {
            tracing::error!("Couldn't find SERVER_PORT | {}", error);
            Err(CustomError::EnvironmentVariableError(error.to_string()))
        }
    }
}

/// Loads environment variables from the .env file.
///
/// # Returns
///
/// A `Result` containing the server IP address or a `CustomError` if an error occurs.
fn load_dotenv() -> Result<(), CustomError> {
    match dotenvy::dotenv() {
        Ok(pathbuf) => {
            tracing::info!("loaded .env file: {:?}", pathbuf);
            Ok(())
        }
        Err(error) => {
            tracing::error!("Couldn't load env | {}", error);
            Err(CustomError::from(error))
        }
    }
}

/// Parses the server port string into a u16.
///
/// # Arguments
///
/// * `server_port_string` - The server port as a string.
///
/// # Returns
///
/// A `Result` containing the server port as a u16 or a `CustomError` if an error occurs.
fn parse_server_port(server_port_string: &str) -> Result<u16, CustomError> {
    match server_port_string.parse::<u16>() {
        Ok(port) => {
            tracing::info!("Successfully parsed port: {}", port);
            Ok(port)
        }
        Err(error) => {
            tracing::error!("Error parsing port | {}", error);
            // Remove unwrap() and propagate the error
            Err(CustomError::ParsingServerPortError(error.to_string()))
        }
    }
}

/// Registers a new user.
///
/// # Arguments
///
/// * `req` - The register request.
/// * `data` - The application state.
///
/// # Returns
///
/// A `Result` indicating success or failure.
#[post("/auth/register")]
async fn register(
    req: web::Json<RegisterRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    tracing::info!("Registering user");
    tracing::info!("{:?}", req.0);
    let user_id = http_req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    tracing::info!("User ID from token: {}", user_id);

    // Validate the request body
    if let Err(validation_errors) = req.0.validate() {
        tracing::warn!("Validation error: {:?}", validation_errors);
        return HttpResponse::BadRequest().json(validation_errors);
    }

    // Extract the request body
    let firstname = req.0.firstname.clone();
    let lastname = req.0.lastname.clone();
    let username = req.0.username.clone();
    let password = req.0.password.clone();
    let email = req.0.email.clone().to_lowercase();

    // Get the database connection from the application state
    let db = &data.db;

    // hashing is handled in the db.register function
    match db
        .register(firstname, lastname, username, password, email)
        .await
    {
        Ok(registered) => {
            if registered {
                // Generate a JWT for the new user. The `register` function returns a boolean,
                // so we need to fetch the user again to get their ID for JWT generation.
                // This is a simplification; ideally, `register` would return the User struct or its ID.
                // For now, we'll assume a successful registration implies we can attempt to log them in.
                // A more robust solution would involve the register function returning the user's ID.
                match db
                    .authenticate_user(req.0.email.clone().to_lowercase(), req.0.password.clone())
                    .await
                {
                    Ok(user) => match crate::jwt::generate_jwt(user.id.to_string()) {
                        Ok(token) => {
                            HttpResponse::Created().json(json!({ "success": true, "token": token }))
                        }
                        Err(_) => HttpResponse::InternalServerError()
                            .json(json!({ "success": false, "error": "JWT generation failed" })),
                    },
                    Err(e) => {
                        tracing::error!("Failed to authenticate user after registration: {}", e);
                        HttpResponse::InternalServerError()
                            .json(json!({ "success": false, "error": "Failed to log in after registration" }))
                    }
                }
            } else {
                HttpResponse::InternalServerError()
                    .json(json!({ "success": false, "error": "User registration failed" }))
            }
        }
        Err(err) => {
            HttpResponse::Conflict().json(json!({ "success": false, "message": err.to_string() }))
        }
    }
}

/// Authenticates a user.
///
/// # Arguments
///
/// * `req` - The login request.
///
/// # Returns
///
/// A `Result` indicating success or failure.
#[post("/auth/login")]
async fn login(req: web::Json<LoginRequest>, data: web::Data<AppState>) -> impl Responder {
    tracing::info!("Authenticating user");
    // Validate the request body
    if let Err(validation_errors) = req.0.validate() {
        tracing::warn!("Validation error: {:?}", validation_errors);
        return HttpResponse::BadRequest().json(validation_errors);
    }

    // Extract the request body
    let email = req.0.email.clone().to_lowercase();
    let password = req.0.password.clone();

    // Get the database connection from the application state
    let db = &data.db;

    // Authenticate the user
    match db.authenticate_user(email, password).await {
        Ok(user) => {
            tracing::info!("User authenticated successfully");
            // Generate JWT
            match crate::jwt::generate_jwt(user.id.to_string()) {
                Ok(token) => HttpResponse::Ok()
                    .json(json!({"success": true, "token": token, "username": user.username})),
                Err(error) => {
                    tracing::error!("Error generating JWT: {}", error);
                    HttpResponse::InternalServerError()
                        .json(json!({"success": false, "error": "Failed to generate token"}))
                }
            }
        }
        Err(error) => {
            tracing::error!("Error authenticating user: {}", error);
            match error {
                CustomError::InvalidPassword => HttpResponse::Ok()
                    .json(json!({"success": false, "message": "Invalid password"})),
                CustomError::UserNotFound => {
                    HttpResponse::Ok().json(json!({"success": false, "message": "User not found"}))
                }
                _ => HttpResponse::InternalServerError()
                    .json(json!({"success": false, "message": "An internal error occurred"})),
            }
        }
    }
}

/// Changes the Username of a user.
///
/// # Arguments
///
/// * `req` - The change username request.
///
/// * `http_req` - The http request.
///
/// # Returns
///
/// A `Result` indicating success or failure.
#[post("/api/change_username")]
async fn change_username(
    http_req: HttpRequest,
    req: web::Json<ChangeUsernameRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Validate the request body
    if let Err(validation_errors) = req.0.validate() {
        tracing::warn!("Validation error: {:?}", validation_errors);
        return HttpResponse::BadRequest().json(validation_errors);
    }
    let new_username = req.0.username;
    let user_id = http_req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    match data.db.change_username(user_id, new_username).await {
        Ok(_) => HttpResponse::Ok()
            .json(json!({"success": true, "message": "Successfully changed username"})),
        Err(error) => {
            tracing::error!("Error changing username: {}", error);
            HttpResponse::InternalServerError()
                .json(json!({"success": false, "message": "Failed to change username"}))
        }
    }
}

/// Changes the Password of a user.
///
/// # Arguments
///
/// * `req` - The change username request.
///
/// * `http_req` - The http request.
///
/// # Returns
///
/// A `Result` indicating success or failure.
#[post("/api/change_password")]
async fn change_password(
    http_req: HttpRequest,
    req: web::Json<ChangePasswordRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Validate the request body
    if let Err(validation_errors) = req.0.validate() {
        tracing::warn!("Validation error: {:?}", validation_errors);
        return HttpResponse::BadRequest().json(validation_errors);
    }
    let new_password = req.0.password;
    let user_id = http_req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    match data.db.change_password(user_id, new_password).await {
        Ok(_) => HttpResponse::Ok()
            .json(json!({"success": true, "message": "Successfully changed password"})),
        Err(error) => {
            tracing::error!("Error changing password: {}", error);
            HttpResponse::InternalServerError()
                .json(json!({"success": false, "message": "Failed to change password"}))
        }
    }
}

/// Creates a new game offer.
///
/// # Arguments
///
/// * `req` - The create offer request.
/// * `data` - The application state.
/// * `http_req` - The HTTP request (to get user ID from token).
///
/// # Returns
///
/// An `HttpResponse` indicating success or failure.
#[post("/api/offers")]
async fn create_offer(
    req: web::Json<CreateOfferRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    tracing::info!("Creating new offer");

    // Validate the request body
    if let Err(validation_errors) = req.0.validate() {
        tracing::warn!("Validation error: {:?}", validation_errors);
        return HttpResponse::BadRequest().json(validation_errors);
    }

    // Extract seller_id from the authenticated user's token
    let seller_id = http_req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    if seller_id == "Unknown" {
        tracing::warn!("Unauthorized attempt to create offer: Missing user ID in token.");
        return HttpResponse::Unauthorized()
            .json(json!({"success": false, "message": "Authentication required"}));
    }

    let game_title = req.0.game_title.clone();
    let platform = req.0.platform.clone();
    let condition = req.0.condition.clone();
    let price = req.0.price;
    let description = req.0.description.clone();

    match data
        .db
        .create_offer(
            game_title,
            platform,
            condition,
            price,
            description,
            seller_id,
        )
        .await
    {
        Ok(offer) => {
            tracing::info!("Offer created successfully: {:?}", offer.id);
            HttpResponse::Created().json(json!({"success": true, "message": "Offer created successfully", "offer_id": offer.id}))
        }
        Err(error) => {
            tracing::error!("Error creating offer: {}", error);
            HttpResponse::InternalServerError().json(
                json!({"success": false, "message": format!("Failed to create offer: {}", error)}),
            )
        }
    }
}

/// Retrieves all game offers.
///
/// # Arguments
///
/// * `data` - The application state.
///
/// # Returns
///
/// An `HttpResponse` containing a list of offers or an error.
#[get("/api/offers")]
async fn get_all_offers(data: web::Data<AppState>) -> impl Responder {
    tracing::info!("Retrieving all offers");
    match data.db.get_all_offers().await {
        Ok(offers) => {
            tracing::info!("Successfully retrieved {} offers", offers.len());
            HttpResponse::Ok().json(json!({"success": true, "offers": offers}))
        }
        Err(error) => {
            tracing::error!("Error retrieving all offers: {}", error);
            HttpResponse::InternalServerError().json(json!({"success": false, "message": format!("Failed to retrieve offers: {}", error)}))
        }
    }
}

/// Retrieves a single game offer by its ID.
///
/// # Arguments
///
/// * `path` - Path containing the offer ID.
/// * `data` - The application state.
///
/// # Returns
///
/// An `HttpResponse` containing the offer or an error.
#[get("/api/offers/{id}")]
async fn get_offer_by_id(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let offer_id = path.into_inner();
    tracing::info!("Retrieving offer with ID: {}", offer_id);
    match data.db.get_offer_by_id(offer_id.clone()).await {
        Ok(Some(offer)) => {
            tracing::info!("Successfully retrieved offer: {:?}", offer.id);
            HttpResponse::Ok().json(json!({"success": true, "offer": offer}))
        }
        Ok(None) => {
            tracing::warn!("Offer with ID {} not found", offer_id);
            HttpResponse::NotFound().json(json!({"success": false, "message": "Offer not found"}))
        }
        Err(error) => {
            tracing::error!("Error retrieving offer {}: {}", offer_id, error);
            HttpResponse::InternalServerError().json(json!({"success": false, "message": format!("Failed to retrieve offer: {}", error)}))
        }
    }
}

/// Retrieves all offers made by a specific seller.
///
/// # Arguments
///
/// * `path` - Path containing the seller ID.
/// * `data` - The application state.
///
/// # Returns
///
/// An `HttpResponse` containing a list of offers or an error.
#[get("/api/offers/seller/{seller_id}")]
async fn get_offers_by_seller_id(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let seller_id = path.into_inner();
    tracing::info!("Retrieving offers for seller ID: {}", seller_id);
    match data.db.get_offers_by_seller_id(seller_id.clone()).await {
        Ok(offers) => {
            tracing::info!(
                "Successfully retrieved {} offers for seller {}",
                offers.len(),
                seller_id
            );
            HttpResponse::Ok().json(json!({"success": true, "offers": offers}))
        }
        Err(error) => {
            tracing::error!(
                "Error retrieving offers for seller {}: {}",
                seller_id,
                error
            );
            HttpResponse::InternalServerError().json(json!({"success": false, "message": format!("Failed to retrieve offers: {}", error)}))
        }
    }
}

/// Updates an existing game offer.
///
/// # Arguments
///
/// * `path` - Path containing the offer ID.
/// * `req` - The update offer request.
/// * `data` - The application state.
/// * `http_req` - The HTTP request (to get user ID from token).
///
/// # Returns
///
/// An `HttpResponse` indicating success or failure.
#[put("/api/offers/{id}")]
async fn update_offer(
    path: web::Path<String>,
    req: web::Json<UpdateOfferRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let offer_id = path.into_inner();
    tracing::info!("Updating offer with ID: {}", offer_id);

    // Validate the request body
    if let Err(validation_errors) = req.0.validate() {
        tracing::warn!("Validation error: {:?}", validation_errors);
        return HttpResponse::BadRequest().json(validation_errors);
    }

    // Extract user_id from the authenticated user's token
    let user_id_from_token = http_req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    if user_id_from_token == "Unknown" {
        tracing::warn!(
            "Unauthorized attempt to update offer {}: Missing user ID in token.",
            offer_id
        );
        return HttpResponse::Unauthorized()
            .json(json!({"success": false, "message": "Authentication required"}));
    }

    // Verify ownership: Get the offer first to check its seller_id
    match data.db.get_offer_by_id(offer_id.clone()).await {
        Ok(Some(existing_offer)) => {
            if existing_offer.seller_id.id.to_string() != user_id_from_token {
                tracing::warn!(
                    "Unauthorized attempt to update offer {}: User {} does not own this offer.",
                    offer_id,
                    user_id_from_token
                );
                return HttpResponse::Forbidden().json(json!({"success": false, "message": "You are not authorized to update this offer"}));
            }
        }
        Ok(None) => {
            tracing::warn!("Attempted to update non-existent offer: {}", offer_id);
            return HttpResponse::NotFound()
                .json(json!({"success": false, "message": "Offer not found"}));
        }
        Err(error) => {
            tracing::error!("Error checking ownership for offer {}: {}", offer_id, error);
            return HttpResponse::InternalServerError().json(json!({"success": false, "message": format!("Failed to verify offer ownership: {}", error)}));
        }
    }

    let game_title = req.0.game_title.clone();
    let platform = req.0.platform.clone();
    let condition = req.0.condition.clone();
    let price = req.0.price;
    let description = req.0.description.clone();

    match data
        .db
        .update_offer(
            offer_id.clone(),
            game_title,
            platform,
            condition,
            price,
            description,
        )
        .await
    {
        Ok(offer) => {
            tracing::info!("Offer updated successfully: {:?}", offer.id);
            HttpResponse::Ok().json(
                json!({"success": true, "message": "Offer updated successfully", "offer": offer}),
            )
        }
        Err(error) => {
            tracing::error!("Error updating offer {}: {}", offer_id, error);
            HttpResponse::InternalServerError().json(
                json!({"success": false, "message": format!("Failed to update offer: {}", error)}),
            )
        }
    }
}

/// Deletes a game offer.
///
/// # Arguments
///
/// * `path` - Path containing the offer ID.
/// * `data` - The application state.
/// * `http_req` - The HTTP request (to get user ID from token).
///
/// # Returns
///
/// An `HttpResponse` indicating success or failure.
#[delete("/api/offers/{id}")]
async fn delete_offer(
    path: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let offer_id = path.into_inner();
    tracing::info!("Deleting offer with ID: {}", offer_id);

    // Extract user_id from the authenticated user's token
    let user_id_from_token = http_req
        .extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    if user_id_from_token == "Unknown" {
        tracing::warn!(
            "Unauthorized attempt to delete offer {}: Missing user ID in token.",
            offer_id
        );
        return HttpResponse::Unauthorized()
            .json(json!({"success": false, "message": "Authentication required"}));
    }

    // Verify ownership: Get the offer first to check its seller_id
    match data.db.get_offer_by_id(offer_id.clone()).await {
        Ok(Some(existing_offer)) => {
            if existing_offer.seller_id.id.to_string() != user_id_from_token {
                tracing::warn!(
                    "Unauthorized attempt to delete offer {}: User {} does not own this offer.",
                    offer_id,
                    user_id_from_token
                );
                return HttpResponse::Forbidden().json(json!({"success": false, "message": "You are not authorized to delete this offer"}));
            }
        }
        Ok(None) => {
            tracing::warn!("Attempted to delete non-existent offer: {}", offer_id);
            return HttpResponse::NotFound()
                .json(json!({"success": false, "message": "Offer not found"}));
        }
        Err(error) => {
            tracing::error!("Error checking ownership for offer {}: {}", offer_id, error);
            return HttpResponse::InternalServerError().json(json!({"success": false, "message": format!("Failed to verify offer ownership: {}", error)}));
        }
    }

    match data.db.delete_offer(offer_id.clone()).await {
        Ok(_) => {
            tracing::info!("Offer {} deleted successfully", offer_id);
            HttpResponse::Ok()
                .json(json!({"success": true, "message": "Offer deleted successfully"}))
        }
        Err(error) => {
            tracing::error!("Error deleting offer {}: {}", offer_id, error);
            HttpResponse::InternalServerError().json(
                json!({"success": false, "message": format!("Failed to delete offer: {}", error)}),
            )
        }
    }
}

#[get("/")]
async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "web/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
