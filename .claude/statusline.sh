#!/bin/bash
# Claude Code Status Line
# =======================
# Model | dir (git branch) | context bar % | tokens | 5h/7d rate limits
#
# Built per current statusline docs (code.claude.com/docs/en/statusline):
# - Single jq invocation (script runs frequently; keep it fast)
# - rate_limits.five_hour / seven_day (Pro/Max; absent until first API response)
# - Null-safe fallbacks everywhere
#
# Install in ~/.claude/settings.json:
#   "statusLine": { "type": "command", "command": "~/.claude/statusline.sh" }

input=$(cat)

command -v jq &>/dev/null || { echo "[jq not installed]"; exit 0; }

# --- Parse everything in ONE jq call -----------------------------------------
# Output: model|dir|pct|in_tokens|out_tokens|ctx_size|5h%|5h_reset|7d%|7d_reset
IFS='|' read -r MODEL DIR PCT TOK_IN TOK_OUT CTX FIVE_H FIVE_RESET WEEK WEEK_RESET <<< "$(
  echo "$input" | jq -r '[
    (.model.display_name // .model.id // "Claude"),
    (.workspace.current_dir // .cwd // ""),
    ((.context_window.used_percentage // 0) | floor),
    (.context_window.total_input_tokens // 0),
    (.context_window.total_output_tokens // 0),
    (.context_window.context_window_size // 200000),
    (.rate_limits.five_hour.used_percentage // "" | tostring),
    (.rate_limits.five_hour.resets_at // "" | tostring),
    (.rate_limits.seven_day.used_percentage // "" | tostring),
    (.rate_limits.seven_day.resets_at // "" | tostring)
  ] | join("|")'
)"

# --- Colors -------------------------------------------------------------------
RESET='\033[0m'; DIM='\033[2m'; CYAN='\033[36m'
GREEN='\033[32m'; YELLOW='\033[33m'; RED='\033[31m'; MAGENTA='\033[35m'

# Color by threshold: <50 green, <80 yellow, else red
pct_color() {
  local p=${1%.*}
  if   (( p < 50 )); then printf '%b' "$GREEN"
  elif (( p < 80 )); then printf '%b' "$YELLOW"
  else                    printf '%b' "$RED"; fi
}

# Human-readable tokens (12k, 200k)
fmt_tokens() {
  local t=$1
  if (( t >= 1000 )); then
    awk -v t="$t" 'BEGIN{printf "%.0fk", t/1000}'
  else
    printf '%s' "$t"
  fi
}

# "↺14:00" if reset is within 24h, else weekday ("↺Mon"). macOS & Linux date.
fmt_reset() {
  local epoch=$1 now diff
  [[ -z "$epoch" || "$epoch" == "null" ]] && return
  now=$(date +%s); diff=$(( epoch - now ))
  if (( diff < 86400 )); then
    date -r "$epoch" +%H:%M 2>/dev/null || date -d "@$epoch" +%H:%M 2>/dev/null
  else
    date -r "$epoch" +%a 2>/dev/null || date -d "@$epoch" +%a 2>/dev/null
  fi
}

# --- Context bar (10 chars) ----------------------------------------------------
BAR_W=10
FILLED=$(( PCT * BAR_W / 100 )); (( FILLED > BAR_W )) && FILLED=$BAR_W
BAR=""
(( FILLED > 0 )) && { printf -v f "%${FILLED}s" ""; BAR=${f// /█}; }
(( FILLED < BAR_W )) && { printf -v e "%$((BAR_W - FILLED))s" ""; BAR=${BAR}${e// /░}; }
CTX_COLOR=$(pct_color "$PCT")

# --- Tokens --------------------------------------------------------------------
TOK_USED=$(fmt_tokens $(( TOK_IN + TOK_OUT )))
TOK_MAX=$(fmt_tokens "$CTX")

# --- Git branch ----------------------------------------------------------------
GIT=""
if [[ -n "$DIR" ]] && command -v git &>/dev/null; then
  BRANCH=$(git -C "$DIR" branch --show-current 2>/dev/null)
  [[ -n "$BRANCH" ]] && GIT=" ${DIM}(${BRANCH})${RESET}"
fi

# --- Rate limits (absent for non-subscribers / before first API response) -------
LIMITS=""
if [[ -n "$FIVE_H" ]]; then
  P=${FIVE_H%.*}; R=$(fmt_reset "$FIVE_RESET")
  LIMITS+="$(pct_color "$P")5h ${P}%${RESET}${R:+ ${DIM}↺${R}${RESET}}"
fi
if [[ -n "$WEEK" ]]; then
  P=${WEEK%.*}; R=$(fmt_reset "$WEEK_RESET")
  [[ -n "$LIMITS" ]] && LIMITS+="  "
  LIMITS+="$(pct_color "$P")7d ${P}%${RESET}${R:+ ${DIM}↺${R}${RESET}}"
fi

# --- Output ----------------------------------------------------------------------
SEP="${DIM}│${RESET}"
LINE="${MAGENTA}${MODEL}${RESET} ${SEP} ${CYAN}${DIR##*/}${RESET}${GIT} ${SEP} ${CTX_COLOR}${BAR}${RESET} ${CTX_COLOR}${PCT}%${RESET} ${DIM}${TOK_USED}/${TOK_MAX}${RESET}"
[[ -n "$LIMITS" ]] && LINE+=" ${SEP} ${LIMITS}"

echo -e "$LINE"