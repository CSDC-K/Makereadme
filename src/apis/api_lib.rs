use crate::libs::build::ApiType;
use crate::printd;
use crate::apis::*;


pub async fn api_lib(api_type: ApiType, model_type: String, api_key: String, project_dir: String, output_file: String) {
    printd!("API Library is being integrated...", Debug);
    
    match api_type {
        ApiType::GROQ => {
            printd!("Selected API: GROQ", Success);
            groq::CreateCommunication(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
        },
        ApiType::GEMINI => {
            printd!("Selected API: GEMINI", Success);
            // CreateCommunicationGemini(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
        },
        ApiType::LLMAPI => {
            printd!("Selected API: LLMAPI", Success);
            // CreateCommunicationLLMAPI(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
        },
        ApiType::LOCAL => {
            printd!("Selected API: LOCAL", Success);
            // CreateCommunicationLocal(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
        },
        ApiType::NVIDIA => {
            printd!("Selected API: NVIDIA", Success);
            // CreateCommunicationNVIDIA(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
        }
    }
 
}

