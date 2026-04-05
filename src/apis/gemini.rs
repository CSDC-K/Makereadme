use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::libs::memory::{Responses};
use crate::printd;
use crate::libs::action_executer::{self, ActionResult};

// ── Request Structs ──

#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize, Debug)]
struct Content {
    parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Serialize, Debug)]
struct Part {
    text: String,
}

// ── Response Structs ──

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize, Debug)]
struct CandidateContent {
    parts: Option<Vec<ResponsePart>>,
}

#[derive(Deserialize, Debug)]
struct ResponsePart {
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GeminiError {
    message: String,
    code: Option<i32>,
}

// ── Public API ──

pub async fn create_communication(
    api_key: String,
    system_prompt: String,
    model_type: String,
    project_dir : &PathBuf,
    output_file: &str,
) -> Result<String, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model_type
    );

    let mut temporary_memory = crate::libs::memory::Memory::default();
    let project_tree = action_executer::project_tree_snapshot(project_dir);
    let tree_context = format!("PROJECT DIRECTORY TREE:\n{}", project_tree);

    let request_body = GeminiRequest {
        contents: vec![
            Content {
                parts: vec![Part {
                    text: system_prompt.clone(),
                }],
                role: Some("User".to_string()),
            },
            Content {
                parts: vec![Part {
                    text: "Understood. I will follow the instructions above.".to_string(),
                }],
                role: Some("Model".to_string()),
            },
            Content {
                parts: vec![Part {
                    text: "READMEMAKER AGENTIC LOOP IS STARTED, START TALKING".to_string(),
                }],
                role: Some("User".to_string()),
            },
            Content {
                parts: vec![Part {
                    text: tree_context.clone(),
                }],
                role: Some("User".to_string()),
            },
        ],
    };

    temporary_memory.append_to_history(
        Responses{
            response: system_prompt.clone(),
            role: "User".to_string(),
    });

    temporary_memory.append_to_history(
        Responses{
            response: "Understood. I will follow the instructions above.".to_string(),
            role: "Model".to_string(),
    });

    temporary_memory.append_to_history(
        Responses{
            response: "READMEMAKER AGENTIC LOOP IS STARTED, START TALKING".to_string(),
            role: "User".to_string(),
    });

    temporary_memory.append_to_history(
        Responses{
            response: tree_context,
            role: "User".to_string(),
    });


    printd!(
        format!("Sending request to Gemini API (model: {})", model_type).as_str(),
        Debug
    );

    let client = Client::new();
    let response = client
        .post(&url)
        .header("x-goog-api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        printd!(
            format!("Gemini API error ({}): {}", status, error_text).as_str(),
            Failed
        );
        return Err(format!("HTTP {}: {}", status, error_text));
    }

    let body: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(err) = body.error {
        printd!(
            format!("Gemini API returned error: {}", err.message).as_str(),
            Failed
        );
        return Err(format!("Gemini error: {}", err.message));
    }

    let mut text = body
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .unwrap_or_default();

    loop {

        sleep(Duration::from_secs(3)).await;

        let mut action_results : Vec<ActionResult> = Vec::new();

        printd!("Parsing actions from Gemini response...", Action);

        let actions = action_executer::parse_actions(&text);

        let actions_result = action_executer::execute_actions(actions, project_dir, output_file);
        let mut i = 1;
        let mut should_exit = false;

        for action_result in actions_result {
            if matches!(action_result.action, action_executer::Action::Exit) {
                should_exit = true;
            }
            
            match action_result.success {
                true => {
                    printd!(format!("Action {} : {:?}", i , action_result.action).as_str(), Success);

                    action_results.push(action_result);
                    i += 1;

                }

                false => {
                    printd!("Found Broken Action Request!", Failed);
                    printd!(format!("Action {} : {:?}", i , action_result.action).as_str(), Failed);

                    action_results.push(action_result);
                    i += 1;

                }

            }
        }

        if should_exit {
            printd!("EXIT action received. Stopping Gemini loop.", Success);
            return Ok("Exited by model request".to_string());
        }

        text = create_gemini_response(api_key.clone(), client.clone(), url.clone(), action_results, &mut temporary_memory).await?;

        temporary_memory.append_to_history(
            Responses{
                response: text.clone(),
                role: "Model".to_string(),
            }
        );



    }

}



async fn create_gemini_response(api_key: String, client: Client, url : String, action_results: Vec<ActionResult>, temporary_memory: &mut crate::libs::memory::Memory) -> Result<String, String> {

    for action_result in action_results {
        temporary_memory.append_to_result(action_result);
    }


    let response = client
    .post(&url)
    .header("x-goog-api-key", &api_key)
    .header("Content-Type", "application/json")
    .json(&GeminiRequest {
        contents: vec![
            // sending action results to the model for better context
            Content {
                parts: vec![Part {
                    text: temporary_memory.execute_result.iter().map(|r| format!("{}: {}", if r.success { "Success" } else { "Failed" }, r.content)).collect::<Vec<String>>().join("\n"),
                }],
                role: Some("User".to_string()),
                
            },

            // sending temporary memory to the model for better context
            Content {
                parts: vec![Part {
                    text : temporary_memory.response_history.iter().map(|r| format!("{}: {}", r.role, r.response)).collect::<Vec<String>>().join("\n"),
                }],
                role: Some("User".to_string()),
            }
            
        ],
    })
    .send()
    .await
    .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();

    if (!status.is_success()) {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        printd!(
            format!("Gemini API error ({}): {}", status, error_text).as_str(),
            Failed
        );
        return Err(format!("HTTP {}: {}", status, error_text));
    }

    let body = response
        .json::<GeminiResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(err) = body.error {
        printd!(
            format!("Gemini API returned error: {}", err.message).as_str(),
            Failed
        );
        return Err(format!("Gemini error: {}", err.message));
    }

    let text = body
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .unwrap_or_default();

    Ok(text)

}