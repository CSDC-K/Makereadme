use std::fs;
use std::path::{Path, PathBuf};

use crate::printd;

const MAX_ACTIONS_PER_RESPONSE: usize = 3;

// ── Action Types ──

#[derive(Debug, Clone)]
pub enum Action {
    Think(String),
    Read(String),
    Write(String),
}

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action: Action,
    pub success: bool,
    pub content: String,
}

// ── Parser ──

pub fn parse_actions(response: &str) -> Vec<Action> {
    let mut all_actions: Vec<Action> = Vec::new();

    let mut remaining = response;

    while let Some(action) = extract_next_action(remaining) {
        match &action {
            (act, rest) => {
                all_actions.push(act.clone());
                remaining = rest;
            }
        }
    }

    // THINK tags do NOT count toward the action limit
    let think_count = all_actions.iter().filter(|a| matches!(a, Action::Think(_))).count();
    let non_think_count = all_actions.iter().filter(|a| !matches!(a, Action::Think(_))).count();

    if non_think_count > MAX_ACTIONS_PER_RESPONSE {
        printd!(
            format!(
                "Action limit exceeded! Found {} non-THINK actions, max allowed is {}. Truncating.",
                non_think_count,
                MAX_ACTIONS_PER_RESPONSE
            )
            .as_str(),
            Failed
        );

        // Keep all THINKs, truncate only READ/WRITE to the limit
        let mut kept: Vec<Action> = Vec::new();
        let mut rw_count = 0;
        for action in all_actions {
            match &action {
                Action::Think(_) => kept.push(action),
                _ => {
                    if rw_count < MAX_ACTIONS_PER_RESPONSE {
                        kept.push(action);
                        rw_count += 1;
                    }
                }
            }
        }
        all_actions = kept;
    }

    printd!(
        format!(
            "Parsed {} action(s) from LLM response ({} THINK, {} READ/WRITE)",
            all_actions.len(),
            think_count,
            non_think_count.min(MAX_ACTIONS_PER_RESPONSE)
        )
        .as_str(),
        Debug
    );

    all_actions
}

fn extract_next_action(input: &str) -> Option<(Action, &str)> {
    let read_start = input.find("<READ>");
    let write_start = input.find("<WRITE>");
    let think_start = input.find("<THINK>");

    // Find the earliest action tag
    let candidates: Vec<(usize, u8)> = [
        read_start.map(|p| (p, 0u8)),
        write_start.map(|p| (p, 1u8)),
        think_start.map(|p| (p, 2u8)),
    ]
    .iter()
    .filter_map(|x| *x)
    .collect();

    let earliest = candidates.iter().min_by_key(|(pos, _)| pos)?;

    match earliest.1 {
        0 => extract_read(input, earliest.0),
        1 => extract_write(input, earliest.0),
        2 => extract_think(input, earliest.0),
        _ => None,
    }
}

fn extract_think(input: &str, start: usize) -> Option<(Action, &str)> {
    let open_tag = "<THINK>";
    let close_tag = "</THINK>";

    let content_start = start + open_tag.len();
    let close_pos = input[content_start..].find(close_tag)?;
    let content = &input[content_start..content_start + close_pos];
    let rest = &input[content_start + close_pos + close_tag.len()..];

    let trimmed = content.strip_prefix('\n').unwrap_or(content);
    let trimmed = trimmed.strip_suffix('\n').unwrap_or(trimmed);

    printd!(
        format!("Extracted THINK action: {} chars", trimmed.len()).as_str(),
        LLM
    );

    Some((Action::Think(trimmed.to_string()), rest))
}

fn extract_read(input: &str, start: usize) -> Option<(Action, &str)> {
    let open_tag = "<READ>";
    let close_tag = "</READ>";

    let content_start = start + open_tag.len();
    let close_pos = input[content_start..].find(close_tag)?;
    let content = input[content_start..content_start + close_pos].trim();
    let rest = &input[content_start + close_pos + close_tag.len()..];

    printd!(format!("Extracted READ action: {}", content).as_str(), Debug);

    Some((Action::Read(content.to_string()), rest))
}

fn extract_write(input: &str, start: usize) -> Option<(Action, &str)> {
    let open_tag = "<WRITE>";
    let close_tag = "</WRITE>";

    let content_start = start + open_tag.len();
    let close_pos = input[content_start..].find(close_tag)?;
    let content = &input[content_start..content_start + close_pos];
    let rest = &input[content_start + close_pos + close_tag.len()..];

    // WRITE içeriğinin baş ve sonundaki tek newline'ları temizle
    let trimmed = content.strip_prefix('\n').unwrap_or(content);
    let trimmed = trimmed.strip_suffix('\n').unwrap_or(trimmed);

    printd!(
        format!("Extracted WRITE action: {} chars", trimmed.len()).as_str(),
        Debug
    );

    Some((Action::Write(trimmed.to_string()), rest))
}

// ── Executor ──

pub fn execute_actions(
    actions: Vec<Action>,
    project_dir: &PathBuf,
    output_file: &str,
) -> Vec<ActionResult> {
    let mut results: Vec<ActionResult> = Vec::new();

    printd!(
        format!(
            "Executing {} action(s) in project dir: {}",
            actions.len(),
            project_dir.display()
        )
        .as_str(),
        Debug
    );

    for (i, action) in actions.iter().enumerate() {
        printd!(
            format!("Executing action {}/{}...", i + 1, actions.len()).as_str(),
            Debug
        );

        let result = match action {
            Action::Think(thought) => {
                printd!(
                    format!("LLM THINKING:\n{}", thought).as_str(),
                    LLM
                );
                ActionResult {
                    action: action.clone(),
                    success: true,
                    content: thought.clone(),
                }
            }
            Action::Read(file_path) => execute_read(project_dir, file_path),
            Action::Write(content) => execute_write(project_dir, output_file, content),
        };

        match &result {
            ActionResult {
                success: true,
                content,
                ..
            } => {
                printd!(
                    format!(
                        "Action {}/{} completed successfully ({} chars)",
                        i + 1,
                        actions.len(),
                        content.len()
                    )
                    .as_str(),
                    Success
                );
            }
            ActionResult {
                success: false,
                content,
                ..
            } => {
                printd!(
                    format!("Action {}/{} failed: {}", i + 1, actions.len(), content).as_str(),
                    Failed
                );
            }
        }

        results.push(result);
    }

    printd!(
        format!(
            "All actions executed. Success: {}, Failed: {}",
            results.iter().filter(|r| r.success).count(),
            results.iter().filter(|r| !r.success).count()
        )
        .as_str(),
        Debug
    );

    results
}

fn execute_read(project_dir: &PathBuf, file_path: &str) -> ActionResult {
    let full_path: PathBuf = project_dir.join(file_path);

    printd!(
        format!("Reading file: {}", full_path.display()).as_str(),
        Debug
    );

    match fs::read_to_string(&full_path) {
        Ok(content) => {
            printd!(
                format!(
                    "File read successfully: {} ({} bytes)",
                    file_path,
                    content.len()
                )
                .as_str(),
                Success
            );
            ActionResult {
                action: Action::Read(file_path.to_string()),
                success: true,
                content,
            }
        }
        Err(e) => {
            printd!(
                format!("Failed to read file '{}': {}", file_path, e).as_str(),
                Failed
            );
            ActionResult {
                action: Action::Read(file_path.to_string()),
                success: false,
                content: format!("ERROR: Could not read file '{}': {}", file_path, e),
            }
        }
    }
}

fn execute_write(project_dir: &PathBuf, output_file: &str, content: &str) -> ActionResult {
    let full_path: PathBuf = project_dir.join(output_file);

    printd!(
        format!("Writing to output file: {}", full_path.display()).as_str(),
        Debug
    );

    // Append modunda aç — dosya yoksa oluştur
    use std::fs::OpenOptions;
    use std::io::Write;

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&full_path)
    {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => {
                // Sonuna newline ekle
                let _ = file.write_all(b"\n");
                printd!(
                    format!(
                        "Content appended to '{}' ({} bytes)",
                        output_file,
                        content.len()
                    )
                    .as_str(),
                    Success
                );
                ActionResult {
                    action: Action::Write(content.to_string()),
                    success: true,
                    content: format!("Successfully wrote {} bytes to '{}'", content.len(), output_file),
                }
            }
            Err(e) => {
                printd!(
                    format!("Failed to write to '{}': {}", output_file, e).as_str(),
                    Failed
                );
                ActionResult {
                    action: Action::Write(content.to_string()),
                    success: false,
                    content: format!("ERROR: Could not write to '{}': {}", output_file, e),
                }
            }
        },
        Err(e) => {
            printd!(
                format!("Failed to open '{}' for writing: {}", output_file, e).as_str(),
                Failed
            );
            ActionResult {
                action: Action::Write(content.to_string()),
                success: false,
                content: format!("ERROR: Could not open '{}': {}", output_file, e),
            }
        }
    }
}

// ── Helper: Build context string from action results ──

pub fn build_context_from_results(results: &[ActionResult]) -> String {
    let mut context = String::new();

    for result in results {
        match &result.action {
            Action::Think(_) => {
                // THINK actions are internal — not included in context sent back to LLM
            }
            Action::Read(path) => {
                if result.success {
                    context.push_str(&format!(
                        "--- FILE: {} ---\n{}\n--- END FILE ---\n\n",
                        path, result.content
                    ));
                } else {
                    context.push_str(&format!(
                        "--- FILE: {} ---\n{}\n--- END FILE ---\n\n",
                        path, result.content
                    ));
                }
            }
            Action::Write(_) => {
                if result.success {
                    context.push_str(&format!("{}\n\n", result.content));
                } else {
                    context.push_str(&format!("WRITE ERROR: {}\n\n", result.content));
                }
            }
        }
    }

    context
}
