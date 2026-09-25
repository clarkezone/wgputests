#!/usr/bin/env bash

# Omarchy's ttfx text saver, scoped to this terminal. The stock script kills
# every org.omarchy.screensaver process when it exits, which breaks rotation.
set -u

ttfx_pid=
window_address=
cleanup() {
  trap - EXIT HUP INT TERM QUIT
  if [[ -n $ttfx_pid ]]; then
    kill "$ttfx_pid" 2>/dev/null || true
    wait "$ttfx_pid" 2>/dev/null || true
  fi
  hyprctl eval 'hl.config({ cursor = { invisible = false } })' &>/dev/null ||
    hyprctl keyword cursor:invisible false &>/dev/null || true
  exit 0
}
trap cleanup EXIT HUP INT TERM QUIT

in_focus() {
  [[ -n $window_address ]] || return 1
  hyprctl activewindow -j 2>/dev/null |
    jq -e --arg address "$window_address" '.address == $address' >/dev/null 2>&1
}

printf '\033]11;rgb:00/00/00\007'
hyprctl eval 'hl.config({ cursor = { invisible = true } })' &>/dev/null ||
  hyprctl keyword cursor:invisible true &>/dev/null || true

# ttfx measures the terminal once at startup; wait for its fullscreen resize.
deadline=$((SECONDS + 2))
while ((SECONDS < deadline)) && [[ $(stty size 2>/dev/null) == '24 80' ]]; do
  sleep 0.02
done
window_address=$(hyprctl activewindow -j 2>/dev/null | jq -r '.address // empty')

while true; do
  ttfx -i "$HOME/.config/omarchy/branding/screensaver.txt" \
    --frame-rate 120 --canvas-width 0 --canvas-height 0 --reuse-canvas \
    --anchor-canvas c --anchor-text c --random-effect --no-eol \
    --no-restore-cursor &
  ttfx_pid=$!
  while kill -0 "$ttfx_pid" 2>/dev/null; do
    if read -r -s -n1 -t1 || ! in_focus; then
      cleanup
    fi
  done
  wait "$ttfx_pid" 2>/dev/null || true
  ttfx_pid=
done
