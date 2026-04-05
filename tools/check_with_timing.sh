#!/usr/bin/env bash

set -u
LC_ALL=C

EXAMPLES=(
  collatz_1000
  control_system
  deep_taxonomy_100000
  delfour
  euler_identity
  fibonacci
  goldbach_1000
  gps
  kaprekar_6174
  matrix_mechanics
  odrl_dpv_ehds_risk_ranked
  path_discovery
  pn_junction_tunneling
  polynomial
  transistor_switch
  sudoku
)

VERBOSE=0

for arg in "$@"; do
  case "$arg" in
    --verbose) VERBOSE=1 ;;
  esac
done

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$ROOT_DIR/bin"

if [ -t 1 ] && [ "${TERM:-}" != "dumb" ] && [ -z "${NO_COLOR:-}" ]; then
  RED="\e[31m"
  GREEN="\e[32m"
  YELLOW="\e[33m"
  NORMAL="\e[0;39m"
else
  RED=''
  GREEN=''
  YELLOW=''
  NORMAL=''
fi

now_ns() {
  date +%s%N
}

run_once() {
  local binary="$1"
  local start_ns end_ns elapsed_ns rc
  start_ns="$(now_ns)"
  if [ "$VERBOSE" -eq 1 ]; then
    "$binary"
    rc=$?
  else
    "$binary" >/dev/null 2>/dev/null
    rc=$?
  fi
  end_ns="$(now_ns)"
  elapsed_ns=$((end_ns - start_ns))
  printf '%s %s\n' "$rc" "$elapsed_ns"
}

ns_to_ms() {
  awk -v ns="$1" 'BEGIN { printf "%.3f", ns / 1000000.0 }'
}

all_ok=1
total_ok=0
total_fail=0
total_missing=0
total_ns=0

for name in "${EXAMPLES[@]}"; do
  binary="$BIN_DIR/$name"
  if [ ! -x "$binary" ]; then
    printf '%b%-25s%b  %b%8s%b  %b%s%b\n' \
      "$NORMAL" "$name" "$NORMAL" \
      "$YELLOW" '-' "$NORMAL" \
      "$YELLOW" 'MISSING' "$NORMAL"
    all_ok=0
    total_missing=$((total_missing + 1))
    continue
  fi

  read -r rc elapsed_ns < <(run_once "$binary")
  if [ "$rc" -eq 0 ]; then
    ms="$(ns_to_ms "$elapsed_ns")"
    printf '%b%-25s%b  %b%8s ms%b  %b%s%b\n' \
      "$NORMAL" "$name" "$NORMAL" \
      "$YELLOW" "$ms" "$NORMAL" \
      "$GREEN" 'OK' "$NORMAL"
    total_ok=$((total_ok + 1))
    total_ns=$((total_ns + elapsed_ns))
  else
    printf '%b%-25s%b  %b%8s%b  %b%s%b\n' \
      "$NORMAL" "$name" "$NORMAL" \
      "$YELLOW" '-' "$NORMAL" \
      "$RED" 'FAIL' "$NORMAL"
    all_ok=0
    total_fail=$((total_fail + 1))
  fi
done

printf '\n'
printf '%bSummary:%b %b%d OK%b' "$NORMAL" "$NORMAL" "$GREEN" "$total_ok" "$NORMAL"
printf '  %b%d FAIL%b' "$RED" "$total_fail" "$NORMAL"
if [ "$total_missing" -gt 0 ]; then
  printf '  %b%d MISSING%b' "$YELLOW" "$total_missing" "$NORMAL"
fi
printf '  %b%s ms total%b\n' "$YELLOW" "$(ns_to_ms "$total_ns")" "$NORMAL"

if [ "$all_ok" -eq 1 ]; then
  exit 0
else
  exit 1
fi
