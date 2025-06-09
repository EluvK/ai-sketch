use std::any::Any;

use salvo::{
    Scribe,
    http::{StatusCode, StatusError},
    oapi::{self, EndpointOutRegister, ToSchema},
};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("400, Bad Request {0}")]
    BadRequest(String),
    #[error("401, Unauthorized {0}")]
    Unauthorized(String),
    #[error("400, Duplicate User {0}")]
    DuplicateUser(String),
    #[error("500, Internal Server Error {0}")]
    InternalServerError(String),

    #[error("MongoDB error: {0}")]
    MongoClientError(#[from] mongodb::error::Error),
    #[error("Bson De error: {0}")]
    BsonDeError(#[from] bson::de::Error),
    #[error("Bson Ser error: {0}")]
    BsonSerError(#[from] bson::ser::Error),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

pub type ServiceResult<T> = std::result::Result<T, ServiceError>;

impl Scribe for ServiceError {
    fn render(self, res: &mut salvo::Response) {
        match self {
            ServiceError::BadRequest(msg) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(msg);
            }
            ServiceError::DuplicateUser(msg) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(format!("Duplicate user: {}", msg));
            }
            ServiceError::InternalServerError(msg) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(msg);
            }
            ServiceError::Unauthorized(msg) => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(format!("Unauthorized: {}", msg));
            }

            ServiceError::MongoClientError(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("MongoDB error: {}", err));
            }
            ServiceError::BsonDeError(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("BSON error: {}", err));
            }
            ServiceError::BsonSerError(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("BSON error: {}", err));
            }
            ServiceError::JwtError(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("JWT error: {}", err));
            }
        }
    }
}

impl EndpointOutRegister for ServiceError {
    fn register(components: &mut oapi::Components, operation: &mut oapi::Operation) {
        operation.responses.insert(
            StatusCode::BAD_REQUEST.as_str(),
            oapi::Response::new("Bad request")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::UNAUTHORIZED.as_str(),
            oapi::Response::new("Unauthorized")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::NOT_FOUND.as_str(),
            oapi::Response::new("Not found")
                .add_content("application/json", StatusError::to_schema(components)),
        );
        operation.responses.insert(
            StatusCode::INTERNAL_SERVER_ERROR.as_str(),
            oapi::Response::new("Internal server error")
                .add_content("application/json", StatusError::to_schema(components)),
        );
    }
}

// for depot.get/obtain
impl From<Option<&Box<dyn Any + Send + Sync>>> for ServiceError {
    fn from(value: Option<&Box<dyn Any + Send + Sync>>) -> Self {
        ServiceError::InternalServerError(
            value
                .and_then(|v| v.downcast_ref::<String>())
                .map(|s| s.clone())
                .unwrap_or_else(|| "Unknown error".to_string()),
        )
    }
}

impl From<salvo::http::ParseError> for ServiceError {
    fn from(err: salvo::http::ParseError) -> Self {
        ServiceError::BadRequest(err.to_string())
    }
}
