//! src/database.rs
//!
//! This module handles all database interactions for the application, using SurrealDB.

use crate::encryption::{encrypt_with_random_nonce, generate_key};
use crate::errors::custom_errors::CustomError;
use crate::hashing::{hash_random_salt, verify_password}; // Assuming hash_random_salt can be used for email hashing too, or you'd add a separate email hashing function.
use sha2::{Digest, Sha256}; // Added for email hashing

use dotenvy::var;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::process::exit;
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
    sql::{Thing, Value}, // Import Thing here
};
use uuid::Uuid;

/// Represents a user in the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// The user's ID. This is a SurrealDB Thing, representing the full record ID (e.g., 'users:uuid').
    pub id: Thing,
    /// The user's encrypted first name.
    pub encrypted_firstname: String,
    /// The user's encrypted last name.
    pub encrypted_lastname: String,
    /// The user's username.
    pub username: String,
    /// The user's password hash.
    pub password_hash: String,
    /// The user's encrypted email. This can be decrypted on the server if needed for specific operations (e.g., sending emails).
    pub encrypted_email: String,
    /// The hash of the user's email address, used for lookups and uniqueness checks.
    pub email_hash: String,
    /// The user's creation timestamp.
    pub created_at: String,
}

/// Represents a game offer in the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Offer {
    /// The offer's ID.
    pub id: Thing,
    /// The title of the game being offered.
    pub game_title: String,
    /// The platform the game is for (e.g., "PS5", "Xbox Series X", "PC").
    pub platform: String,
    /// The condition of the game (e.g., "New", "Like New", "Good", "Acceptable").
    pub condition: String,
    /// The price of the game.
    pub price: f64,
    /// A detailed description of the offer.
    pub description: String,
    /// The ID of the user who created this offer. This is a Uuid reference to the 'user' table.
    pub seller_id: Thing,
    /// The timestamp when the offer was created.
    pub created_at: String,
}

/// Represents the single database connection for all application data.
#[derive(Clone)]
pub struct Database {
    /// The SurrealDB database connection.
    pub db: Surreal<Db>,
}

impl Database {
    /// Creates a new database connection and defines schemas for both users and offers.
    ///
    /// This function initializes a connection to the SurrealDB database using the path, namespace,
    /// and database name specified in the environment variables. It defines unique indexes and
    /// schemas for both the `users` and `offers` tables in their respective namespaces.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new database connection or an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns a `CustomError` if:
    /// - The `DATABASE_PATH`, `DATABASE_NAME`, `USER_DATABASE_NAMESPACE`, or `OFFER_DB_NAMESPACE`
    /// - The connection to the database fails.
    /// - Defining any of the schemas or indexes fails.
    pub async fn new() -> Result<Self, CustomError> {
        // Get the database path from the environment variables.
        let database_path = match var("DATABASE_PATH") {
            Ok(path) => path,
            Err(error) => {
                tracing::error!("Error getting DATABASE_PATH: {}", error);
                exit(1);
            }
        };

        // Connect to the database.
        let db = Surreal::new::<RocksDb>(database_path)
            .await
            .map_err(|e| CustomError::DatabaseError(e.to_string()))?;

        // Get database name from environment variables.
        let database_name =
            var("DATABASE_NAME").map_err(|e| CustomError::DatabaseError(e.to_string()))?;

        // Use the common database name for the connection.
        db.use_db(&database_name)
            .await
            .map_err(|e| CustomError::DatabaseError(e.to_string()))?;

        // --- Define schema for 'users' table in USER_DATABASE_NAMESPACE ---
        let user_namespace = var("USER_DATABASE_NAMESPACE").map_err(|e| {
            CustomError::DatabaseError(format!("USER_DATABASE_NAMESPACE not set: {}", e))
        })?;
        db.use_ns(&user_namespace).await.map_err(|e| {
            CustomError::DatabaseError(format!("Failed to use user namespace: {}", e))
        })?;

        match db.query("DEFINE TABLE users SCHEMALESS;").await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining users table: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE INDEX users_id ON users FIELDS id UNIQUE")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining users_id index on users: {}", error);
                exit(1);
            }
        };
        // Define email_hash field and unique index
        match db
            .query("DEFINE FIELD email_hash ON users TYPE string;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining email_hash field on users: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE INDEX users_email_hash ON users FIELDS email_hash UNIQUE")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining users_email_hash index on users: {}", error);
                exit(1);
            }
        };

        // --- Define schema for 'offers' table in OFFER_DB_NAMESPACE ---
        let offer_namespace = var("OFFER_DB_NAMESPACE").map_err(|e| {
            CustomError::DatabaseError(format!("OFFER_DB_NAMESPACE not set: {}", e))
        })?;
        db.use_ns(&offer_namespace).await.map_err(|e| {
            CustomError::DatabaseError(format!("Failed to use offer namespace: {}", e))
        })?;

        match db.query("DEFINE TABLE offers SCHEMALESS;").await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining offers table: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE INDEX offers_id ON offers FIELDS id UNIQUE")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining offers_id index on offers: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE FIELD game_title ON offers TYPE string;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining game_title field on offers: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE FIELD platform ON offers TYPE string;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining platform field on offers: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE FIELD condition ON offers TYPE string;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining condition field on offers: {}", error);
                exit(1);
            }
        };
        match db.query("DEFINE FIELD price ON offers TYPE float;").await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining price field on offers: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE FIELD description ON offers TYPE string;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining description field on offers: {}", error);
                exit(1);
            }
        };
        // This defines a link to the 'user' table. Note: This link assumes 'user' is in the 'users' namespace.
        // This setup (same database, different namespaces) allows this.
        match db
            .query("DEFINE FIELD seller_id ON offers TYPE record<user>;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining seller_id field on offers: {}", error);
                exit(1);
            }
        };
        match db
            .query("DEFINE FIELD created_at ON offers TYPE datetime;")
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Error defining created_at field on offers: {}", error);
                exit(1);
            }
        };

        Ok(Database { db })
    }

    /// Helper to set the user namespace.
    async fn use_user_namespace(&self) -> Result<(), CustomError> {
        let user_namespace = var("USER_DATABASE_NAMESPACE").map_err(|e| {
            CustomError::DatabaseError(format!("USER_DATABASE_NAMESPACE not set: {}", e))
        })?;
        self.db.use_ns(&user_namespace).await.map_err(|e| {
            CustomError::DatabaseError(format!("Failed to switch to user namespace: {}", e))
        })?;
        Ok(())
    }

    /// Helper to set the offer namespace.
    async fn use_offer_namespace(&self) -> Result<(), CustomError> {
        let offer_namespace = var("OFFER_DB_NAMESPACE").map_err(|e| {
            CustomError::DatabaseError(format!("OFFER_DB_NAMESPACE not set: {}", e))
        })?;
        self.db.use_ns(&offer_namespace).await.map_err(|e| {
            CustomError::DatabaseError(format!("Failed to switch to offer namespace: {}", e))
        })?;
        Ok(())
    }

    /// Registers a new user in the database.
    ///
    /// This function takes user details as input, encrypts sensitive information, hashes the password,
    /// and stores the user data in the database.
    ///
    /// # Arguments
    ///
    /// * `firstname` - The user's first name.
    /// * `lastname` - The user's last name.
    /// * `username` - The user's username.
    /// * `password` - The user's password.
    /// * `email` - The user's email address.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns a `CustomError` if:
    /// - A user with the given email already exists.
    /// - Encryption fails.
    /// - Hashing the password fails.
    /// - Creating the user in the database fails.
    pub async fn register(
        &self,
        firstname: String,
        lastname: String,
        username: String,
        password: String,
        email: String,
    ) -> Result<bool, CustomError> {
        self.use_user_namespace().await?; // Switch to user namespace
        tracing::info!("Registering user with email: {}", email);

        // Hash the email for lookup and storage
        let email_hash = format!("{:x}", Sha256::digest(email.as_bytes()));

        let sql = "SELECT * FROM users WHERE email_hash = $email_hash";

        // Bind the parameters to the query.
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("email_hash".into(), Value::from(email_hash.as_str()));

        // Execute the query.
        let mut response = self.db.query(sql).bind(vars).await?;
        let mut users: Vec<User> = response.take(0)?;

        if let Some(_user) = users.pop() {
            tracing::warn!("User with email hash {} already exists", email_hash);
            return Err(CustomError::UserAlreadyExists);
        }

        // Generate a new UUID for the user.
        let uuid = Uuid::new_v4().to_string();
        // Generate a new encryption key.
        let key = match generate_key() {
            Ok(key) => key,
            Err(error) => {
                tracing::error!("Couldn't get key: {}", error);
                return Err(error);
            }
        };
        let key_bytes: [u8; 32] = key.into();

        // Encrypt the user's personal information.
        let encrypted_firstname = encrypt_with_random_nonce(&key_bytes, &firstname)
            .map_err(|_| CustomError::EncryptionError)?;
        let encrypted_lastname = encrypt_with_random_nonce(&key_bytes, &lastname)
            .map_err(|_| CustomError::EncryptionError)?;
        let encrypted_email = encrypt_with_random_nonce(&key_bytes, &email)
            .map_err(|_| CustomError::EncryptionError)?;

        // Hash the password.
        let password_hash = match hash_random_salt(&password) {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Error hashing password: {}", e);
                return Err(CustomError::HashingError);
            }
        };

        // Create the SQL query.
        let sql = "CREATE users SET id = $id, encrypted_firstname = $encrypted_firstname, encrypted_lastname = $encrypted_lastname, username = $username, password_hash = $password_hash, encrypted_email = $encrypted_email, email_hash = $email_hash, created_at = time::now();";

        // Bind the parameters to the query.
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("id".into(), Value::from(uuid.as_str()));
        vars.insert(
            "encrypted_firstname".into(),
            Value::from(encrypted_firstname.as_str()),
        );
        vars.insert(
            "encrypted_lastname".into(),
            Value::from(encrypted_lastname.as_str()),
        );
        vars.insert("username".into(), Value::from(username.as_str()));
        vars.insert("password_hash".into(), Value::from(password_hash.as_str()));
        vars.insert(
            "encrypted_email".into(),
            Value::from(encrypted_email.as_str()),
        );
        vars.insert("email_hash".into(), Value::from(email_hash.as_str()));

        // Execute the query.
        let created: Result<surrealdb::Response, surrealdb::Error> =
            self.db.query(sql).bind(vars).await;

        // Return the result.
        match created {
            Ok(_) => {
                tracing::info!(
                    "User registered successfully with email hash: {}",
                    email_hash
                );
                Ok(true)
            }
            Err(error) => {
                tracing::error!("Error creating user: {}", error);
                Err(CustomError::DatabaseError(error.to_string()))
            }
        }
    }

    /// Authenticates a user.
    ///
    /// This function authenticates a user by verifying the provided email and password against the
    /// stored user data in the database.
    ///
    /// # Arguments
    ///
    /// * `email` - The user's email address.
    /// * `password` - The user's password.
    ///
    /// # Returns
    ///
    /// A `Result` containing the user's data or a `CustomError` if authentication fails.
    ///
    /// # Errors
    ///
    /// Returns a `CustomError` if:
    /// - The user is not found.
    /// - The password is invalid.
    pub async fn authenticate_user(
        &self,
        email: String,
        password: String,
    ) -> Result<User, CustomError> {
        self.use_user_namespace().await?; // Switch to user namespace
        tracing::info!(
            "Authenticating user with email (hashed for lookup): {}",
            email
        );

        // Hash the incoming email for lookup
        let email_hash = format!("{:x}", Sha256::digest(email.as_bytes()));

        // Create the SQL query.
        let sql = "SELECT * FROM users WHERE email_hash = $email_hash";

        // Bind the parameters to the query.
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("email_hash".into(), Value::from(email_hash.as_str()));

        // Execute the query.
        let mut response = self.db.query(sql).bind(vars).await?;
        let mut users: Vec<User> = response.take(0)?;

        if let Some(user) = users.pop() {
            if verify_password(&password, &user.password_hash).is_ok() {
                tracing::info!(
                    "User authenticated successfully with email hash: {}",
                    email_hash
                );
                Ok(user)
            } else {
                tracing::warn!("Invalid password for user with email hash: {}", email_hash);
                Err(CustomError::InvalidPassword)
            }
        } else {
            tracing::warn!("User not found with email hash: {}", email_hash);
            Err(CustomError::UserNotFound)
        }
    }

    /// Changes the username of a user.
    ///
    /// This function updates the username of an existing user in the database.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to update.
    /// * `new_username` - The new username.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns a `CustomError` if:
    /// - The update operation fails.
    pub async fn change_username(
        &self,
        user_id: String,
        new_username: String,
    ) -> Result<(), CustomError> {
        self.use_user_namespace().await?; // Switch to user namespace
        // Create the SQL query.
        let sql = "UPDATE users SET username = $new_username WHERE id = $user_id;";

        // Bind the parameters to the query.
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("user_id".into(), Value::from(user_id.as_str()));
        vars.insert("new_username".into(), Value::from(new_username.as_str()));

        // Execute the query.
        self.db.query(sql).bind(vars).await?;
        Ok(())
    }

    /// Changes the password of a user.
    ///
    /// This function updates the password of an existing user in the database.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to update.
    /// * `new_password` - The new password.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns a `CustomError` if:
    /// - The update operation fails.
    pub async fn change_password(
        &self,
        user_id: String,
        new_password: String,
    ) -> Result<(), CustomError> {
        self.use_user_namespace().await?; // Switch to user namespace
        // Hash the new password.
        let password_hash = match hash_random_salt(&new_password) {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Error hashing new password: {}", e);
                return Err(CustomError::HashingError);
            }
        };

        // Create the SQL query.
        let sql = "UPDATE users SET password_hash = $password_hash WHERE id = $user_id;";

        // Bind the parameters to the query.
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("user_id".into(), Value::from(user_id.as_str()));
        vars.insert("password_hash".into(), Value::from(password_hash.as_str()));

        // Execute the query.
        self.db.query(sql).bind(vars).await?;
        Ok(())
    }

    /// Creates a new game offer in the database.
    ///
    /// # Arguments
    ///
    /// * `game_title` - The title of the game.
    /// * `platform` - The platform of the game.
    /// * `condition` - The condition of the game.
    /// * `price` - The price of the game.
    /// * `description` - The description of the offer.
    /// * `seller_id` - The ID of the user selling the game.
    ///
    /// # Returns
    ///
    /// A `Result` containing the created `Offer` or a `CustomError` if creation fails.
    pub async fn create_offer(
        &self,
        game_title: String,
        platform: String,
        condition: String,
        price: f64,
        description: String,
        seller_id: String, // This is the UUID string
    ) -> Result<Offer, CustomError> {
        self.use_offer_namespace().await?; // Switch to offer namespace
        tracing::info!("Creating offer for game: {}", game_title);

        let offer_id = Uuid::new_v4().to_string();

        // Construct the Thing for seller_id explicitly, e.g., 'user:your-uuid'
        let seller_id_thing = Thing::from(("user".to_string(), seller_id.clone()));

        let sql = "CREATE offers SET id = $id, game_title = $game_title, platform = $platform, condition = $condition, price = $price, description = $description, seller_id = $seller_id_thing, created_at = time::now();";

        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("id".into(), Value::from(offer_id.as_str()));
        vars.insert("game_title".into(), Value::from(game_title.as_str()));
        vars.insert("platform".into(), Value::from(platform.as_str()));
        vars.insert("condition".into(), Value::from(condition.as_str()));
        vars.insert("price".into(), Value::from(price));
        vars.insert("description".into(), Value::from(description.as_str()));
        // Bind the constructed Thing for seller_id
        vars.insert("seller_id_thing".into(), Value::from(seller_id_thing));

        let mut response: surrealdb::Response = self.db.query(sql).bind(vars).await?;
        let created_offer: Option<Offer> = response.take(0)?;

        created_offer.ok_or_else(|| {
            tracing::error!("Failed to retrieve created offer after insertion.");
            CustomError::DatabaseError("Failed to retrieve created offer".to_string())
        })
    }

    /// Retrieves all offers from the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Offer` structs or a `CustomError` if retrieval fails.
    pub async fn get_all_offers(&self) -> Result<Vec<Offer>, CustomError> {
        self.use_offer_namespace().await?; // Switch to offer namespace
        tracing::info!("Retrieving all offers.");
        let sql = "SELECT * FROM offers ORDER BY created_at DESC;";
        let mut response: surrealdb::Response = self.db.query(sql).await?;
        let offers: Vec<Offer> = response.take(0)?;
        Ok(offers)
    }

    /// Retrieves a single offer by its ID.
    ///
    /// # Arguments
    ///
    /// * `offer_id` - The ID of the offer to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option` of the `Offer` struct or a `CustomError` if retrieval fails.
    pub async fn get_offer_by_id(&self, offer_id: String) -> Result<Option<Offer>, CustomError> {
        self.use_offer_namespace().await?; // Switch to offer namespace
        tracing::info!("Retrieving offer with ID: {}", offer_id);
        let sql = "SELECT * FROM offers WHERE id = $offer_id;";
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("offer_id".into(), Value::from(offer_id.as_str()));

        let mut response: surrealdb::Response = self.db.query(sql).bind(vars).await?;
        let offer: Option<Offer> = response.take(0)?;
        Ok(offer)
    }

    /// Retrieves all offers made by a specific seller.
    ///
    /// # Arguments
    ///
    /// * `seller_id` - The ID of the seller.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Offer` structs or a `CustomError` if retrieval fails.
    pub async fn get_offers_by_seller_id(
        &self,
        seller_id: String,
    ) -> Result<Vec<Offer>, CustomError> {
        self.use_offer_namespace().await?; // Switch to offer namespace
        tracing::info!("Retrieving offers for seller ID: {}", seller_id);
        // Correctly form the record link for the WHERE clause
        let seller_id_thing = Thing::from(("user".to_string(), seller_id));
        let sql =
            "SELECT * FROM offers WHERE seller_id = $seller_id_thing ORDER BY created_at DESC;";
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("seller_id_thing".into(), Value::from(seller_id_thing));

        let mut response: surrealdb::Response = self.db.query(sql).bind(vars).await?;
        let offers: Vec<Offer> = response.take(0)?;
        Ok(offers)
    }

    /// Updates an existing offer in the database.
    ///
    /// # Arguments
    ///
    /// * `offer_id` - The ID of the offer to update.
    /// * `game_title` - The new game title (optional).
    /// * `platform` - The new platform (optional).
    /// * `condition` - The new condition (optional).
    /// * `price` - The new price (optional).
    /// * `description` - The new description (optional).
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated `Offer` or a `CustomError` if update fails.
    pub async fn update_offer(
        &self,
        offer_id: String,
        game_title: Option<String>,
        platform: Option<String>,
        condition: Option<String>,
        price: Option<f64>,
        description: Option<String>,
    ) -> Result<Offer, CustomError> {
        self.use_offer_namespace().await?; // Switch to offer namespace
        tracing::info!("Updating offer with ID: {}", offer_id);
        let mut updates = Vec::new();
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("offer_id".into(), Value::from(offer_id.as_str()));

        if let Some(gt) = game_title {
            updates.push("game_title = $game_title".to_string());
            vars.insert("game_title".into(), Value::from(gt));
        }
        if let Some(p) = platform {
            updates.push("platform = $platform".to_string());
            vars.insert("platform".into(), Value::from(p));
        }
        if let Some(c) = condition {
            updates.push("condition = $condition".to_string());
            vars.insert("condition".into(), Value::from(c));
        }
        if let Some(pr) = price {
            updates.push("price = $price".to_string());
            vars.insert("price".into(), Value::from(pr));
        }
        if let Some(d) = description {
            updates.push("description = $description".to_string());
            vars.insert("description".into(), Value::from(d));
        }

        if updates.is_empty() {
            tracing::warn!("No fields provided for update for offer ID: {}", offer_id);
            return Err(CustomError::DatabaseError(
                "No fields to update".to_string(),
            ));
        }

        let sql = format!(
            "UPDATE offers SET {} WHERE id = $offer_id RETURN *;",
            updates.join(", ")
        );

        let mut response: surrealdb::Response = self.db.query(sql).bind(vars).await?;
        let updated_offer: Option<Offer> = response.take(0)?;

        updated_offer.ok_or_else(|| {
            tracing::error!("Failed to retrieve updated offer for ID: {}", offer_id);
            CustomError::DatabaseError("Failed to update or retrieve offer".to_string())
        })
    }

    /// Deletes an offer from the database.
    ///
    /// # Arguments
    ///
    /// * `offer_id` - The ID of the offer to delete.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub async fn delete_offer(&self, offer_id: String) -> Result<(), CustomError> {
        self.use_offer_namespace().await?; // Switch to offer namespace
        tracing::info!("Deleting offer with ID: {}", offer_id);
        let sql = "DELETE offers WHERE id = $offer_id;";
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("offer_id".into(), Value::from(offer_id.as_str()));

        self.db.query(sql).bind(vars).await?;
        Ok(())
    }
}
