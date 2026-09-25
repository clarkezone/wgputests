#!/usr/bin/env bash

# Run by the cloned Omarchy idle service for one idle cycle. Keep a screensaver
# window open while switching so Omarchy's existing lock timer stays armed.
set -euo pipefail

plugin_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
app=${WGPU_SCREENSAVER_BIN:-"$HOME/.local/bin/wgputests-screensaver"}
interval=${WGPU_SCREENSAVER_SECONDS:-180}
old_address=
current_address=

[[ -x $app && $interval =~ ^[1-9][0-9]*$ ]] || exit 1

idle_cycle_active() {
  omarchy-shell idle status 2>/dev/null |
    jq -e '.enabled and .inIdleCycle and .screensaverStarted' >/dev/null 2>&1
}

window_exists() {
  hyprctl clients -j 2>/dev/null |
    jq -e --arg address "$1" 'any(.[]; .address == $address)' >/dev/null 2>&1
}

close_previous_window() {
  local pid count
  [[ -n $old_address ]] || return 0

  # Both viewers exit when another window takes focus. Give them a moment to
  # do that themselves, then terminate only a process owning one window.
  sleep 2
  window_exists "$old_address" || return 0
  pid=$(hyprctl clients -j | jq -r --arg address "$old_address" \
    '.[] | select(.address == $address and .class == "org.omarchy.screensaver") | .pid')
  [[ $pid =~ ^[1-9][0-9]*$ ]] || return 1
  count=$(hyprctl clients -j | jq -r --argjson pid "$pid" \
    '[.[] | select(.pid == $pid)] | length')
  [[ $count == 1 ]] || return 1
  kill "$pid"
}

find_new_window() {
  hyprctl clients -j 2>/dev/null |
    jq -r --arg old "$old_address" \
      '[.[] | select(.class == "org.omarchy.screensaver" and .address != $old)] | .[-1].address // empty'
}

launch_text() {
  local terminal
  terminal=$(xdg-terminal-exec --print-id)
  case $terminal in
    *Alacritty*)
      alacritty --class=org.omarchy.screensaver \
        --config-file "$OMARCHY_PATH/default/alacritty/screensaver.toml" \
        -e bash "$plugin_dir/text.sh" >/dev/null 2>&1 &
      ;;
    *ghostty*)
      ghostty --class=org.omarchy.screensaver \
        --config-file="$OMARCHY_PATH/default/ghostty/screensaver" --font-size=18 \
        -e bash "$plugin_dir/text.sh" >/dev/null 2>&1 &
      ;;
    *foot*)
      foot --app-id=org.omarchy.screensaver \
        --config="$OMARCHY_PATH/default/foot/screensaver.ini" \
        -e bash "$plugin_dir/text.sh" >/dev/null 2>&1 &
      ;;
    *kitty*)
      kitty --class=org.omarchy.screensaver --override font_size=18 \
        --override window_padding_width=0 \
        -e bash "$plugin_dir/text.sh" >/dev/null 2>&1 &
      ;;
    *) return 1 ;;
  esac
}

launch_mode() {
  local mode=$1 deadline
  idle_cycle_active || return 1

  if [[ $mode == text ]]; then
    launch_text
  else
    "$app" --screensaver=org.omarchy.screensaver --scene="$mode" --no-rotate \
      >/dev/null 2>&1 &
  fi

  deadline=$((SECONDS + 20))
  current_address=
  while ((SECONDS < deadline)); do
    idle_cycle_active || return 1
    current_address=$(find_new_window)
    [[ -n $current_address ]] && return 0
    sleep 0.2
  done
  return 1
}

while true; do
  for mode in text cube logic-core orbital-sphere prismatic; do
    launch_mode "$mode" || exit 1

    # The new window is mapped before the previous one closes. This prevents
    # the idle service from interpreting a rotation as user activity.
    close_previous_window || exit 1
    old_address=$current_address

    for ((second = 0; second < interval; second++)); do
      sleep 1
      idle_cycle_active || exit 0
      window_exists "$current_address" || exit 0
    done
  done
done
