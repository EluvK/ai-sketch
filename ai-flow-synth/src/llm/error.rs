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

    #[error("LLMError Tool: {0}")]
    Tool(String),

    #[error("LLMError SteamSendError: {0}")]
    StreamSendError(
        #[from]
        tokio::sync::broadcast::error::SendError<crate::core::stream_message::StreamMessage>,
    ),
}

pub type LLMResult<T> = Result<T, LLMError>;
