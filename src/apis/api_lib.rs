use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use crate::libs::build::ApiType;
use crate::printd;
use crate::apis::*;
use crate::libs::errors::Error;


pub async fn api_lib(api_type: ApiType, model_type: String, api_key: String, project_dir: &PathBuf, output_file: String) -> Result<bool, Error> {
    printd!("API Library is being integrated...", Debug);
    
    let mut prompt_file = File::open("system_prompt.md")
        .map_err(|e| Error::SystemPromptFileError(e.to_string()))?;
    let mut contents = String::new();
    prompt_file
        .read_to_string(&mut contents)
        .map_err(|e| Error::SystemPromptFileError(e.to_string()))?;

    match api_type {
        ApiType::GROQ => {
            printd!("Selected API: GROQ", Success);
            groq::create_communication(api_key, contents, model_type).await
        },
        ApiType::GEMINI => {
            printd!("Selected API: GEMINI", Success);
            match gemini::create_communication(
                api_key,
                contents,
                model_type,
                project_dir,
                output_file.as_str(),
            ).await {
                Ok(exit_received) => {
                    printd!("Gemini communication completed", Success);
                    Ok(exit_received)
                }
                Err(e) => Err(e)
            }
        },
        ApiType::LLMAPI => {
            printd!("Selected API: LLMAPI", Success);
            match llmapi::create_communication(
                api_key,
                contents,
                model_type,
                project_dir,
                output_file.as_str(),
            ).await {
                Ok(exit_received) => {
                    printd!("LLMAPI communication completed", Success);
                    Ok(exit_received)
                }
                Err(e) => Err(e)
            }
        },
        ApiType::LOCAL => {
            printd!("Selected API: LOCAL", Success);
            // CreateCommunicationLocal(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
            Ok(false)
        },
        ApiType::NVIDIA => {
            printd!("Selected API: NVIDIA", Success);
            // CreateCommunicationNVIDIA(api_key, "You are a helpful assistant that helps to create readme files.".to_string(), model_type).await;
            Ok(false)
        }
    }
 
}

