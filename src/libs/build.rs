use std::path::PathBuf;

use crate::printd;
use crate::apis::api_lib::{api_lib};
use crate::libs::errors::Error;

fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }

    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 6 {
        return "*".repeat(chars.len());
    }

    let prefix: String = chars.iter().take(3).collect();
    let suffix: String = chars.iter().rev().take(3).rev().collect();
    let middle_mask = "*".repeat(chars.len() - 6);
    format!("{}{}{}", prefix, middle_mask, suffix)
}

pub enum ApiType {
    GEMINI,
    GROQ,
    LLMAPI,
    LOCAL,
    NVIDIA,
}


pub struct Build {
    API_TYPE : String,
    LLM_MODEL : String,
    API_KEY : String,
    PROJECT_DIR : PathBuf,
    OUTPUT_FILE : String
}

pub struct LocalBuild {
    LLM_MODEL : String,
    PROJECT_DIR : PathBuf,
    OUTPUT_FILE : String
}

impl LocalBuild {
    pub fn new(llm_model : String, project_dir : PathBuf, output_file : String) -> Self {
        LocalBuild {
            LLM_MODEL : llm_model,
            PROJECT_DIR : project_dir,
            OUTPUT_FILE : output_file
        }
    }

    pub async fn build(&self) -> Result<bool, Error> {
        printd!("Building process started!", Success);
        printd!("Reading configs...", Debug);
        printd!(format!("MODEL TYPE : {}", self.LLM_MODEL).as_str(), Debug);
        printd!(format!("PROJECT DIR : {}", self.PROJECT_DIR.to_str().unwrap()).as_str(), Debug);
        printd!(format!("OUTPUT FILE : {}", self.OUTPUT_FILE).as_str(), Debug);
        printd!("Starting Ai Service...", Debug);

        // CreateCommunicationLocal(self.LLM_MODEL.clone(), self.PROJECT_DIR.clone()).await;

        Ok(true)
    }
}

impl Build {
    pub fn new(api_type : String, llm_model : String, api_key : String, project_dir : PathBuf, output_file : String) -> Self {
        Build {
            API_TYPE : api_type,
            LLM_MODEL : llm_model,
            API_KEY : api_key,
            PROJECT_DIR : project_dir,
            OUTPUT_FILE : output_file
        }
    }

    pub async fn build(&self) -> Result<bool, Error> {
        printd!("Building process started!", Success);
        printd!("Reading configs...", Debug);
        printd!(format!("API TYPE : {}", self.API_TYPE).as_str(), Debug);
        printd!(format!("MODEL TYPE : {}", self.LLM_MODEL).as_str(), Debug);
        printd!(format!("API KEY : {}", mask_secret(self.API_KEY.as_str())).as_str(), Debug);
        printd!(format!("PROJECT DIR : {}", self.PROJECT_DIR.to_str().unwrap()).as_str(), Debug);
        printd!(format!("OUTPUT FILE : {}", self.OUTPUT_FILE).as_str(), Debug);
        printd!("Starting Ai Service...", Debug);

        let api_type_enum = match self.API_TYPE.as_str() {
            "GEMINI" => ApiType::GEMINI,
            "GROQ" => ApiType::GROQ,
            "LLMAPI" => ApiType::LLMAPI,
            "LOCAL" => ApiType::LOCAL,
            "NVIDIA" => ApiType::NVIDIA,
            _ => {
                printd!("Invalid API Type!", Failed);
                return Err(Error::UnknownApiTypeError(self.API_TYPE.clone()));
            }
        };

        match api_lib(
            api_type_enum,
            self.LLM_MODEL.clone(),
            self.API_KEY.clone(),
            &self.PROJECT_DIR,
            self.OUTPUT_FILE.clone(),
        )
        .await
        {
            Ok(exit_received) => Ok(exit_received),
            Err(e) => Err(e)
        }

    }

}