use std::path::PathBuf;

use crate::libs::errors::Error;
use crate::local::ollama;
use crate::libs::build::LocalBuild;
use crate::printd;


pub async fn create_communication_local(
    build_content : &LocalBuild,
) -> Result<bool, Error> {

    printd!("Initiating local model inference...", Debug);

    ollama::local_model_inference(
            build_content.ollama_gateway_url.clone(),
            build_content.llm_model.clone(),
            build_content.project_dir.clone(),
            build_content.output_file.clone(),
            build_content.temperature,
            build_content.top_k,
            build_content.top_p
    ).await?;


    Ok(true)
}