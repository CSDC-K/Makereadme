pub mod libs {   
    pub mod debug;
    pub mod build;
    pub mod action_executer;
    pub mod memory;
}

pub mod apis {
    pub mod api_lib;
    pub mod groq;
    pub mod gemini;
    pub mod llmapi;
    pub mod nvidia;
}

use std::{io::{Write, stdin, stdout}, vec};
use std::env;
use std::fs;

use libs::debug::*;           // DEBUG INFO
use libs::build::Build;      // Build Struct
use colored::*;         // Cli Conf
use dotenv::dotenv;
use inquire::Select;


#[tokio::main]
async fn main() {
    let api_types_vec = vec!["LOCAL", "GEMINI", "GROQ", "LLMAPI", "NVIDIA"];
    let banner : String = r#"
▗▖  ▗▖ ▗▄▖ ▗▖ ▗▖▗▄▄▄▖▗▄▄▖ ▗▄▄▄▖ ▗▄▖ ▗▄▄▄  ▗▖  ▗▖▗▄▄▄▖
▐▛▚▞▜▌▐▌ ▐▌▐▌▗▞▘▐▌   ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌  █ ▐▛▚▞▜▌▐▌   
▐▌  ▐▌▐▛▀▜▌▐▛▚▖ ▐▛▀▀▘▐▛▀▚▖▐▛▀▀▘▐▛▀▜▌▐▌  █ ▐▌  ▐▌▐▛▀▀▘
▐▌  ▐▌▐▌ ▐▌▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▀ ▐▌  ▐▌▐▙▄▄▖
                                   -- Made By Kuzey (CSDC-K)
    "#.to_string();

    println!("{}", banner.bright_yellow());

    println!("Welcome to MakeREADME V 0.0.1");
    

    let mut load_env = String::new();
    print!("Do you want to load .env file? (Y/N) ");
    stdout().flush().unwrap();
    stdin().read_line(&mut load_env).expect("ERROR AT LOAD_ENV_INPUT");


    let load_env_yn = load_env.trim().chars().next();

    let load_env_file = match load_env_yn {
        Some('N') | Some('n') => {
            printd!("Starting without .env file. You will be asked to input all the values.", Debug);
            "0"
        },
        Some('Y') | Some('y') => { "1" }

        _ => {
            printd!("Unkown input! Starting with defaults.", Failed);
            ""
        } ,
    };


    if load_env_file == "1" {
        dotenv().ok();
        printd!("Loaded .env file successfully!", Success);
    }

    let mut api_type: String;
    let mut model_type: String;
    let mut api_key : String;
    let mut project_folder : String = String::new();
    let mut output_name : String = String::new();

    if load_env_file == "1" {
        api_type = read_env_key("LLM_TYPE").unwrap_or_default();
        model_type = read_env_key("LLM_MODEL").unwrap_or_default();
        api_key = read_env_key("API_KEY").unwrap_or_default();

        if api_type.is_empty() || !api_types_vec.contains(&api_type.as_str()) {
            if !api_type.is_empty() {
                printd!(
                    format!("Invalid LLM_TYPE in .env: '{}'. Asking interactively.", api_type).as_str(),
                    Failed
                );
            }
            api_type = Select::new("Select Api type:", api_types_vec.clone())
                .prompt()
                .unwrap()
                .to_string();
        }

        if model_type.is_empty() {
            printd!("LLM_MODEL was not found in .env, asking interactively.", Debug);
            model_type = match_model_type(api_type.as_str());
        }

        if api_key.is_empty() {
            printd!("API_KEY was not found in .env, asking interactively.", Debug);
            api_key = read_line_input("~Api Key: ", "ERROR AT API_KEY_INPUT");
        }

        printd!("Loaded llm type, model, and api key from .env", Success);
    } else {
        api_type = Select::new("Select Api type:", api_types_vec.clone())
            .prompt()
            .unwrap()
            .to_string();
        model_type = match_model_type(api_type.as_str());
        api_key = read_line_input("~Api Key: ", "ERROR AT API_KEY_INPUT");
    }

    print!("~Project Folder: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut project_folder).expect("ERROR AT PROJECT_FOLDER_INPUT");

    print!("~Output File Name: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut output_name).expect("ERROR AT OUTPUT_INPUT");

    loop {
        printd!("Reading configs...", Debug);
        printd!(format!("LLM API : {}", api_type.as_str()).as_str(), Debug);
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

        if what_user_wants_to_change == "" {
            let build = Build::new(
                api_type.clone(),
                model_type.trim().to_string(),
                api_key.trim().to_string(),
                std::path::PathBuf::from(project_folder.trim()),
                output_name.trim().to_string()
            );
            let exit_received = build.build().await;

            if exit_received && ask_yes_no("Model EXIT aksiyonu verdi. Ayarlari .env dosyasina kaydetmek ister misin? (Y/N) ") {
                match save_llm_settings_to_env(api_type.as_str(), api_key.trim(), model_type.trim()) {
                    Ok(_) => printd!("Settings saved to .env", Success),
                    Err(e) => printd!(format!("Failed to save .env: {}", e).as_str(), Failed),
                }
            }

            continue;
        }

        match what_user_wants_to_change {
            "API TYPE" => {
                api_type = Select::new("Select Api type:", api_types_vec.clone())
                    .prompt()
                    .unwrap()
                    .to_string();
                model_type = match_model_type(api_type.as_str());
            }
            "MODEL TYPE" => {
                model_type = match_model_type(api_type.as_str());
            }
            "API KEY" => {
                print!("~Api Key: ");
                stdout().flush().unwrap();
                api_key.clear();
                stdin()
                    .read_line(&mut api_key)
                    .expect("ERROR AT API_KEY_INPUT");
            }
            "PROJECT FOLDER" => {
                print!("~Project Folder: ");
                stdout().flush().unwrap();
                project_folder.clear();
                stdin()
                    .read_line(&mut project_folder)
                    .expect("ERROR AT PROJECT_FOLDER_INPUT");
            }
            "OUTPUT NAME" => {
                print!("~Output File Name: ");
                stdout().flush().unwrap();
                output_name.clear();
                stdin()
                    .read_line(&mut output_name)
                    .expect("ERROR AT OUTPUT_INPUT");
            }
            _ => {
                printd!("Unknown selection. Keeping current values.", Failed);
            }
        }

    }

}

fn ask_yes_no(prompt: &str) -> bool {
    print!("{}", prompt);
    stdout().flush().unwrap();

    let mut answer = String::new();
    stdin().read_line(&mut answer).expect("ERROR AT YES_NO_INPUT");

    matches!(answer.trim().chars().next(), Some('Y') | Some('y'))
}

fn read_line_input(prompt: &str, error_message: &str) -> String {
    let mut value = String::new();
    print!("{}", prompt);
    stdout().flush().unwrap();
    stdin().read_line(&mut value).expect(error_message);
    value.trim().to_string()
}

fn read_env_key(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn format_env_value(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':'))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn save_llm_settings_to_env(llm_type: &str, api_key: &str, llm_model: &str) -> std::io::Result<()> {
    let env_path = ".env";
    let mut lines: Vec<String> = match fs::read_to_string(env_path) {
        Ok(content) => content.lines().map(|l| l.to_string()).collect(),
        Err(_) => Vec::new(),
    };

    let desired = [
        ("LLM_TYPE", llm_type),
        ("API_KEY", api_key),
        ("LLM_MODEL", llm_model),
    ];

    for (key, value) in desired {
        let mut replaced = false;
        for line in &mut lines {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || !trimmed.contains('=') {
                continue;
            }

            let existing_key = trimmed.splitn(2, '=').next().unwrap().trim();
            if existing_key == key {
                *line = format!("{}={}", key, format_env_value(value));
                replaced = true;
                break;
            }
        }

        if !replaced {
            lines.push(format!("{}={}", key, format_env_value(value)));
        }
    }

    let mut content = lines.join("\n");
    content.push('\n');
    fs::write(env_path, content)
}

fn match_model_type(llm_type : &str) -> String{
    let llm_models_gemini = vec!["gemini-2.5-flash-lite","gemini-2.5-flash", "gemini-2.5-pro", "gemini-3.0-flash-lite", "gemini-3.0-flash", "gemini-3.0-pro"];
    let llm_models_groq = vec!["llama-3.1-8b-instant", "llama-3.3-70b-versatile", "meta-llama/llama-4-scout-17b-16e-instruct", "openai/gpt-oss-120b", "openai/gpt-oss-20b", "qwen/qwen3-32b"];
    

    let model_type = match llm_type {
        "LOCAL" => {
            let mut local_model : String = String::new();
            print!("$~ LOCAL MODEL PATH: ");
            stdout().flush().unwrap();
            stdin().read_line(&mut local_model).expect("ERROR AT LOCAL_MODEL_PATH_INPUT");
            local_model.trim().to_string()
        },
        "GEMINI" => {
            Select::new("Select Model:", llm_models_gemini).prompt().unwrap().to_string()
        },
        "GROQ" => {
            Select::new("Select Model:", llm_models_groq).prompt().unwrap().to_string()
        },

       "NVIDIA" => {
            let mut nvidia_model : String = String::new();
            print!("$~ NVIDIA MODEL : ");
            stdout().flush().unwrap();
            stdin().read_line(&mut nvidia_model).expect("ERROR AT NVIDIA_MODEL_INPUT");
            nvidia_model.trim().to_string()
        },

        "LLMAPI" => {
            let mut llmapi_model : String = String::new();
            print!("$~ LLMAPI MODEL : ");
            stdout().flush().unwrap();
            stdin().read_line(&mut llmapi_model).expect("ERROR AT LLMAPI_MODEL_INPUT");
            llmapi_model.trim().to_string()
        },

        _ => {
            printd!("Model selection process failed because of match did not get any llm_type", Failed);
            panic!();
        }
    };

    model_type
}