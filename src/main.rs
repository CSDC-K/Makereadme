pub mod debug;


use std::io::{Write, stdin, stdout};

use debug::*;           // DEBUG INFO
use colored::*;         // Cli Conf
use inquire::Select;


fn main() {
    let llm_types_vec = vec!["LOCAL", "GEMINI", "GROQ", "LLMAPI", "NVIDIA"];
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

    

    let mut llm_type = Select::new("Select LLM type:", llm_types_vec).prompt().unwrap();
    let mut model_type = match_model_type(llm_type);
    let mut api_key : String = String::new();
    let mut file_name : String = String::new();
    let mut output_name : String = String::new();

    print!("~Api Key: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut api_key).expect("ERROR AT API_KEY_INPUT");

    print!("~File Name: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut file_name).expect("ERROR AT API_KEY_INPUT");

    print!("~Output File Name: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut output_name).expect("ERROR AT API_KEY_INPUT");

    loop {
        
    }

}


fn match_model_type(llm_type : &str) -> String{
    let LLM_MODELS_GEMINI = vec!["GEMINI-2.5-FLASH-LITE","GEMINI-2.5-FLASH", "GEMINI-2.5-PRO", "GEMINI-3.0-FLASH-LITE", "GEMINI-3.0-FLASH", "GEMINI-3.0-PRO"];
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