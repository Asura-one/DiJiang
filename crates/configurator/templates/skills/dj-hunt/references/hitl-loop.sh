#!/bin/bash
# hitl-loop.sh — Human-In-The-Loop debug cycle
CMD="$@"
while true; do
  echo "=== Running: $CMD ==="
  eval "$CMD"
  echo "Exit: $?"
  echo "[n]ext [q]uit [d]iff"
  read -r a
  case "$a" in n|N) read -p "Fix: " f;; d|D) git diff;; q|Q) exit 0;; esac
done