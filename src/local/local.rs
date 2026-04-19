use crate::libs::errors::Error;
use crate::local::llama_cpp2;
use crate::libs::build::LocalBuild;
use crate::printd;


pub async fn create_communication_local(
    build_content : &LocalBuild,
) -> Result<bool, Error> {

    printd!("Initiating local model inference...", Debug);

        llama_cpp2::local_model_inference(
            build_content.model_path.clone(),
            build_content.gpu_backend.clone(),
            build_content.context_size,
            build_content.batch_size,
            build_content.threads,
            build_content.llm_model.clone(),
            build_content.project_dir.clone(),
            build_content.output_file.clone(),
            build_content.temperature,
            build_content.top_k,
            build_content.top_p
    ).await?;


    Ok(true)
}