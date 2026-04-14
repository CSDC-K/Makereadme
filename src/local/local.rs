use crate::printd;

use std::path::PathBuf;
use crate::libs::errors::Error;



pub async fn create_communication_local(
    model_type: String,
    project_dir: PathBuf,
) -> Result<bool, Error> {



    // Here you would implement the logic to execute the model locally.
    // This could involve running a subprocess, loading a local model, etc.
    // For demonstration purposes, we'll just print the information and return Ok(true).

    Ok(true)
}