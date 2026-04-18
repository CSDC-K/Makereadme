# MakeREADME

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)
![CLI](https://img.shields.io/badge/App-CLI-blue)
![Status](https://img.shields.io/badge/Status-Active-success)
![License](https://img.shields.io/badge/License-MIT-lightgrey)

MakeREADME is a Rust CLI tool that generates `README.md` files by analyzing a project and running an agentic action loop (`THINK`, `READ`, `WRITE`, `EXIT`) with LLM providers.

## Features

- Multi-provider support: `GEMINI`, `GROQ`, `LLMAPI`, `LOCAL (Ollama)`, `NVIDIA` (WIP)
- Agentic workflow with controlled action parsing and execution
- Project tree snapshot + selective file reading for context
- Environment-based config loading and saving (`.env`)
- Local model parameters for Ollama (`temperature`, `top_k`, `top_p`)
- Colored debug/action logs for easier tracing

## Tech Stack

- Rust 2024
- Async runtime: Tokio
- HTTP: reqwest
- Serialization: serde / serde_json
- CLI prompts: inquire

## Quick Start

### 1. Build

```bash
cargo build --release
```

### 2. Run

```bash
cargo run
```

### 3. Optional `.env`

```env
LLM_TYPE=GEMINI
LLM_MODEL=gemini-2.5-flash
API_KEY=your_api_key_here
```

## Local (Ollama)

- Start Ollama server (default: `127.0.0.1:11434`)
- Select `LOCAL` in the CLI
- Provide gateway URL and sampling params when prompted
- The tool fetches models via Ollama API and runs the same agentic loop locally

## Project Structure

```text
src/
  main.rs
  apis/
    api_lib.rs
    gemini.rs
    groq.rs
    llmapi.rs
    nvidia.rs
  local/
    local.rs
    ollama.rs
  libs/
    action_executer.rs
    build.rs
    debug.rs
    errors.rs
    memory.rs
    prompt.rs
```

## Notes

- For stable action-format behavior with local models, use low sampling values (e.g. `temperature=0.2`, `top_p=0.9`, `top_k=40`).
- If a model returns plain prose instead of action tags, runtime guards now request strict tagged output.

## License

MIT
