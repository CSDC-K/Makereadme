use std::path::PathBuf;
use tokio::time::{sleep, Duration};

use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde_json::{Value, json};

use crate::libs::action_executer::{self, ActionResult};
use crate::libs::memory::Responses;
use crate::printd;

// ── Public API ──

pub async fn create_communication(
    api_key: String,
    system_prompt: String,
    model_type: String,
    project_dir: &PathBuf,
    output_file: &str,
) -> Result<String, String> {
    let url = "https://api.llmapi.ai/v1/chat/completions".to_string();
    let normalized_api_key = api_key.trim().to_string();
    let normalized_model = model_type.trim().to_string();

    let mut temporary_memory = crate::libs::memory::Memory::default();
    let project_tree = action_executer::project_tree_snapshot(project_dir);
    let tree_context = format!("PROJECT DIRECTORY TREE:\n{}", project_tree);

    let request_body = json!({
        "model": normalized_model,
        "messages": [
            {
                "role": "system",
                "content": system_prompt.clone()
            },
            {
                "role": "assistant",
                "content": "Understood. I will follow the instructions above."
            },
            {
                "role": "user",
                "content": "READMEMAKER AGENTIC LOOP IS STARTED, START TALKING"
            },
            {
                "role": "user",
                "content": tree_context.clone()
            }
        ]
    });

    temporary_memory.append_to_history(Responses {
        response: system_prompt,
        role: "User".to_string(),
    });

    temporary_memory.append_to_history(Responses {
        response: "Understood. I will follow the instructions above.".to_string(),
        role: "Model".to_string(),
    });

    temporary_memory.append_to_history(Responses {
        response: "READMEMAKER AGENTIC LOOP IS STARTED, START TALKING".to_string(),
        role: "User".to_string(),
    });

    temporary_memory.append_to_history(Responses {
        response: tree_context,
        role: "User".to_string(),
    });

    printd!(
        format!("Sending request to LLMAPI (model: {})", normalized_model).as_str(),
        Debug
    );

    let client = Client::new();
    let mut text = send_llmapi_request(
        &client,
        &url,
        &normalized_api_key,
        &request_body,
    )
    .await?;

    loop {
        sleep(Duration::from_secs(3)).await;

        let mut action_results: Vec<ActionResult> = Vec::new();

        printd!("Parsing actions from LLMAPI response...", Action);

        let actions = action_executer::parse_actions(&text);
        let actions_result = action_executer::execute_actions(actions, project_dir, output_file);

        let mut i = 1;
        let mut should_exit = false;
        for action_result in actions_result {
            if matches!(action_result.action, action_executer::Action::Exit) {
                should_exit = true;
            }
            if action_result.success {
                printd!(
                    format!("Action {} : {:?}", i, action_result.action).as_str(),
                    Success
                );
            } else {
                printd!("Found Broken Action Request!", Failed);
                printd!(
                    format!("Action {} : {:?}", i, action_result.action).as_str(),
                    Failed
                );
            }

            action_results.push(action_result);
            i += 1;
        }

        if should_exit {
            printd!("EXIT action received. Stopping LLMAPI loop.", Success);
            return Ok("Exited by model request".to_string());
        }

        text = create_llmapi_response(
            normalized_api_key.clone(),
            client.clone(),
            url.clone(),
            normalized_model.clone(),
            action_results,
            &mut temporary_memory,
        )
        .await?;

        temporary_memory.append_to_history(Responses {
            response: text.clone(),
            role: "Model".to_string(),
        });
    }
}

async fn create_llmapi_response(
    api_key: String,
    client: Client,
    url: String,
    model_type: String,
    action_results: Vec<ActionResult>,
    temporary_memory: &mut crate::libs::memory::Memory,
) -> Result<String, String> {
    for action_result in action_results {
        temporary_memory.append_to_result(action_result);
    }

    let request_body = json!({
        "model": model_type,
        "messages": [
            {
                "role": "user",
                "content": temporary_memory
                    .execute_result
                    .iter()
                    .map(|r| format!(
                        "{}: {}",
                        if r.success { "Success" } else { "Failed" },
                        r.content
                    ))
                    .collect::<Vec<String>>()
                    .join("\n")
            },
            {
                "role": "user",
                "content": temporary_memory
                    .response_history
                    .iter()
                    .map(|r| format!("{}: {}", r.role, r.response))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        ]
    });

    send_llmapi_request(&client, &url, &api_key, &request_body).await
}

async fn send_llmapi_request(
    client: &Client,
    url: &str,
    api_key: &str,
    body: &Value,
) -> Result<String, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key.trim()))
            .map_err(|e| format!("Invalid Authorization header: {}", e))?,
    );

    let response = client
        .post(url)
        .headers(headers)
        .json(body)
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
            format!("LLMAPI error ({}): {}", status, error_text).as_str(),
            Failed
        );
        return Err(format!("HTTP {}: {}", status, error_text));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(message) = body
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        printd!(format!("LLMAPI returned error: {}", message).as_str(), Failed);
        return Err(format!("LLMAPI error: {}", message));
    }

    let text = body
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(text)
}
