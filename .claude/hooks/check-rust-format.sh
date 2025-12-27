#!/bin/bash

# Read the hook input JSON from stdin
input=$(cat)

# Extract the file path from tool_input (Edit/Write use file_path, Serena uses relative_path)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // .tool_input.relative_path // empty')

# Only process .rs files
if [[ "$file_path" == *.rs ]]; then
  echo "Checking Rust formatting for: $file_path"
  cd "$CLAUDE_PROJECT_DIR" || exit 1
  just format
else
  # Skip non-Rust files silently
  exit 0
fi
