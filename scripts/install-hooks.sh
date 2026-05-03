#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

cd "$REPO_ROOT"
chmod +x .githooks/pre-push
git config core.hooksPath .githooks

echo "Git hooks installed. core.hooksPath -> .githooks"
