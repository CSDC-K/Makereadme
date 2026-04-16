// This file contains the error definitions for the application.
// Version 0.1.0 - Initial implementation of error handling.

use thiserror::Error;


#[derive(Debug, Error)]
pub enum Error{ // GeneralError is a catch-all error type for various error scenarios in the application.
    #[error("Failed to read system_prompt.md file : {0}")]
    SystemPromptFileError(String),
    #[error("Unknown Api Type : {0}")]
    UnknownApiTypeError(String),
    #[error("Unknown LLM Model : {0}")]
    UnknownLlmModelError(String),
    #[error("Wrong API Key : {0}")]
    WrongApiKey(String),
    #[error(
r#"
Failed to read .env variables : {0}

# Allowed: GEMINI, GROQ, LLMAPI, LOCAL, NVIDIA
Example of usage in .env file:
LLM_TYPE=GEMINI
LLM_MODEL=gpt-4
API_KEY=your_api_key_here
"#
    )]
    EnvReadError(String),

    #[error("Failed to run API : {0}")]
    RunError(String),
    #[error("Rate limit exceeded (Token&Rpd-Rpm): {0}")]
    RateLimitExceededError(String),


    #[error("Failed to create response from local model : {0}")]
    LocalModelInferenceError(String),

    #[error("Failed to read file : {0}")]
    FileReadError(String),
    #[error("Failed to write file : {0}")]
    FileWriteError(String),
    #[error("Failed to execute command : {0}")]
    CommandExecutionError(String),

    #[error("Unknown error occurred : {0}")]
    UnknownError(String),

}
