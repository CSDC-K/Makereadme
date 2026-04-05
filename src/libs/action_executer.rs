use std::fs;
use std::path::{Path, PathBuf};

use crate::printd;

const MAX_ACTIONS_PER_RESPONSE: usize = 3;
const MAX_TREE_DEPTH: usize = 6;
const MAX_TREE_ENTRIES: usize = 400;

pub fn project_tree_snapshot(project_dir: &PathBuf) -> String {
    render_directory_tree(project_dir, ".", MAX_TREE_DEPTH, MAX_TREE_ENTRIES)
}

// ── Action Types ──

#[derive(Debug, Clone)]
pub enum Action {
    Think(String),
    Read(String),
    Write(String),
    Exit,
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

        // Keep all THINKs, truncate only READ/WRITE/EXIT to the limit
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
            "Parsed {} action(s) from LLM response ({} THINK, {} non-THINK)",
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
    let exit_start = input.find("<EXIT>");

    // Find the earliest action tag
    let candidates: Vec<(usize, u8)> = [
        read_start.map(|p| (p, 0u8)),
        write_start.map(|p| (p, 1u8)),
        think_start.map(|p| (p, 2u8)),
        exit_start.map(|p| (p, 3u8)),
    ]
    .iter()
    .filter_map(|x| *x)
    .collect();

    let earliest = candidates.iter().min_by_key(|(pos, _)| pos)?;

    match earliest.1 {
        0 => extract_read(input, earliest.0),
        1 => extract_write(input, earliest.0),
        2 => extract_think(input, earliest.0),
        3 => extract_exit(input, earliest.0),
        _ => None,
    }
}

fn extract_exit(input: &str, start: usize) -> Option<(Action, &str)> {
    let open_tag = "<EXIT>";
    let close_tag = "</EXIT>";

    let content_start = start + open_tag.len();
    let rest = if input[content_start..].starts_with(close_tag) {
        &input[content_start + close_tag.len()..]
    } else {
        &input[content_start..]
    };

    printd!("Extracted EXIT action", Debug);

    Some((Action::Exit, rest))
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
            Action::Exit => ActionResult {
                action: Action::Exit,
                success: true,
                content: "EXIT requested by model".to_string(),
            },
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
    let requested_path = file_path.trim();

    if requested_path == "." || requested_path == "/" {
        let tree = render_directory_tree(project_dir, ".", MAX_TREE_DEPTH, MAX_TREE_ENTRIES);
        return ActionResult {
            action: Action::Read(file_path.to_string()),
            success: true,
            content: tree,
        };
    }

    let full_path: PathBuf = project_dir.join(requested_path);

    printd!(
        format!("Reading file: {}", full_path.display()).as_str(),
        Debug
    );

    if full_path.is_dir() {
        let tree = render_directory_tree(&full_path, requested_path, MAX_TREE_DEPTH, MAX_TREE_ENTRIES);
        return ActionResult {
            action: Action::Read(file_path.to_string()),
            success: true,
            content: tree,
        };
    }

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

fn render_directory_tree(root: &Path, label: &str, max_depth: usize, max_entries: usize) -> String {
    let mut lines: Vec<String> = vec![format!("{}", label)];
    let mut emitted = 0usize;

    build_tree_lines(
        root,
        "",
        0,
        max_depth,
        max_entries,
        &mut emitted,
        &mut lines,
    );

    if emitted >= max_entries {
        lines.push("└── ... (tree truncated: too many entries)".to_string());
    }

    lines.join("\n")
}

fn build_tree_lines(
    dir: &Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    max_entries: usize,
    emitted: &mut usize,
    lines: &mut Vec<String>,
) {
    if depth >= max_depth || *emitted >= max_entries {
        return;
    }

    let mut entries: Vec<fs::DirEntry> = match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(Result::ok).collect(),
        Err(_) => return,
    };

    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        match b_is_dir.cmp(&a_is_dir) {
            std::cmp::Ordering::Equal => a
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase()),
            other => other,
        }
    });

    let total = entries.len();

    for (idx, entry) in entries.into_iter().enumerate() {
        if *emitted >= max_entries {
            return;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_last = idx + 1 == total;
        let branch = if is_last { "└── " } else { "├── " };
        let suffix = if path.is_dir() { "/" } else { "" };

        lines.push(format!("{}{}{}{}", prefix, branch, name, suffix));
        *emitted += 1;

        if path.is_dir() {
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            build_tree_lines(
                &path,
                &child_prefix,
                depth + 1,
                max_depth,
                max_entries,
                emitted,
                lines,
            );
        }
    }
}

fn execute_write(project_dir: &PathBuf, output_file: &str, content: &str) -> ActionResult {
    let full_path: PathBuf = project_dir.join(output_file);

    printd!(
        format!("Writing to output file: {}", full_path.display()).as_str(),
        Debug
    );

    // Overwrite modunda aç — dosya yoksa oluştur
    use std::fs::OpenOptions;
    use std::io::Write;

    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            printd!(
                format!("Failed to create parent directories for '{}': {}", output_file, e).as_str(),
                Failed
            );
            return ActionResult {
                action: Action::Write(content.to_string()),
                success: false,
                content: format!(
                    "ERROR: Could not create parent directories for '{}': {}",
                    output_file, e
                ),
            };
        }
    }

    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&full_path)
    {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => {
                printd!(
                    format!(
                        "Content written to '{}' ({} bytes)",
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
            Action::Exit => {
                context.push_str("EXIT requested by model\n\n");
            }
        }
    }

    context
}
