use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::{LlamaModelParams, LlamaSplitMode};
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::{LlamaBackendDeviceType, list_llama_ggml_backend_devices};

use crate::libs::action_executer::{self, ActionResult};
use crate::libs::errors::Error;
use crate::libs::memory::HistoryEntry;
use crate::libs::prompt;
use crate::printd;

pub async fn local_model_inference(
    model_path: String,
    gpu_backend: String,
    context_size: u32,
    batch_size: u32,
    threads: i32,
    llm_model: String,
    project_dir: PathBuf,
    output_file: String,
    temperature: f32,
    top_k: i32,
    top_p: f32,
) -> Result<bool, Error> {
    ensure_model_file_exists(&model_path)?;

    printd!(
        format!(
            "Initializing llama-cpp-2 backend (backend={}, model={})",
            gpu_backend, llm_model
        )
        .as_str(),
        Debug
    );

    let backend = LlamaBackend::init().map_err(|e| {
        Error::LocalModelInferenceError(format!("llama backend init failed: {}", e))
    })?;

    let backend_devices = list_llama_ggml_backend_devices();
    for device in &backend_devices {
        printd!(
            format!(
                "Detected device idx={} backend={} type={:?} name={} desc={}",
                device.index, device.backend, device.device_type, device.name, device.description
            )
            .as_str(),
            Debug
        );
    }

    let use_gpu = !gpu_backend.eq_ignore_ascii_case("CPU");
    let gpu_layers = if use_gpu {
        u32::MAX
    } else {
        0
    };

    let mut model_params = LlamaModelParams::default();
    model_params = model_params.with_n_gpu_layers(gpu_layers);

    if use_gpu {
        let gpu_devices: Vec<_> = backend_devices
            .iter()
            .filter(|d| {
                matches!(
                    d.device_type,
                    LlamaBackendDeviceType::Gpu | LlamaBackendDeviceType::IntegratedGpu
                )
            })
            .collect();

        if gpu_devices.is_empty() {
            return Err(Error::LocalModelInferenceError(
                "GPU backend selected but llama.cpp found no GPU devices. Rebuild with GPU features (AMD ROCm: llama-cpp-2 feature 'rocm').".to_string(),
            ));
        }

        let preferred_gpu = gpu_devices
            .iter()
            .find(|d| {
                let backend_name = d.backend.to_ascii_lowercase();
                if gpu_backend.eq_ignore_ascii_case("AMD") {
                    backend_name.contains("hip")
                        || backend_name.contains("rocm")
                        || backend_name.contains("vulkan")
                } else if gpu_backend.eq_ignore_ascii_case("NVIDIA") {
                    backend_name.contains("cuda") || backend_name.contains("vulkan")
                } else {
                    true
                }
            })
            .copied()
            .unwrap_or(gpu_devices[0]);

        printd!(
            format!(
                "Using GPU device idx={} backend={} name={}",
                preferred_gpu.index, preferred_gpu.backend, preferred_gpu.name
            )
            .as_str(),
            Debug
        );

        model_params = model_params
            .with_split_mode(LlamaSplitMode::None)
            .with_main_gpu(preferred_gpu.index as i32);
        model_params = model_params
            .with_devices(&[preferred_gpu.index])
            .map_err(|e| Error::LocalModelInferenceError(format!("device select failed: {}", e)))?;
    }

    let model = LlamaModel::load_from_file(&backend, Path::new(&model_path), &model_params)
        .map_err(|e| Error::LocalModelInferenceError(format!("model load failed: {}", e)))?;

    run_action_loop(
        &backend,
        &model,
        use_gpu,
        context_size,
        batch_size,
        threads,
        project_dir,
        output_file,
        temperature,
        top_k,
        top_p,
    )
    .await
}

async fn run_action_loop(
    backend: &LlamaBackend,
    model: &LlamaModel,
    use_gpu: bool,
    context_size: u32,
    batch_size: u32,
    threads: i32,
    project_dir: PathBuf,
    output_file: String,
    temperature: f32,
    top_k: i32,
    top_p: f32,
) -> Result<bool, Error> {
    let mut temporary_memory = crate::libs::memory::Memory::default();
    let project_tree = action_executer::project_tree_snapshot(&project_dir);
    let tree_context = format!("PROJECT DIRECTORY TREE:\n{}", project_tree);
    let system_prompt = prompt::Prompt::default().content;

    temporary_memory.append_to_history(HistoryEntry {
        content: system_prompt,
        role: "System".to_string(),
    });
    temporary_memory.append_to_history(HistoryEntry {
        content: tree_context,
        role: "System".to_string(),
    });

    let mut no_action_retries = 0usize;

    loop {
        let initial_prompt = compose_iteration_prompt(&temporary_memory, None);
        let text = generate_completion(
            backend,
            model,
            use_gpu,
            context_size,
            batch_size,
            threads,
            &initial_prompt,
            2048,
            temperature,
            top_k,
            top_p,
        )?;

        print_raw_model_response(&text);
        printd!("Parsing actions from llama-cpp-2 response...", Action);

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
                content: text,
                role: "Model".to_string(),
            });

            let retry_prompt = compose_iteration_prompt(
                &temporary_memory,
                Some("FORMAT VIOLATION: Reply ONLY with <THINK>...</THINK>, <READ>...</READ>, <WRITE>...</WRITE>, <EXIT>. Start with <THINK> and include at least one non-THINK action."),
            );

            let retry_text = generate_completion(
                backend,
                model,
                use_gpu,
                context_size,
                batch_size,
                threads,
                &retry_prompt,
                1024,
                temperature,
                top_k,
                top_p,
            )?;

            temporary_memory.append_to_history(HistoryEntry {
                content: retry_text,
                role: "Model".to_string(),
            });

            continue;
        }

        no_action_retries = 0;
        let mut action_results: Vec<ActionResult> = Vec::new();
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

        temporary_memory.append_to_history(HistoryEntry {
            content: text,
            role: "Model".to_string(),
        });

        for action_result in action_results {
            temporary_memory.append_to_result(action_result);
        }

        if should_exit {
            printd!("EXIT action received. Stopping local loop.", Success);
            return Ok(true);
        }
    }
}

fn generate_completion(
    backend: &LlamaBackend,
    model: &LlamaModel,
    use_gpu: bool,
    context_size: u32,
    batch_size: u32,
    threads: i32,
    prompt: &str,
    max_tokens: usize,
    temperature: f32,
    top_k: i32,
    top_p: f32,
) -> Result<String, Error> {
    let n_ctx = NonZeroU32::new(context_size.max(512)).ok_or_else(|| {
        Error::LocalModelInferenceError("invalid context size".to_string())
    })?;

    let effective_batch = batch_size.clamp(32, context_size.max(32));
    let effective_ubatch = (effective_batch / 2).max(32).min(effective_batch);

    let mut ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(n_ctx))
        .with_n_batch(effective_batch)
        .with_n_ubatch(effective_ubatch)
        .with_op_offload(use_gpu);
    if threads > 0 {
        ctx_params = ctx_params.with_n_threads(threads);
    }

    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| Error::LocalModelInferenceError(format!("context init failed: {}", e)))?;

    let tokens = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| Error::LocalModelInferenceError(format!("prompt tokenize failed: {}", e)))?;

    if tokens.is_empty() {
        return Err(Error::LocalModelInferenceError(
            "prompt tokenization returned no tokens".to_string(),
        ));
    }

    let batch_capacity = effective_batch as usize;
    let mut batch = LlamaBatch::new(batch_capacity, 1);
    let mut n_cur: i32 = 0;

    for chunk in tokens.chunks(batch_capacity.saturating_sub(1).max(1)) {
        batch.clear();
        for (i, token) in chunk.iter().enumerate() {
            let is_last = i + 1 == chunk.len();
            batch
                .add(*token, n_cur, &[0], is_last)
                .map_err(|e| Error::LocalModelInferenceError(format!("batch add failed: {}", e)))?;
            n_cur += 1;
        }

        ctx.decode(&mut batch)
            .map_err(|e| Error::LocalModelInferenceError(format!("decode failed: {}", e)))?;
    }

    let min_p = (1.0f32 - top_p.clamp(0.0, 1.0)).max(0.01);
    let seed = if top_k <= 0 { 42 } else { top_k as u32 };

    let mut sampler = LlamaSampler::chain_simple([
        LlamaSampler::temp(temperature.max(0.01)),
        LlamaSampler::min_p(min_p, 1),
        LlamaSampler::dist(seed),
    ]);

    let mut full_response = String::new();

    for _ in 0..max_tokens {
        let last_index = if batch.n_tokens() > 0 { batch.n_tokens() - 1 } else { 0 };
        let token = sampler.sample(&ctx, last_index);
        sampler.accept(token);

        if model.is_eog_token(token) {
            break;
        }

        let output_str = model
            .token_to_str(token, Special::Tokenize)
            .map_err(|e| Error::LocalModelInferenceError(format!("token decode failed: {}", e)))?;
        full_response.push_str(&output_str);

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| Error::LocalModelInferenceError(format!("batch update failed: {}", e)))?;
        ctx.decode(&mut batch)
            .map_err(|e| Error::LocalModelInferenceError(format!("decode failed: {}", e)))?;
        n_cur += 1;

        if full_response.contains("<EXIT>") {
            break;
        }
    }

    Ok(full_response)
}

fn compose_iteration_prompt(
    memory: &crate::libs::memory::Memory,
    extra_instruction: Option<&str>,
) -> String {
    let system_memory = memory
        .history
        .iter()
        .filter(|r| r.role == "System")
        .map(|r| r.content.clone())
        .collect::<Vec<String>>()
        .join("\n\n");

    let model_history = memory
        .history
        .iter()
        .filter(|r| r.role != "System")
        .map(|r| format!("{}: {}", r.role, r.content))
        .collect::<Vec<String>>()
        .join("\n");

    let execution_results = memory
        .execute_result
        .iter()
        .map(|r| {
            format!(
                "{}: {}",
                if r.success { "Success" } else { "Failed" },
                r.content
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let mut prompt = format!(
        "You are a coding assistant that MUST use tags: <THINK>, <READ>, <WRITE>, <EXIT>.\n\
         Start with <THINK>. Include at least one non-THINK action until task is complete.\n\
         Never output markdown fences.\n\n\
         SYSTEM CONTEXT:\n{}\n\n\
         EXECUTION RESULTS:\n{}\n\n\
         HISTORY:\n{}\n\n\
         Now produce the next actions.",
        system_memory, execution_results, model_history
    );

    if let Some(extra) = extra_instruction {
        prompt.push_str("\n\n");
        prompt.push_str(extra);
    }

    prompt
}

fn ensure_model_file_exists(model_path: &str) -> Result<(), Error> {
    let path = Path::new(model_path);
    if !path.exists() {
        return Err(Error::LocalModelInferenceError(format!(
            "Model file not found: {}",
            model_path
        )));
    }

    if !path.is_file() {
        return Err(Error::LocalModelInferenceError(format!(
            "Model path is not a file: {}",
            model_path
        )));
    }

    Ok(())
}

fn print_raw_model_response(text: &str) {
    printd!("============= RAW LLAMA-CPP-2 RESPONSE (BEGIN) =============", Action);
    printd!(text, LLM);
    printd!("============== RAW LLAMA-CPP-2 RESPONSE (END) ==============", Action);
}

pub fn default_model_alias(model_path: &str) -> String {
    let path = Path::new(model_path);
    path.file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "local-model".to_string())
}
