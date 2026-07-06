# MakeREADME

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)
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

## Example of usage
<img src="exampleofusage.gif" alt="MakeREADME Engine Pipeline" width="100%" max-width="800px" />

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

### *1: Local LLM: integration of llama.cpp*
### *2: Provide More Apis: nvidia or other use to free apis.*
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

## Used AIs

| LLM | Usage |
| --- |  ---  |
| Copilot | Action Executer and catcher |
| Gemini | For Researchs |

