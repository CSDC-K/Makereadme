use std::path::PathBuf;

use crate::libs::build::ApiType;
use crate::printd;
use crate::apis::*;
use crate::libs::errors::Error;
use crate::libs::prompt::Prompt;


pub async fn api_lib(api_type: ApiType, model_type: String, api_key: String, project_dir: &PathBuf, output_file: String) -> Result<bool, Error> {
    printd!("API Library is being integrated...", Debug);
    
    let system_prompt = Prompt::default().content;

    match api_type {
        ApiType::GROQ => {
            printd!("Selected API: GROQ", Success);
            groq::create_communication(
                api_key,
                system_prompt,
                model_type,
                project_dir,
                output_file.as_str(),
            ).await
        },
        ApiType::GEMINI => {
            printd!("Selected API: GEMINI", Success);
            match gemini::create_communication(
                api_key,
                system_prompt,
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
                system_prompt,
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
            // CreateCommunicationLocal(api_key, system_prompt, model_type).await;
            Ok(false)
        },
        ApiType::NVIDIA => {
            printd!("Selected API: NVIDIA", Success);
            // CreateCommunicationNVIDIA(api_key, system_prompt, model_type).await;
            Ok(false)
        }
    }
 
}

