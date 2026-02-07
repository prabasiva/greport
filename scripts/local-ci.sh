#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DASHBOARD_DIR="$ROOT_DIR/dashboard"
PASSED=0
FAILED=0
SKIPPED=0
FAILURES=()

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

run_step() {
  local name="$1"
  shift
  printf "\n${BLUE}${BOLD}[STEP]${NC} %s\n" "$name"
  printf "%s\n" "----------------------------------------------"
  if "$@"; then
    printf "${GREEN}${BOLD}[PASS]${NC} %s\n" "$name"
    PASSED=$((PASSED + 1))
  else
    printf "${RED}${BOLD}[FAIL]${NC} %s\n" "$name"
    FAILED=$((FAILED + 1))
    FAILURES+=("$name")
  fi
}

skip_step() {
  local name="$1"
  local reason="$2"
  printf "\n${YELLOW}${BOLD}[SKIP]${NC} %s -- %s\n" "$name" "$reason"
  SKIPPED=$((SKIPPED + 1))
}

printf "${BOLD}========================================${NC}\n"
printf "${BOLD}  greport local CI${NC}\n"
printf "${BOLD}========================================${NC}\n"
printf "Root: %s\n" "$ROOT_DIR"

# ------------------------------------------------------------------
# Rust checks
# ------------------------------------------------------------------
cd "$ROOT_DIR"

run_step "cargo fmt --check" \
  cargo fmt --all -- --check

run_step "cargo clippy" \
  cargo clippy --all-targets --all-features -- -D warnings

run_step "cargo build" \
  cargo build --workspace

run_step "cargo test" \
  cargo test --workspace

# ------------------------------------------------------------------
# Frontend checks
# ------------------------------------------------------------------
cd "$DASHBOARD_DIR"

if [ ! -d "node_modules" ]; then
  run_step "npm install" npm ci
else
  skip_step "npm install" "node_modules already present"
fi

run_step "next build" \
  npm run build

run_step "vitest (unit tests)" \
  npx vitest run

# ------------------------------------------------------------------
# E2E tests (optional -- pass --e2e flag)
# ------------------------------------------------------------------
RUN_E2E=false
for arg in "$@"; do
  if [ "$arg" = "--e2e" ]; then
    RUN_E2E=true
  fi
done

if $RUN_E2E; then
  run_step "playwright install" \
    npx playwright install --with-deps chromium
  run_step "playwright e2e tests" \
    npx playwright test
else
  skip_step "playwright e2e tests" "pass --e2e flag to enable"
fi

# ------------------------------------------------------------------
# Summary
# ------------------------------------------------------------------
cd "$ROOT_DIR"
TOTAL=$((PASSED + FAILED + SKIPPED))

printf "\n${BOLD}========================================${NC}\n"
printf "${BOLD}  Summary${NC}\n"
printf "${BOLD}========================================${NC}\n"
printf "${GREEN}Passed:  %d${NC}\n" "$PASSED"
printf "${RED}Failed:  %d${NC}\n" "$FAILED"
printf "${YELLOW}Skipped: %d${NC}\n" "$SKIPPED"
printf "Total:   %d\n" "$TOTAL"

if [ ${#FAILURES[@]} -gt 0 ]; then
  printf "\n${RED}${BOLD}Failed steps:${NC}\n"
  for f in "${FAILURES[@]}"; do
    printf "  - %s\n" "$f"
  done
  printf "\n"
  exit 1
fi

printf "\n${GREEN}${BOLD}All checks passed.${NC}\n"
