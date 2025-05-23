//! src/middleware.rs
//!
//! This module provides authentication middleware for Actix Web applications.

use crate::jwt::{extract_user_id_from_jwt, validate_jwt};
use actix_web::dev::Transform;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, forward_ready},
    error::ErrorUnauthorized,
    http::Method,
};
use futures::future::err;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use tracing::info;

/// Authentication middleware that checks for a valid JWT in the request header.
pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    /// Creates a new `AuthenticationMiddleware` instance.
    ///
    /// # Arguments
    ///
    /// * `service` - The service to wrap with authentication.
    pub fn new(service: Rc<S>) -> Self {
        AuthenticationMiddleware { service }
    }
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    /// Processes the service request and performs authentication.
    ///
    /// # Arguments
    ///
    /// * `req` - The service request to process.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for OPTIONS requests or specific routes
        if *req.method() == Method::OPTIONS
            || req.path() == "/"
            || req.path().starts_with("/web/")
            || req.path().starts_with("/auth/")
        {
            return Box::pin(self.service.call(req));
        }

        let auth_header = req.headers().get("Authorization");
        let auth_header = match auth_header {
            Some(header) => header,
            None => {
                tracing::error!("Missing authorization header");
                return Box::pin(err(ErrorUnauthorized("Missing authorization header")));
            }
        };

        let auth_value = match auth_header.to_str() {
            Ok(value) => value,
            Err(_) => {
                tracing::error!("Invalid authorization header value");
                return Box::pin(err(ErrorUnauthorized("Invalid authorization header value")));
            }
        };

        let token = match auth_value.strip_prefix("Bearer ") {
            Some(token) => token.trim(),
            None => {
                tracing::error!("Invalid authorization format");
                return Box::pin(err(ErrorUnauthorized("Invalid authorization format")));
            }
        };

        match validate_jwt(token) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Invalid token: {}", e);
                return Box::pin(err(ErrorUnauthorized("Invalid token")));
            }
        };

        let user_id = match extract_user_id_from_jwt(token) {
            Ok(user_id) => user_id,
            Err(e) => {
                tracing::error!("Failed to extract user ID: {}", e);
                return Box::pin(err(ErrorUnauthorized("Invalid token")));
            }
        };

        info!("Authenticated user with ID: {}", user_id);
        req.extensions_mut().insert(user_id.clone()); // Store user_id in extensions
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

/// Factory for creating `AuthenticationMiddleware` instances.
#[derive(Default)]
pub struct AuthenticationMiddlewareFactory;

impl AuthenticationMiddlewareFactory {
    /// Creates a new `AuthenticationMiddlewareFactory` instance.
    pub fn new() -> Self {
        AuthenticationMiddlewareFactory
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticationMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates a new `AuthenticationMiddleware` instance for each service.
    ///
    /// # Arguments
    ///
    /// * `service` - The service to wrap with authentication.
    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuthenticationMiddleware::new(Rc::new(service))))
    }
}
