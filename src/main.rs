pub mod libs {
    pub mod debug;
    pub mod build;
    pub mod action_executer;
}

pub mod apis {
    pub mod api_lib;
    pub mod groq;
    pub mod gemini;
    pub mod llmapi;
    pub mod nvidia;
}

use std::{io::{Write, stdin, stdout}, vec};

use libs::debug::*;           // DEBUG INFO
use libs::build::Build;      // Build Struct
use colored::*;         // Cli Conf
use inquire::Select;
use tokio::main;


#[tokio::main]
async fn main() {
    let api_types_vec = vec!["LOCAL", "GEMINI", "GROQ", "LLMAPI", "NVIDIA"];
    let banner : String = r#"
  _____  ______          _____  __  __ ______   __  __          _  ________ _____  
 |  __ \|  ____|   /\   |  __ \|  \/  |  ____| |  \/  |   /\   | |/ /  ____|  __ \ 
 | |__) | |__     /  \  | |  | | \  / | |__    | \  / |  /  \  | ' /| |__  | |__) |
 |  _  /|  __|   / /\ \ | |  | | |\/| |  __|   | |\/| | / /\ \ |  < |  __| |  _  / 
 | | \ \| |____ / ____ \| |__| | |  | | |____  | |  | |/ ____ \| . \| |____| | \ \ 
 |_|  \_\______/_/    \_\_____/|_|  |_|______| |_|  |_/_/    \_\_|\_\______|_|  \_\
                                                                                   
                                                                    -- Made By Kuzey
    "#.to_string();

    println!("{}", banner.bright_yellow());

    println!("Welcome to Readme Maker V 0.0.1");
    

    let mut api_type = Select::new("Select Api type:", api_types_vec).prompt().unwrap();
    let mut model_type = match_model_type(api_type);
    let mut api_key : String = String::new();
    let mut project_folder : String = String::new();
    let mut output_name : String = String::new();

    print!("~Api Key: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut api_key).expect("ERROR AT API_KEY_INPUT");

    print!("~Project Folder: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut project_folder).expect("ERROR AT PROJECT_FOLDER_INPUT");

    print!("~Output File Name: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut output_name).expect("ERROR AT OUTPUT_INPUT");

    loop {
        printd!("Reading configs...", Debug);
        printd!(format!("LLM API : {}", api_type).as_str(), Debug);
        printd!(format!("MODEL TYPE : {}", model_type).as_str(), Debug);
        printd!(format!("API KEY : {}", api_key.trim()).as_str(), Debug);
        printd!(format!("PROJECT DIR : {}", project_folder.trim()).as_str(), Debug);
        printd!(format!("OUTPUT FILE : {}", output_name.trim()).as_str(), Debug);
        print!("Is that build true? (Y/N) ");
        stdout().flush().unwrap();
        let mut y_n = String::new();
        stdin().read_line(&mut y_n).expect("ERROR AT BUILD_Y_N_INPUT");

        let y_n_input = y_n.trim().chars().next();

        let what_user_wants_to_change = match y_n_input {
            Some('N') | Some('n') => {
                Select::new("Which input you wanting to change?", vec!["API TYPE", "MODEL TYPE", "API KEY", "PROJECT FOLDER", "OUTPUT NAME"]).prompt().unwrap()
            },
            Some('Y') | Some('y') => { "" }

            _ => {
                printd!("Unkown input! Please answer correctly.", Failed);
                continue
            } ,
        };

        if (what_user_wants_to_change == "") {
            let build = Build::new(
                api_type.to_string(),
                model_type.clone(),
                api_key.trim().to_string(),
                std::path::PathBuf::from(project_folder.trim()),
                output_name.trim().to_string()
            );
            build.build().await;
            
        }

    }

}

fn match_model_type(llm_type : &str) -> String{
    let LLM_MODELS_GEMINI = vec!["gemini-2.5-flash-lite","gemini-2.5-flash", "gemini-2.5-pro", "gemini-3.0-flash-lite", "gemini-3.0-flash", "gemini-3.0-pro"];
    let LLM_MODELS_GROQ = vec!["llama-3.1-8b-instant", "llama-3.3-70b-versatile", "meta-llama/llama-4-scout-17b-16e-instruct", "openai/gpt-oss-120b", "openai/gpt-oss-20b", "qwen/qwen3-32b"];
    

    let mut model_type = match llm_type {
        "LOCAL" => {
            let mut LOCAL_MODEL : String = String::new();
            print!("$~ LOCAL MODEL PATH: ");
            stdout().flush().unwrap();
            stdin().read_line(&mut LOCAL_MODEL).expect("ERROR AT LOCAL_MODEL_PATH_INPUT");
            LOCAL_MODEL.to_string()
        },
        "GEMINI" => {
            Select::new("Select Model:", LLM_MODELS_GEMINI).prompt().unwrap().to_string()
        },
        "GROQ" => {
            Select::new("Select Model:", LLM_MODELS_GROQ).prompt().unwrap().to_string()
        },

       "NVIDIA" => {
            let mut NVIDIA_MODEL : String = String::new();
            print!("$~ NVIDIA MODEL : ");
            stdout().flush().unwrap();
            stdin().read_line(&mut NVIDIA_MODEL).expect("ERROR AT NVIDIA_MODEL_INPUT");
            NVIDIA_MODEL.to_string()
        },

        _ => {
            printd!("Model selection process failed because of match did not get any llm_type", Failed);
            panic!();
        }
    };

    model_type
}