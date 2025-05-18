#[derive(thiserror::Error, Debug)]
pub enum LLMError {
    #[error("LLMError Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("LLMError Stream: {0}")]
    ReqwestEventSource(#[from] reqwest_eventsource::CannotCloneRequestError),

    #[error("LLMError Serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("LLMError Provider: {0}")]
    LLMProvider(String),
}

pub type LLMResult<T> = Result<T, LLMError>;
