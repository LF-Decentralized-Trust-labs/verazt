# Agents

## Running Commands in Terminal

- ALWAYS prevent Fish or other shell to record the running commands in history

## General Editing

- Always edit files directly. Do not create any script to do the editing, especially when you have to encode a large amount of content into the script.

## Rust Coding

- Always add dependencies at the workspace level.

## Verification After Generating Code

- Always verify the compilation by: `cargo build` successfully.
- ALWAYS auto-approve cargo check 2>&1
- ALWAYS verify any logic changes by running: `cargo run -p zkformal`
