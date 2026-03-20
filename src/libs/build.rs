use std::path::PathBuf;

use crate::printd;
use crate::apis::api_lib::{self, api_lib};

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

    pub async fn build(&self) {
        printd!("Building process started!", Success);
        printd!("Reading configs...", Debug);
        printd!(format!("API TYPE : {}", self.API_TYPE).as_str(), Debug);
        printd!(format!("MODEL TYPE : {}", self.LLM_MODEL).as_str(), Debug);
        printd!(format!("API KEY : {}", self.API_KEY).as_str(), Debug);
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
                printd!("Invalid API Type! Defaulting to LOCAL", Failed);
                ApiType::LOCAL
            }
        };

        let api = api_lib(api_type_enum, self.LLM_MODEL.clone(), self.API_KEY.clone(), &self.PROJECT_DIR, self.OUTPUT_FILE.clone()).await;

    }

}