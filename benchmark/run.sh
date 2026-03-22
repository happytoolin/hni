#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
NODE_CANDIDATES=()

while IFS= read -r line; do
  [[ -n "$line" ]] && NODE_CANDIDATES+=("$line")
done < <(which -a node 2>/dev/null || true)

REAL_NODE=""
for candidate in "${NODE_CANDIDATES[@]}"; do
  if [[ "$candidate" != *"/.local/bin/node" ]]; then
    REAL_NODE="$candidate"
    break
  fi
done

if [[ -z "$REAL_NODE" && "${#NODE_CANDIDATES[@]}" -gt 0 ]]; then
  REAL_NODE="${NODE_CANDIDATES[0]}"
fi

if [[ -z "$REAL_NODE" ]]; then
  echo "Could not find a real node binary on PATH" >&2
  exit 1
fi

exec "$REAL_NODE" "$SCRIPT_DIR/run.mjs" "$@"
