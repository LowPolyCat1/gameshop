//! src/server.rs
//!
//! This module defines the Actix Web server and its routes for the IAM project.

use crate::database::Database;
use crate::middleware::AuthenticationMiddlewareFactory;
use actix_files as fs;
use actix_files::NamedFile;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::HttpRequest;
use actix_web::Result;
use actix_web::{App, HttpMessage, HttpResponse, delete, get, post, put, web};
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
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    username: String,
    #[validate(email(message = "Email is invalid"))]
    email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    password: String,
}

/// Struct representing the change username request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct ChangeUsernameRequest {
    #[validate(length(min = 3, message = "New username must be at least 3 characters long"))]
    new_username: String,
}

/// Struct representing the change password request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct ChangePasswordRequest {
    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    new_password: String,
}

/// Struct representing the create offer request body
#[derive(Debug, Deserialize, Serialize, Validate)]
struct CreateOfferRequest {
    #[validate(length(min = 3, message = "Game title is required"))]
    game_title: String,
    #[validate(length(min = 2, message = "Platform is required"))]
    platform: String,
    #[validate(length(min = 2, message = "Condition is required"))]
    condition: String,
    #[validate(range(min = 0.0, message = "Price cannot be negative"))]
    price: f64,
    #[validate(length(min = 10, message = "Description must be at least 10 characters long"))]
    description: String,
}

/// Struct representing the update offer request body
#[derive(Debug, Deserialize, Serialize)]
struct UpdateOfferRequest {
    game_title: Option<String>,
    platform: Option<String>,
    condition: Option<String>,
    price: Option<f64>,
    description: Option<String>,
}

/// Handles user login requests.
///
/// This function validates the login credentials (email and password), authenticates the user
/// against the database, and if successful, generates and returns a JWT.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection pool.
/// * `req` - JSON payload containing the user's email and password.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the login attempt.
#[post("/auth/login")]
async fn login(db: web::Data<Database>, req: web::Json<LoginRequest>) -> HttpResponse {
    if let Err(e) = req.validate() {
        tracing::warn!("Login request validation failed: {:?}", e);
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string()
        }));
    }

    match db
        .authenticate_user(req.email.clone(), req.password.clone())
        .await
    {
        Ok(user) => {
            let user_id = user.id.id.to_string(); // Extract ID from Thing
            let token = crate::jwt::generate_jwt(user_id).unwrap();
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Login successful",
                "token": token,
                "username": user.username
            }))
        }
        Err(e) => {
            tracing::warn!("Login failed: {:?}", e);
            HttpResponse::Unauthorized().json(json!({
                "success": false,
                "message": e.to_string()
            }))
        }
    }
}

/// Handles user registration requests.
///
/// This function validates the registration details, registers the new user in the database,
/// and if successful, generates and returns a JWT.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection pool.
/// * `req` - JSON payload containing the new user's details.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the registration attempt.
#[post("/auth/register")]
async fn register(db: web::Data<Database>, req: web::Json<RegisterRequest>) -> HttpResponse {
    if let Err(e) = req.validate() {
        tracing::warn!("Register request validation failed: {:?}", e);
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string()
        }));
    }

    match db
        .register(
            req.firstname.clone(),
            req.lastname.clone(),
            req.username.clone(),
            req.password.clone(),
            req.email.clone(),
        )
        .await
    {
        Ok(_) => {
            // After successful registration, log the user in to get a token
            match db
                .authenticate_user(req.email.clone(), req.password.clone())
                .await
            {
                Ok(user) => {
                    let user_id = user.id.id.to_string(); // Extract ID from Thing
                    let token = crate::jwt::generate_jwt(user_id).unwrap();
                    HttpResponse::Ok().json(json!({
                        "success": true,
                        "message": "Registration successful",
                        "token": token,
                        "username": user.username
                    }))
                }
                Err(e) => {
                    tracing::error!("Authentication failed after registration: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "message": "Registration successful but failed to log in automatically."
                    }))
                }
            }
        }
        Err(e) => {
            tracing::warn!("Registration failed: {:?}", e);
            HttpResponse::Conflict().json(json!({
                "success": false,
                "message": e.to_string()
            }))
        }
    }
}

/// Handles requests to change a user's username.
///
/// This route is protected by the `AuthenticationMiddlewareFactory`.
/// It extracts the `user_id` from the authenticated request and updates the username in the database.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `req` - HTTP request to access extensions.
/// * `body` - JSON payload containing the new username.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the username change.
#[put("/user/change-username")]
async fn change_username(
    db: web::Data<Database>,
    req: HttpRequest,
    body: web::Json<ChangeUsernameRequest>,
) -> HttpResponse {
    let user_id = match req.extensions().get::<String>() {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "User ID not found in request context."
            }));
        }
    };

    if let Err(e) = body.validate() {
        tracing::warn!("Change username request validation failed: {:?}", e);
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string()
        }));
    }

    match db.change_username(user_id, body.new_username.clone()).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Username changed successfully."
        })),
        Err(e) => {
            tracing::error!("Failed to change username: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to change username."
            }))
        }
    }
}

/// Handles requests to change a user's password.
///
/// This route is protected by the `AuthenticationMiddlewareFactory`.
/// It extracts the `user_id` from the authenticated request and updates the password in the database.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `req` - HTTP request to access extensions.
/// * `body` - JSON payload containing the new password.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the password change.
#[put("/user/change-password")]
async fn change_password(
    db: web::Data<Database>,
    req: HttpRequest,
    body: web::Json<ChangePasswordRequest>,
) -> HttpResponse {
    let user_id = match req.extensions().get::<String>() {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "User ID not found in request context."
            }));
        }
    };

    if let Err(e) = body.validate() {
        tracing::warn!("Change password request validation failed: {:?}", e);
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string()
        }));
    }

    match db.change_password(user_id, body.new_password.clone()).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Password changed successfully."
        })),
        Err(e) => {
            tracing::error!("Failed to change password: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to change password."
            }))
        }
    }
}

/// Handles requests to create a new game offer.
///
/// This route is protected by the `AuthenticationMiddlewareFactory`.
/// It extracts the `seller_id` (user_id) from the authenticated request and creates a new offer in the database.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `req` - HTTP request to access extensions.
/// * `body` - JSON payload containing the offer details.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the offer creation.
#[post("offers")]
async fn create_offer(
    db: web::Data<Database>,
    req: HttpRequest,
    body: web::Json<CreateOfferRequest>,
) -> HttpResponse {
    let seller_id = match req.extensions().get::<String>() {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Seller ID not found in request context."
            }));
        }
    };

    if let Err(e) = body.validate() {
        tracing::warn!("Create offer request validation failed: {:?}", e);
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string()
        }));
    }

    match db
        .create_offer(
            body.game_title.clone(),
            body.platform.clone(),
            body.condition.clone(),
            body.price,
            body.description.clone(),
            seller_id,
        )
        .await
    {
        Ok(offer) => HttpResponse::Created().json(json!({
            "success": true,
            "message": "Offer created successfully.",
            "offer": offer
        })),
        Err(e) => {
            tracing::error!("Failed to create offer: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to create offer."
            }))
        }
    }
}

/// Handles requests to get all game offers.
///
/// This route retrieves all existing game offers from the database.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
///
/// # Returns
///
/// An `HttpResponse` containing a list of offers or an error.
#[get("offers")]
async fn get_all_offers(db: web::Data<Database>) -> HttpResponse {
    match db.get_all_offers().await {
        Ok(offers) => HttpResponse::Ok().json(json!({
            "success": true,
            "offers": offers
        })),
        Err(e) => {
            tracing::error!("Failed to retrieve offers: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to retrieve offers."
            }))
        }
    }
}

/// Handles requests to get a single game offer by ID.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `path` - Path containing the offer ID.
///
/// # Returns
///
/// An `HttpResponse` containing the offer details or an error.
#[get("offers/{offer_id}")]
async fn get_offer_by_id(db: web::Data<Database>, path: web::Path<String>) -> HttpResponse {
    let offer_id = path.into_inner();
    match db.get_offer_by_id(offer_id).await {
        Ok(Some(offer)) => HttpResponse::Ok().json(json!({
            "success": true,
            "offer": offer
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Offer not found."
        })),
        Err(e) => {
            tracing::error!("Failed to retrieve offer: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to retrieve offer."
            }))
        }
    }
}

/// Handles requests to get all offers made by a specific seller.
///
/// This route is protected by the `AuthenticationMiddlewareFactory`.
/// It extracts the `seller_id` (user_id) from the authenticated request.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `req` - HTTP request to access extensions.
///
/// # Returns
///
/// An `HttpResponse` containing a list of offers or an error.
#[get("my-offers")]
async fn get_my_offers(db: web::Data<Database>, req: HttpRequest) -> HttpResponse {
    let seller_id = match req.extensions().get::<String>() {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Seller ID not found in request context."
            }));
        }
    };

    match db.get_offers_by_seller_id(seller_id).await {
        Ok(offers) => HttpResponse::Ok().json(json!({
            "success": true,
            "offers": offers
        })),
        Err(e) => {
            tracing::error!("Failed to retrieve user's offers: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to retrieve user's offers."
            }))
        }
    }
}

/// Handles requests to update an existing game offer.
///
/// This route is protected by the `AuthenticationMiddlewareFactory`.
/// It checks if the authenticated user is the seller of the offer before allowing the update.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `req` - HTTP request to access extensions.
/// * `path` - Path containing the offer ID.
/// * `body` - JSON payload containing the fields to update.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the offer update.
#[put("offers/{offer_id}")]
async fn update_offer(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateOfferRequest>,
) -> HttpResponse {
    let user_id = match req.extensions().get::<String>() {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "User ID not found in request context."
            }));
        }
    };
    let offer_id = path.into_inner();

    match db.get_offer_by_id(offer_id.clone()).await {
        Ok(Some(offer)) => {
            // Check if the authenticated user is the seller of this offer
            if offer.seller_id.id.to_string() != user_id {
                return HttpResponse::Forbidden().json(json!({
                    "success": false,
                    "message": "You do not have permission to update this offer."
                }));
            }

            match db
                .update_offer(
                    offer_id,
                    body.game_title.clone(),
                    body.platform.clone(),
                    body.condition.clone(),
                    body.price,
                    body.description.clone(),
                )
                .await
            {
                Ok(updated_offer) => HttpResponse::Ok().json(json!({
                    "success": true,
                    "message": "Offer updated successfully.",
                    "offer": updated_offer
                })),
                Err(e) => {
                    tracing::error!("Failed to update offer: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "message": "Failed to update offer."
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Offer not found."
        })),
        Err(e) => {
            tracing::error!("Failed to retrieve offer for update: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to retrieve offer for update."
            }))
        }
    }
}

/// Handles requests to delete an existing game offer.
///
/// This route is protected by the `AuthenticationMiddlewareFactory`.
/// It checks if the authenticated user is the seller of the offer before allowing the deletion.
///
/// # Arguments
///
/// * `db` - Web data containing the database connection.
/// * `req` - HTTP request to access extensions.
/// * `path` - Path containing the offer ID.
///
/// # Returns
///
/// An `HttpResponse` indicating the success or failure of the offer deletion.
#[delete("offers/{offer_id}")]
async fn delete_offer(
    db: web::Data<Database>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let user_id = match req.extensions().get::<String>() {
        Some(id) => id.clone(),
        None => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "User ID not found in request context."
            }));
        }
    };
    let offer_id = path.into_inner();

    match db.get_offer_by_id(offer_id.clone()).await {
        Ok(Some(offer)) => {
            // Check if the authenticated user is the seller of this offer
            if offer.seller_id.id.to_string() != user_id {
                return HttpResponse::Forbidden().json(json!({
                    "success": false,
                    "message": "You do not have permission to delete this offer."
                }));
            }

            match db.delete_offer(offer_id).await {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "success": true,
                    "message": "Offer deleted successfully."
                })),
                Err(e) => {
                    tracing::error!("Failed to delete offer: {:?}", e);
                    HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "message": "Failed to delete offer."
                    }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(json!({
            "success": false,
            "message": "Offer not found."
        })),
        Err(e) => {
            tracing::error!("Failed to retrieve offer for deletion: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Failed to retrieve offer for deletion."
            }))
        }
    }
}

/// Serves the static HTML files.
///
/// This function handles requests for static files, primarily HTML pages for the web frontend.
///
/// # Arguments
///
/// * `file` - The path to the requested file.
///
/// # Returns
///
/// A `Result` containing a `NamedFile` to be served.
#[get("")]
async fn static_files(file: web::Path<String>) -> Result<NamedFile> {
    let filename = file.into_inner();
    let path: PathBuf = if filename.is_empty() {
        "./web/index.html".into() // Default to index.html for root requests
    } else {
        format!("./web/{}", filename).into()
    };
    Ok(NamedFile::open(path)?)
}

/// Handles requests for the root path, redirecting to `index.html`.
#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("./web/index.html")?)
}

/// Configures and runs the Actix Web server.
///
/// This function sets up the logging, database connection, JWT secret, and all
/// the application routes, including static file serving and authentication routes.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the server startup.
pub async fn run_server() -> std::io::Result<()> {
    // Initialize tracing subscriber for logging
    let file_appender =
        tracing_appender::rolling::RollingFileAppender::new(Rotation::DAILY, "./logs", "iam.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(false)
        .with_writer(non_blocking_appender)
        .init();

    tracing::info!("Server starting...");

    // Create database connection
    let db = match Database::new().await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to database",
            ));
        }
    };
    let db_data = web::Data::new(db);

    // Get JWT secret from environment variable
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let jwt_secret_data = web::Data::new(jwt_secret);

    // Configure governor for rate limiting
    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(1) // Allow 2 requests per second
        .burst_size(5) // Allow a burst of 5 requests
        .finish()
        .unwrap();

    // Start the server
    actix_web::HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .app_data(jwt_secret_data.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(Governor::new(&governor_conf)) // Apply rate limiting
            .service(login)
            .service(static_files)
            .service(register)
            .service(index)
            .service(
                web::scope("api") // API routes that require authentication
                    .wrap(AuthenticationMiddlewareFactory)
                    .service(change_username)
                    .service(change_password)
                    .service(create_offer)
                    .service(get_all_offers) // You might want to make this public or controlled by roles later
                    .service(get_offer_by_id) // Same as above
                    .service(get_my_offers)
                    .service(update_offer)
                    .service(delete_offer),
            )
            // Serve static files from the "web" directory
            // This order is important: specific paths before generic
            .service(fs::Files::new("/web", "./web").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
