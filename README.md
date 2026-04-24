# MakeREADME

![Rust](https://img.shields.io/badge/Rust-2026_edition-orange?logo=rust)
![CLI](https://img.shields.io/badge/App-CLI-blue)
![Status](https://img.shields.io/badge/Status-Active-success)
![License](https://img.shields.io/badge/License-MIT-lightgrey)

### makereadme is basicly a system orchestrator, thats creates README.md files for your project.

## How it's works?

### When you give some infos about your project to *makereadme* thats uses some tags to read, think, write about your project.
```text
Examples:

Given informations:

- path: home/user/desktop/calculator/
- output name: spicyreadme
- llm type: GEMINI // or other apis that *makereadme* haves.

in backend side we sending a system prompt to LLM and some instructions about how to act,
then we checking your projects path directly and sending that treeview to LLM

and our cheff is starts to cook!

```

## How to run from source code?

### Install project
```bash
git clone https://github.com/CSDC-K/Makereadme.git
```
### Build with rust
```bash
cargo build --release
```
### Run compiled version
```bash
cargo run --release
```

## What is the future features?

### *1: LLAMA_CPP_2 for local models*
### *2: New apis (nvidia and more will coming soon)*
### *3: Improvable system prompt (user can add some comments into prompt via CLI)*

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

-- Copilot: for code review and basic changes (tab + action executer)
-- Gemini: for researchs (not for codes)
