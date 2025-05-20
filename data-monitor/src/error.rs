use std::any::Any;

use salvo::{Depot, Request, Response, Writer};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("MongoDB error: {0}")]
    MongoError(#[from] ai_flow_synth::utils::MongoError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Others error: {0}")]
    Other(#[from] anyhow::Error),

    #[error("500, Internal Server Error {0}")]
    InternalServerError(String),
}

pub type ServiceResult<T> = std::result::Result<T, ServiceError>;

#[async_trait::async_trait]
impl Writer for ServiceError {
    async fn write(self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // todo
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
