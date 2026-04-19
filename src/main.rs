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
    pub mod llama_cpp2;         // llama.cpp local backend integration.
}

use std::env;
use std::fs;
use std::io::{Write, stdin, stdout};
use std::path::Path;

use colored::*;                 // Colored Terminal Output
use dotenv::dotenv;             // ENV
use inquire::Select;            // Interactive CLI Selections
use libs::build::*;             // Build Struct
use libs::debug::*;             // DEBUG INFO
use libs::errors::Error;        // Error Handling

use crate::libs::build;
use crate::local::llama_cpp2;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        printd!(format!("{e}").as_str(), Failed);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let api_types_vec = vec!["LOCAL", "GEMINI", "GROQ", "LLMAPI", "NVIDIA"];
    let banner: String = r#"
▗▖  ▗▖ ▗▄▖ ▗▖ ▗▖▗▄▄▄▖▗▄▄▖ ▗▄▄▄▖ ▗▄▖ ▗▄▄▄  ▗▖  ▗▖▗▄▄▄▖
▐▛▚▞▜▌▐▌ ▐▌▐▌▗▞▘▐▌   ▐▌ ▐▌▐▌   ▐▌ ▐▌▐▌  █ ▐▛▚▞▜▌▐▌
▐▌  ▐▌▐▛▀▜▌▐▛▚▖ ▐▛▀▀▘▐▛▀▚▖▐▛▀▀▘▐▛▀▜▌▐▌  █ ▐▌  ▐▌▐▛▀▀▘
▐▌  ▐▌▐▌ ▐▌▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▖▐▌ ▐▌▐▙▄▄▀ ▐▌  ▐▌▐▙▄▄▖

-- Generate README.md files with the power of LLMs! --
-- Made By Kuzey (CSDC-K)
    "#
    .to_string();

    println!("{}", banner.bright_yellow());
    println!("Welcome to MakeREADME V 0.0.1");

    let mut load_env = String::new();
    print!("Do you want to load .env file? (Y/N) ");
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut load_env)
        .expect("ERROR AT LOAD_ENV_INPUT");

    let load_env_file = match load_env.trim().chars().next() {
        Some('N') | Some('n') => {
            printd!(
                "Starting without .env file. You will be asked to input all the values.",
                Debug
            );
            false
        }
        Some('Y') | Some('y') => {
            dotenv().ok();
            printd!("Loaded .env file successfully!", Success);
            true
        }
        _ => {
            printd!("Unkown input! Starting with defaults.", Failed);
            false
        }
    };

    let mut api_type: String;
    let mut model_type: String;
    let mut api_key: String;

    if load_env_file {
        api_type = read_env_key("LLM_TYPE").unwrap_or_default();

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

        if api_type == "LOCAL" {
            let model_from_env = read_env_key("LLM_MODEL");
            run_local_mode(model_from_env).await?;
            return Ok(());
        }

        model_type = read_env_key("LLM_MODEL").unwrap_or_default();
        api_key = read_env_key("API_KEY").unwrap_or_default();

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
            run_local_mode(None).await?;
            return Ok(());
        }

        model_type = match_model_type(api_type.as_str());
        api_key = read_line_input("~Api Key: ", "ERROR AT API_KEY_INPUT");
    }

    let mut project_folder = read_line_input("~Project Folder: ", "ERROR AT PROJECT_FOLDER_INPUT");
    let mut output_name = read_line_input("~Output File Name: ", "ERROR AT OUTPUT_INPUT");

    loop {
        printd!("Reading configs...", Debug);
        printd!(format!("LLM API : {}", api_type.as_str()).as_str(), Debug);
        printd!(format!("MODEL TYPE : {}", model_type.as_str()).as_str(), Debug);
        printd!(format!("API KEY : {}", mask_secret(api_key.trim())).as_str(), Debug);
        printd!(format!("PROJECT DIR : {}", project_folder.trim()).as_str(), Debug);
        printd!(format!("OUTPUT FILE : {}", output_name.trim()).as_str(), Debug);
        print!("Is that build true? (Y/N) ");
        stdout().flush().unwrap();

        let mut y_n = String::new();
        stdin()
            .read_line(&mut y_n)
            .expect("ERROR AT BUILD_Y_N_INPUT");

        let what_user_wants_to_change = match y_n.trim().chars().next() {
            Some('N') | Some('n') => Select::new(
                "Which input you wanting to change?",
                vec!["API TYPE", "MODEL TYPE", "API KEY", "PROJECT FOLDER", "OUTPUT NAME"],
            )
            .prompt()
            .unwrap(),
            Some('Y') | Some('y') => "",
            _ => {
                printd!("Unkown input! Please answer correctly.", Failed);
                continue;
            }
        };

        if what_user_wants_to_change.is_empty() {
            let build = Build::new(
                api_type.clone(),
                model_type.trim().to_string(),
                api_key.trim().to_string(),
                std::path::PathBuf::from(project_folder.trim()),
                output_name.trim().to_string(),
            );
            let exit_received = build.build().await?;

            if exit_received
                && ask_yes_no(
                    "Build process quit by LLM, do you want to save the basic settings (LLM_TYPE, LLM_MODEL, API_KEY) to .env? (Y/N) ",
                )
            {
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

                if api_type == "LOCAL" {
                    run_local_mode(None).await?;
                    return Ok(());
                }

                model_type = match_model_type(api_type.as_str());
            }
            "MODEL TYPE" => {
                model_type = match_model_type(api_type.as_str());
            }
            "API KEY" => {
                api_key = read_line_input("~Api Key: ", "ERROR AT API_KEY_INPUT");
            }
            "PROJECT FOLDER" => {
                project_folder = read_line_input("~Project Folder: ", "ERROR AT PROJECT_FOLDER_INPUT");
            }
            "OUTPUT NAME" => {
                output_name = read_line_input("~Output File Name: ", "ERROR AT OUTPUT_INPUT");
            }
            _ => {
                printd!("Unknown selection. Keeping current values.", Failed);
            }
        }
    }
}

async fn run_local_mode(model_path_from_env: Option<String>) -> Result<(), Error> {
    let api_type = "LOCAL".to_string();
    let mut model_path = match model_path_from_env {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => read_existing_file_path("~Model Full Path (.gguf): "),
    };

    if !Path::new(&model_path).exists() {
        printd!("LLM_MODEL in .env is not a valid file path. Asking interactively.", Failed);
        model_path = read_existing_file_path("~Model Full Path (.gguf): ");
    }

    let mut llm_model_alias = llama_cpp2::default_model_alias(&model_path);
    let mut gpu_backend = Select::new("Select GPU backend:", vec!["NVIDIA", "AMD", "CPU"])
        .prompt()
        .unwrap()
        .to_string();

    let mut context_size = read_input_with_default("~Context size", "16384")
        .parse::<u32>()
        .unwrap_or(16384);
    let mut batch_size = read_input_with_default("~Batch size", "2048")
        .parse::<u32>()
        .unwrap_or(2048);
    let mut threads = read_input_with_default("~Threads (0=auto)", "0")
        .parse::<i32>()
        .unwrap_or(0);

    let mut model_temperature = read_input_with_default("~Model Temperature", "0.7")
        .parse::<f32>()
        .unwrap_or(0.7);
    let mut model_top_k = read_input_with_default("~Model Top K", "20")
        .parse::<i32>()
        .unwrap_or(20);
    let mut model_top_p = read_input_with_default("~Model Top P", "0.95")
        .parse::<f32>()
        .unwrap_or(0.95);

    let mut project_folder = read_line_input("~Project Folder: ", "ERROR AT PROJECT_FOLDER_INPUT");
    let mut output_name = read_line_input("~Output File Name: ", "ERROR AT OUTPUT_INPUT");

    loop {
        printd!("Reading local configs...", Debug);
        printd!(format!("LLM API : {}", api_type).as_str(), Debug);
        printd!(format!("MODEL PATH : {}", model_path).as_str(), Debug);
        printd!(format!("MODEL ALIAS : {}", llm_model_alias).as_str(), Debug);
        printd!(format!("GPU BACKEND : {}", gpu_backend).as_str(), Debug);
        printd!(format!("CONTEXT SIZE : {}", context_size).as_str(), Debug);
        printd!(format!("BATCH SIZE : {}", batch_size).as_str(), Debug);
        printd!(format!("THREADS : {}", threads).as_str(), Debug);
        printd!(format!("TEMPERATURE : {}", model_temperature).as_str(), Debug);
        printd!(format!("TOP K : {}", model_top_k).as_str(), Debug);
        printd!(format!("TOP P : {}", model_top_p).as_str(), Debug);
        printd!(format!("PROJECT DIR : {}", project_folder.trim()).as_str(), Debug);
        printd!(format!("OUTPUT FILE : {}", output_name.trim()).as_str(), Debug);
        print!("Is that build true? (Y/N) ");
        stdout().flush().unwrap();

        let mut y_n = String::new();
        stdin()
            .read_line(&mut y_n)
            .expect("ERROR AT BUILD_Y_N_INPUT");

        let what_user_wants_to_change = match y_n.trim().chars().next() {
            Some('N') | Some('n') => Select::new(
                "Which input you wanting to change?",
                vec![
                    "MODEL PATH",
                    "MODEL ALIAS",
                    "GPU BACKEND",
                    "CONTEXT SIZE",
                    "BATCH SIZE",
                    "THREADS",
                    "TEMPERATURE",
                    "TOP K",
                    "TOP P",
                    "PROJECT FOLDER",
                    "OUTPUT NAME",
                ],
            )
            .prompt()
            .unwrap()
            .to_string(),
            Some('Y') | Some('y') => "".to_string(),
            _ => {
                printd!("Unkown input! Please answer correctly.", Failed);
                continue;
            }
        };

        if what_user_wants_to_change.is_empty() {
            let build = build::LocalBuild::new(
                model_path.clone(),
                gpu_backend.clone(),
                context_size,
                batch_size,
                threads,
                llm_model_alias.clone(),
                std::path::PathBuf::from(project_folder.trim()),
                output_name.trim().to_string(),
                model_temperature,
                model_top_k,
                model_top_p,
            );
            let exit_received = build.build().await?;

            if exit_received
                && ask_yes_no(
                    "Build process quit by LLM, do you want to save LOCAL settings (LLM_TYPE=LOCAL, LLM_MODEL=model_path)? (Y/N) ",
                )
            {
                match save_llm_settings_to_env("LOCAL", "NONE", model_path.as_str()) {
                    Ok(_) => printd!("Settings saved to .env", Success),
                    Err(e) => printd!(format!("Failed to save .env: {}", e).as_str(), Failed),
                }
            }

            continue;
        }

        match what_user_wants_to_change.as_str() {
            "MODEL PATH" => {
                model_path = read_existing_file_path("~Model Full Path (.gguf): ");
                llm_model_alias = llama_cpp2::default_model_alias(&model_path);
            }
            "MODEL ALIAS" => {
                llm_model_alias = read_input_with_default("~Model alias", llm_model_alias.as_str());
            }
            "GPU BACKEND" => {
                gpu_backend = Select::new("Select GPU backend:", vec!["NVIDIA", "AMD", "CPU"])
                    .prompt()
                    .unwrap()
                    .to_string();
            }
            "CONTEXT SIZE" => {
                context_size = read_input_with_default("~Context size", context_size.to_string().as_str())
                    .parse::<u32>()
                    .unwrap_or(context_size);
            }
            "BATCH SIZE" => {
                batch_size = read_input_with_default("~Batch size", batch_size.to_string().as_str())
                    .parse::<u32>()
                    .unwrap_or(batch_size);
            }
            "THREADS" => {
                threads = read_input_with_default("~Threads (0=auto)", threads.to_string().as_str())
                    .parse::<i32>()
                    .unwrap_or(threads);
            }
            "TEMPERATURE" => {
                model_temperature = read_input_with_default("~Model Temperature", model_temperature.to_string().as_str())
                    .parse::<f32>()
                    .unwrap_or(model_temperature);
            }
            "TOP K" => {
                model_top_k = read_input_with_default("~Model Top K", model_top_k.to_string().as_str())
                    .parse::<i32>()
                    .unwrap_or(model_top_k);
            }
            "TOP P" => {
                model_top_p = read_input_with_default("~Model Top P", model_top_p.to_string().as_str())
                    .parse::<f32>()
                    .unwrap_or(model_top_p);
            }
            "PROJECT FOLDER" => {
                project_folder = read_line_input("~Project Folder: ", "ERROR AT PROJECT_FOLDER_INPUT");
            }
            "OUTPUT NAME" => {
                output_name = read_line_input("~Output File Name: ", "ERROR AT OUTPUT_INPUT");
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
    stdin()
        .read_line(&mut answer)
        .expect("ERROR AT YES_NO_INPUT");

    matches!(answer.trim().chars().next(), Some('Y') | Some('y'))
}

fn read_line_input(prompt: &str, error_message: &str) -> String {
    let mut value = String::new();
    print!("{}", prompt);
    stdout().flush().unwrap();
    stdin().read_line(&mut value).expect(error_message);
    value.trim().to_string()
}

fn read_input_with_default(prompt: &str, default_value: &str) -> String {
    let mut value = String::new();
    print!("{} (default {}): ", prompt, default_value);
    stdout().flush().unwrap();
    stdin().read_line(&mut value).expect("ERROR AT INPUT");

    let trimmed = value.trim();
    if trimmed.is_empty() {
        default_value.to_string()
    } else {
        trimmed.to_string()
    }
}

fn read_existing_file_path(prompt: &str) -> String {
    loop {
        let value = read_line_input(prompt, "ERROR AT MODEL_PATH_INPUT");
        let path = Path::new(&value);

        if path.exists() && path.is_file() {
            return value;
        }

        printd!(
            format!("Model file not found or invalid path: {}", value).as_str(),
            Failed
        );
    }
}

fn read_env_key(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
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

fn match_model_type(llm_type: &str) -> String {
    let llm_models_gemini = vec![
        "gemini-2.5-flash-lite",
        "gemini-2.5-flash",
        "gemini-2.5-pro",
        "gemini-3.0-flash-lite",
        "gemini-3.0-flash",
        "gemini-3.0-pro",
    ];
    let llm_models_groq = vec![
        "llama-3.1-8b-instant",
        "llama-3.3-70b-versatile",
        "meta-llama/llama-4-scout-17b-16e-instruct",
        "openai/gpt-oss-120b",
        "openai/gpt-oss-20b",
        "qwen/qwen3-32b",
    ];

    match llm_type {
        "GEMINI" => Select::new("Select Model:", llm_models_gemini)
            .prompt()
            .unwrap()
            .to_string(),
        "GROQ" => Select::new("Select Model:", llm_models_groq)
            .prompt()
            .unwrap()
            .to_string(),
        "NVIDIA" => {
            let mut nvidia_model: String = String::new();
            print!("$~ NVIDIA MODEL : ");
            stdout().flush().unwrap();
            stdin()
                .read_line(&mut nvidia_model)
                .expect("ERROR AT NVIDIA_MODEL_INPUT");
            nvidia_model.trim().to_string()
        }
        "LLMAPI" => {
            let mut llmapi_model: String = String::new();
            print!("$~ LLMAPI MODEL : ");
            stdout().flush().unwrap();
            stdin()
                .read_line(&mut llmapi_model)
                .expect("ERROR AT LLMAPI_MODEL_INPUT");
            llmapi_model.trim().to_string()
        }
        _ => {
            printd!(
                "Model selection process failed because of match did not get any llm_type",
                Failed
            );
            panic!();
        }
    }
}
