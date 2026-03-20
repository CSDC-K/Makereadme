use std::path::PathBuf;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::printd;
use crate::libs::action_executer;

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
    project_dir : &PathBuf
) -> Result<String, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model_type
    );

    let request_body = GeminiRequest {
        contents: vec![
            Content {
                parts: vec![Part {
                    text: system_prompt,
                }],
                role: Some("Model".to_string()),
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
                role: Some("Model".to_string()),
            },
        ],
    };

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
        printd!(
            format!("Gemini response received ({} chars)", text.len()).as_str(),
            LLM
        );

        let actions = action_executer::parse_actions(&text);

        let actions_result = action_executer::execute_actions(actions, project_dir, "README.md");
        let i = 1;
        for action_result in actions_result {
            
            match action_result.success {
                true => {
                    printd!(format!("Action {} : {:?}", i , action_result.action).as_str(), Success);
                    printd!(format!("Action {}'s Content : {:?}", i , action_result.action).as_str(), Success);
 

                }

                false => {
                    printd!("Found Broken Action Request!", Failed);
                    printd!(format!("Action {} : {:?}", i , action_result.action).as_str(), Failed);
                    printd!(format!("Action {}'s Content : {:?}", i , action_result.action).as_str(), Failed);
                }

            }
        }


    }



    Ok(text)
}