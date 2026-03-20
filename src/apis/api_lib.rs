use std::path::PathBuf;
use std::fs::File;
use std::io::{self, Read};

use crate::libs::build::ApiType;
use crate::printd;
use crate::apis::*;



pub async fn api_lib(api_type: ApiType, model_type: String, api_key: String, project_dir: &PathBuf, output_file: String) {
    printd!("API Library is being integrated...", Debug);
    
    let mut prompt_file = File::open("system_prompt.md").expect("ERROR AT PROMPT_FILE_OPEN");
    let mut contents = String::new();
    prompt_file.read_to_string(&mut contents).expect("ERROR AT PROMPT_FILE_READ");

    match api_type {
        ApiType::GROQ => {
            printd!("Selected API: GROQ", Success);
            groq::create_communication(api_key, contents, model_type).await;
        },
        ApiType::GEMINI => {
            printd!("Selected API: GEMINI", Success);
            match gemini::create_communication(
                api_key,
                contents,
                model_type,
                project_dir
            ).await {
                Ok(_) => printd!("Gemini communication completed", Success),
                Err(e) => printd!(format!("Gemini communication failed: {}", e).as_str(), Failed),
            }
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

