use std::any::Any;

use salvo::{Response, Writer, http::StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("MongoDB error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Others error: {0}")]
    Other(#[from] anyhow::Error),

    #[error("400, Bad Request {0}")]
    BadRequest(String),

    #[error("404, Not Found {0}")]
    NotFound(String),

    #[error("500, Internal Server Error {0}")]
    InternalServerError(String),
}

pub type ServiceResult<T> = std::result::Result<T, ServiceError>;

#[async_trait::async_trait]
impl Writer for ServiceError {
    async fn write(
        mut self,
        _req: &mut salvo::Request,
        _depot: &mut salvo::Depot,
        res: &mut Response,
    ) {
        match self {
            ServiceError::BadRequest(msg) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(msg);
            }
            ServiceError::NotFound(msg) => {
                res.status_code(StatusCode::NOT_FOUND);
                res.render(msg);
            }
            ServiceError::InternalServerError(msg) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(msg);
            }
            ServiceError::MongoError(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("MongoDB error: {}", err));
            }
            ServiceError::IoError(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("IO error: {}", err));
            }
            ServiceError::Other(err) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(format!("Other error: {}", err));
            } // _ => {
              //     res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
              //     res.render("Internal Server Error");
              // }
        }
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
