use std::path::PathBuf;

use serde_json::{Value, json};
use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::libs::errors::Error;
use crate::libs::action_executer::{self, ActionResult};
use crate::libs::memory::HistoryEntry;
use crate::libs::prompt;
use crate::printd;

pub async fn local_model_inference(
    ollama_gateway_url : String,
    llm: String,
    project_dir: PathBuf,
    output_file: String,
    temperature: f32,
    top_k: i32,
    top_p: f32
) -> Result<bool, Error> {

    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| ollama_gateway_url);
    let base_url = if ollama_host.starts_with("http://") || ollama_host.starts_with("https://") {
        ollama_host
    } else {
        format!("http://{}", ollama_host)
    };
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));

    let mut temporary_memory = crate::libs::memory::Memory::default();
    let project_tree = action_executer::project_tree_snapshot(&project_dir);
    let tree_context = format!("PROJECT DIRECTORY TREE:\n{}", project_tree);
    let system_prompt = prompt::Prompt::default().content;

    let client = Client::new();
    let body = json!({
        "model": llm,
        "messages": [
            {"role":"system","content":system_prompt.clone()},
            {"role":"system","content": tree_context},
            {"role":"user","content":"MAKEREADME AGENTIC LOOP IS STARTED, START TALKING"}
        ],
        "stream": false,
        "options": {
            "temperature": temperature,
            "top_k": top_k,
            "top_p": top_p
        }
    });

    temporary_memory.append_to_history(HistoryEntry {
        content: system_prompt,
        role: "System".to_string(),
    });
    temporary_memory.append_to_history(HistoryEntry {
        content: tree_context.clone(),
        role: "System".to_string(),
    });
    temporary_memory.append_to_history(HistoryEntry {
        content: "MAKEREADME AGENTIC LOOP IS STARTED, START TALKING".to_string(),
        role: "System".to_string(),
    });

    printd!(
        format!("Sending request to Ollama (model: {})", llm).as_str(),
        Debug
    );

    let mut text = send_ollama_request(&client, &url, &body).await?;
    let mut no_action_retries = 0usize;

    loop {
        sleep(Duration::from_secs(3)).await;

        let mut action_results: Vec<ActionResult> = Vec::new();
        print_raw_model_response(&text);
        printd!("Parsing actions from Ollama response...", Action);

        let actions = action_executer::parse_actions(&text);
        if actions.is_empty() {
            no_action_retries += 1;
            printd!(
                format!(
                    "No valid actions parsed from model output (retry {}/3). Requesting strict tagged response.",
                    no_action_retries
                )
                .as_str(),
                Failed
            );

            if no_action_retries >= 3 {
                return Err(Error::LocalModelInferenceError(
                    "Model did not return any valid action tags after 3 retries.".to_string(),
                ));
            }

            temporary_memory.append_to_history(HistoryEntry {
                content: text.clone(),
                role: "Model".to_string(),
            });

            text = create_ollama_format_retry_response(
                client.clone(),
                url.clone(),
                llm.clone(),
                temperature,
                top_k,
                top_p,
                &temporary_memory,
            )
            .await?;

            continue;
        }

        no_action_retries = 0;
        let results = action_executer::execute_actions(actions, &project_dir, output_file.as_str());

        let mut should_exit = false;
        let mut i = 1;
        for action_result in results {
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
            printd!("EXIT action received. Stopping Ollama loop.", Success);
            return Ok(true);
        }

        text = create_ollama_response(
            client.clone(),
            url.clone(),
            llm.clone(),
            temperature,
            top_k,
            top_p,
            action_results,
            &mut temporary_memory,
        )
        .await?;

        temporary_memory.append_to_history(HistoryEntry {
            content: text.clone(),
            role: "Model".to_string(),
        });
    }
}

async fn create_ollama_format_retry_response(
    client: Client,
    url: String,
    model: String,
    temperature: f32,
    top_k: i32,
    top_p: f32,
    temporary_memory: &crate::libs::memory::Memory,
) -> Result<String, Error> {
    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": temporary_memory
                    .history
                    .iter()
                    .map(|r| format!("{}: {}", r.role, r.content))
                    .collect::<Vec<String>>()
                    .join("\n")
            },
            {
                "role": "user",
                "content": "FORMAT VIOLATION: Your previous response did not include valid action tags. Reply ONLY with these tags: <THINK>...</THINK>, <READ>...</READ>, <WRITE>...</WRITE>, <EXIT>. Start with <THINK> and include at least one non-THINK action."
            }
        ],
        "stream": false,
        "options": {
            "temperature": temperature,
            "top_k": top_k,
            "top_p": top_p
        }
    });

    send_ollama_request(&client, &url, &body).await
}

async fn create_ollama_response(
    client: Client,
    url: String,
    model: String,
    temperature: f32,
    top_k: i32,
    top_p: f32,
    action_results: Vec<ActionResult>,
    temporary_memory: &mut crate::libs::memory::Memory,
) -> Result<String, Error> {
    for action_result in action_results {
        temporary_memory.append_to_result(action_result);
    }

    let body = json!({
        "model": model,
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
                    .history
                    .iter()
                    .map(|r| format!("{}: {}", r.role, r.content))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        ],
        "stream": false,
        "options": {
            "temperature": temperature,
            "top_k": top_k,
            "top_p": top_p
        }
    });

    send_ollama_request(&client, &url, &body).await
}

async fn send_ollama_request(
    client: &Client,
    url: &str,
    body: &Value,
) -> Result<String, Error> {
    let response = client
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|e| Error::LocalModelInferenceError(format!("Ollama request failed: {}", e)))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(Error::LocalModelInferenceError(format!(
            "Ollama HTTP {}: {}",
            status, error_text
        )));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| Error::LocalModelInferenceError(format!("Ollama parse error: {}", e)))?;

    let text = body
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(text)

}

fn print_raw_model_response(text: &str) {
    // Print the exact model output before action parsing for debugging.
    printd!("================ RAW OLLAMA RESPONSE (BEGIN) ================", Action);
    printd!(text, LLM);
    printd!("================= RAW OLLAMA RESPONSE (END) =================", Action);
}