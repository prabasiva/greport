#!/usr/bin/env bash
#
# greport API test script
# Tests all endpoints against a running API server.
#
# Usage:
#   ./scripts/test-api.sh                          # uses defaults
#   ./scripts/test-api.sh -b http://localhost:9423  # custom base URL
#   ./scripts/test-api.sh -r rust-lang/rust         # custom repo
#   ./scripts/test-api.sh -v                        # verbose output

set -euo pipefail

# --- defaults ---
BASE_URL="${GREPORT_API_URL:-http://localhost:9423}"
REPO="${GREPORT_REPO:-prabasiva/greport}"
VERBOSE=false

# --- parse flags ---
while getopts "b:r:vh" opt; do
  case $opt in
    b) BASE_URL="$OPTARG" ;;
    r) REPO="$OPTARG" ;;
    v) VERBOSE=true ;;
    h)
      echo "Usage: $0 [-b base_url] [-r owner/repo] [-v] [-h]"
      echo "  -b  API base URL (default: http://localhost:9423)"
      echo "  -r  Repository as owner/repo (default: prabasiva/greport)"
      echo "  -v  Verbose output (show response bodies)"
      echo "  -h  Show this help"
      exit 0
      ;;
    *) exit 1 ;;
  esac
done

OWNER="${REPO%%/*}"
REPO_NAME="${REPO##*/}"
API="${BASE_URL}/api/v1"

# --- counters ---
PASS=0
FAIL=0
SKIP=0
TOTAL=0

# --- helpers ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_header() {
  echo ""
  echo -e "${CYAN}=== $1 ===${NC}"
}

run_test() {
  local description="$1"
  local url="$2"
  local expected_status="${3:-200}"

  TOTAL=$((TOTAL + 1))

  # Make the request, capture status code and body
  local tmpfile
  tmpfile=$(mktemp)
  local http_code
  http_code=$(curl -s -o "$tmpfile" -w "%{http_code}" "$url" 2>/dev/null) || {
    echo -e "  ${RED}FAIL${NC} $description"
    echo "       Could not connect to $url"
    FAIL=$((FAIL + 1))
    rm -f "$tmpfile"
    return
  }

  if [ "$http_code" = "$expected_status" ]; then
    echo -e "  ${GREEN}PASS${NC} $description (HTTP $http_code)"
    PASS=$((PASS + 1))
  else
    echo -e "  ${RED}FAIL${NC} $description (expected $expected_status, got $http_code)"
    FAIL=$((FAIL + 1))
  fi

  if [ "$VERBOSE" = true ]; then
    echo "       URL: $url"
    # Pretty-print JSON if python3 is available, otherwise raw
    if command -v python3 &>/dev/null; then
      python3 -m json.tool "$tmpfile" 2>/dev/null | head -30 | sed 's/^/       /'
    else
      head -c 500 "$tmpfile" | sed 's/^/       /'
      echo ""
    fi
  fi

  rm -f "$tmpfile"
}

# --- check server is reachable ---
echo -e "${CYAN}greport API Test Suite${NC}"
echo "Base URL: $BASE_URL"
echo "Repo:     $OWNER/$REPO_NAME"
echo ""

echo -n "Checking server connectivity... "
if curl -s --connect-timeout 5 "$BASE_URL/health" > /dev/null 2>&1; then
  echo -e "${GREEN}OK${NC}"
else
  echo -e "${RED}FAILED${NC}"
  echo "Error: Cannot connect to $BASE_URL"
  echo "Make sure the API server is running:"
  echo "  cargo run --bin greport-api"
  exit 1
fi

# =============================================================================
# Health Check
# =============================================================================
log_header "Health Check"
run_test "GET /health" "$BASE_URL/health"

# =============================================================================
# Issues
# =============================================================================
log_header "Issues"
run_test "GET issues (open)"          "$API/repos/$OWNER/$REPO_NAME/issues"
run_test "GET issues (closed)"        "$API/repos/$OWNER/$REPO_NAME/issues?state=closed"
run_test "GET issues (all)"           "$API/repos/$OWNER/$REPO_NAME/issues?state=all"
run_test "GET issues (paginated)"     "$API/repos/$OWNER/$REPO_NAME/issues?page=1&per_page=5"
run_test "GET issues metrics"         "$API/repos/$OWNER/$REPO_NAME/issues/metrics"
run_test "GET issues velocity"        "$API/repos/$OWNER/$REPO_NAME/issues/velocity"
run_test "GET velocity (daily)"       "$API/repos/$OWNER/$REPO_NAME/issues/velocity?period=day&last=7"
run_test "GET velocity (monthly)"     "$API/repos/$OWNER/$REPO_NAME/issues/velocity?period=month&last=6"
run_test "GET stale issues"           "$API/repos/$OWNER/$REPO_NAME/issues/stale"
run_test "GET stale issues (90 days)" "$API/repos/$OWNER/$REPO_NAME/issues/stale?days=90"

# =============================================================================
# Pull Requests
# =============================================================================
log_header "Pull Requests"
run_test "GET pulls (open)"       "$API/repos/$OWNER/$REPO_NAME/pulls"
run_test "GET pulls (closed)"     "$API/repos/$OWNER/$REPO_NAME/pulls?state=closed"
run_test "GET pulls (all)"        "$API/repos/$OWNER/$REPO_NAME/pulls?state=all"
run_test "GET pulls (paginated)"  "$API/repos/$OWNER/$REPO_NAME/pulls?page=1&per_page=5"
run_test "GET pull metrics"       "$API/repos/$OWNER/$REPO_NAME/pulls/metrics"

# =============================================================================
# Releases
# =============================================================================
log_header "Releases"
run_test "GET releases"           "$API/repos/$OWNER/$REPO_NAME/releases"
run_test "GET releases (limit 5)" "$API/repos/$OWNER/$REPO_NAME/releases?per_page=5"

# =============================================================================
# Contributors
# =============================================================================
log_header "Contributors"
run_test "GET contributors"                "$API/repos/$OWNER/$REPO_NAME/contributors"
run_test "GET contributors (by PRs)"       "$API/repos/$OWNER/$REPO_NAME/contributors?sort_by=prs"
run_test "GET contributors (limit 5)"      "$API/repos/$OWNER/$REPO_NAME/contributors?limit=5"

# =============================================================================
# SLA
# =============================================================================
log_header "SLA"
run_test "GET SLA report"             "$API/repos/$OWNER/$REPO_NAME/sla"
run_test "GET SLA (custom thresholds)" "$API/repos/$OWNER/$REPO_NAME/sla?response_hours=8&resolution_hours=72"

# =============================================================================
# Error Cases
# =============================================================================
log_header "Error Handling"
run_test "GET nonexistent route (404)"       "$BASE_URL/api/v1/nonexistent" "404"
run_test "GET burndown without milestone"    "$API/repos/$OWNER/$REPO_NAME/issues/burndown" "400"

# =============================================================================
# Summary
# =============================================================================
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}Test Summary${NC}"
echo -e "${CYAN}========================================${NC}"
echo -e "  Total:   $TOTAL"
echo -e "  ${GREEN}Passed:  $PASS${NC}"
if [ "$FAIL" -gt 0 ]; then
  echo -e "  ${RED}Failed:  $FAIL${NC}"
else
  echo -e "  Failed:  $FAIL"
fi
if [ "$SKIP" -gt 0 ]; then
  echo -e "  ${YELLOW}Skipped: $SKIP${NC}"
fi
echo ""

if [ "$FAIL" -gt 0 ]; then
  echo -e "${RED}Some tests failed.${NC}"
  exit 1
else
  echo -e "${GREEN}All tests passed.${NC}"
  exit 0
fi
