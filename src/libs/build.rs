use std::path::PathBuf;

use crate::printd;

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

    pub fn build(&self) {
        printd!("Building process started!", Success);
        printd!("Reading configs...", Debug);
        printd!(format!("API TYPE : {}", self.API_TYPE).as_str(), Debug);
        printd!(format!("MODEL TYPE : {}", self.LLM_MODEL).as_str(), Debug);
        printd!(format!("API KEY : {}", self.API_KEY).as_str(), Debug);
        printd!(format!("PROJECT DIR : {}", self.PROJECT_DIR.to_str().unwrap()).as_str(), Debug);
        printd!(format!("OUTPUT FILE : {}", self.OUTPUT_FILE).as_str(), Debug);
        printd!("Starting Ai Service...", Debug);
    }

}