// Main entry point for the MakeREADME application, responsible for initializing the application, handling user input, and orchestrating the overall flow of the program.
// Version 0.1.0 - Initial implementation of the main application logic.
pub mod libs {   
    pub mod debug;             // Debugging utilities for the application, providing functions to print debug information in a structured and colored format.
    pub mod build;             // Build utilities for the application, providing functions to manage and execute build processes.
    pub mod action_executer;   // Action execution utilities for the application, providing functions to execute various actions.
    pub mod memory;            // Memory management utilities for the application, providing functions to manage memory allocation and deallocation.
    pub mod errors;            // Error handling utilities for the application, providing functions to handle and report errors.
    pub mod prompt;            // Prompt management utilities for the application, providing functions to create and manage prompts.
}

pub mod apis {
    pub mod api_lib;            // API library for the application, providing functions to integrate and communicate with various APIs.
    pub mod groq;               // GROQ API integration module, providing functions to communicate with the GROQ API.
    pub mod gemini;             // Gemini API integration module, providing functions to communicate with the Gemini API.
    pub mod llmapi;             // LLM API integration module, providing functions to communicate with various LLM APIs.
    pub mod nvidia;             // NVIDIA API integration module, providing functions to communicate with NVIDIA's APIs.
}

pub mod local {
    pub mod local;              // Local execution module, providing functions to execute processes locally without API integration.
    pub mod ollama;             // Ollama integration module, providing functions to communicate with the Ollama API for local model execution.
}

use std::{io::{Write, stdin, stdout}, vec}; // Standard library imports for input/output operations and vector handling.
use std::env;                               // Standard library import for environment variable handling.
use std::fs;                                // Standard library import for file system operations.

use libs::debug::*;             // DEBUG INFO
use libs::build::*    ;         // Build Struct
use colored::*;                 // Colored Terminal Output
use libs::errors::Error;        // Error Handling
use dotenv::dotenv;             // ENV
use inquire::Select;            // Interactive CLI Selections
use serde::{Deserialize};

use crate::libs::build;       // Serialization/Deserialization for data handling


#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    name: String,
}



#[tokio::main]
async fn main() {

    if let Err(e) = run().await {
        printd!(format!("{e}").as_str(), Failed); // thiserror Display mesajını basar
        std::process::exit(1);
    }

}

async fn run() -> Result<(), Error> {
    let api_types_vec = vec!["LOCAL", "GEMINI", "GROQ", "LLMAPI", "NVIDIA"];
    let banner : String = r#"
▗▖  ▗▖ ▗▄▖ ▗▖ ▗▖▗▄▄▄▖▗▄▄▖ ▗▄▄▄▖ ▗▄▖ ▗▄▄▄  ▗▖  ▗▖▗▄▄▄▖
▐▛▚▞▜▌▐▌ ▐▌▐▌▗▞▘▐▌   ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌  █ ▐▛▚▞▜▌▐▌   
▐▌  ▐▌▐▛▀▜▌▐▛▚▖ ▐▛▀▀▘▐▛▀▚▖▐▛▀▀▘▐▛▀▜▌▐▌  █ ▐▌  ▐▌▐▛▀▀▘
▐▌  ▐▌▐▌ ▐▌▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▀ ▐▌  ▐▌▐▙▄▄▖

-- Generate README.md files with the power of LLMs! --
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
        api_type = read_env_key("LLM_TYPE").ok_or(Error::EnvReadError("LLM_TYPE VARIABLE NOT FOUND".to_string()))?;
        model_type = read_env_key("LLM_MODEL").ok_or(Error::EnvReadError("LLM_MODEL VARIABLE NOT FOUND".to_string()))?;
        api_key = read_env_key("API_KEY").ok_or(Error::EnvReadError("API_KEY VARIABLE NOT FOUND".to_string()))?;

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

        if api_type == "LOCAL" {
            let ollama_yn = ask_yes_no("Is ollama installed? (Y/N) ");

            if ollama_yn {
                // asking api gateway url and port for local execution

                let mut url = String::new();
                let mut model_temperature = String::new();
                let mut model_top_k = String::new();
                let mut model_top_p = String::new();


                print!("~Ollama API URL (default http://127.0.0.1:11434): ");
                stdout().flush().unwrap();
                stdin().read_line(&mut url).expect("ERROR AT OLLAMA_URL_INPUT");
                let url = if url.trim().is_empty() {
                    "http://127.0.0.1:11434"
                } else {
                    url.trim()
                };

                print!("~Model Temperature (default 0.7): ");
                stdout().flush().unwrap();
                stdin().read_line(&mut model_temperature).expect("ERROR AT MODEL_TEMPERATURE_INPUT");
                


                print!("~Model Top K (default 20): ");
                stdout().flush().unwrap();
                stdin().read_line(&mut model_top_k).expect("ERROR AT MODEL_TOP_K_INPUT");

                print!("~Model Top P (default 0.95): ");
                stdout().flush().unwrap();
                stdin().read_line(&mut model_top_p).expect("ERROR AT MODEL_TOP_P_INPUT");


                match get_ollama_models(url) {
                    Ok(models) => {
                        if models.is_empty() {
                            printd!("No models found in Ollama. Please add models to Ollama and try again.", Failed);
                            std::process::exit(1);
                        }
                        model_type = Select::new("Select Model:", models).prompt().unwrap();
                        api_key = "NONE".to_string();

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
                            printd!(format!("PROJECT DIR : {}", project_folder.trim()).as_str(), Debug);
                            printd!(format!("OUTPUT FILE : {}", output_name.trim()).as_str(), Debug);
                            print!("Is that build true? (Y/N) ");
                            stdout().flush().unwrap();
                            let mut y_n = String::new();
                            stdin().read_line(&mut y_n).expect("ERROR AT BUILD_Y_N_INPUT");

                            let y_n_input = y_n.trim().chars().next();

                            let what_user_wants_to_change = match y_n_input {
                                Some('N') | Some('n') => {
                                    Select::new("Which input you wanting to change?", vec!["MODEL TYPE", "PROJECT FOLDER", "OUTPUT NAME"]).prompt().unwrap().to_string()
                                },
                                Some('Y') | Some('y') => { "".to_string() }

                                _ => {
                                    printd!("Unkown input! Please answer correctly.", Failed);
                                    continue
                                } ,
                            };

                            if what_user_wants_to_change == "" {
                                let temperature = model_temperature.trim().parse::<f32>().unwrap_or(0.7);
                                let top_k = model_top_k.trim().parse::<i32>().unwrap_or(20);
                                let top_p = model_top_p.trim().parse::<f32>().unwrap_or(0.95);
                                let build = build::LocalBuild::new(
                                    url.to_string(),
                                    model_type.clone(),
                                    std::path::PathBuf::from(project_folder.trim()),
                                    output_name.trim().to_string(),
                                    temperature,
                                    top_k,
                                    top_p,
                                );
                                build.build().await?;

                                continue;
                            }

                            match what_user_wants_to_change.as_str() {
                                "MODEL TYPE" => {
                                    let models = get_ollama_models(&url).unwrap_or_else(|e| {
                                        printd!(format!("Failed to fetch models from Ollama: {}", e).as_str(), Failed);
                                        std::process::exit(1);
                                    });
                                    model_type = Select::new("Select Model:", models).prompt().unwrap();
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
                    Err(e) => {
                        printd!(format!("Failed to fetch models from Ollama: {}", e).as_str(), Failed);
                        std::process::exit(1);
                    }
                }
            } else {
                printd!("Ollama is required for LOCAL mode.", Failed);
                std::process::exit(1);
            }

        } else {
            model_type = match_model_type(api_type.as_str());
            api_key = read_line_input("~Api Key: ", "ERROR AT API_KEY_INPUT");
        }
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
        printd!(format!("API KEY : {}", mask_secret(api_key.trim())).as_str(), Debug);
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
            let exit_received = build.build().await?;

            if exit_received && ask_yes_no("Build process quit by LLM, do you want to save the basic settings (LLM_TYPE, LLM_MODEL, API_KEY) to .env? (Y/N) ") {
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


fn get_ollama_models(url : &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let normalized = url.trim().trim_end_matches('/');
    let endpoint = if normalized.ends_with("/api/tags") {
        normalized.to_string()
    } else if normalized.ends_with("/v1") {
        format!("{}/api/tags", normalized.trim_end_matches("/v1"))
    } else {
        format!("{}/api/tags", normalized)
    };

    let resp = reqwest::blocking::get(endpoint.clone())?;
    let status = resp.status();
    let raw = resp.text()?;

    if !status.is_success() {
        return Err(format!("Ollama API error at {}: {}", endpoint, status).into());
    }

    let data: TagsResponse = serde_json::from_str(&raw)
        .map_err(|e| format!("Ollama tags decode error at {}: {} | body: {}", endpoint, e, raw))?;
    Ok(data.models.into_iter().map(|m| m.name).collect())
}

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