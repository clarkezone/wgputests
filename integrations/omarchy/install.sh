#!/usr/bin/env bash

# Install the Omarchy idle rotation after building the release binary.
set -euo pipefail

repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
binary="$repo_dir/target/release/native-3d-app"
plugin_id="${USER:-$(id -un)}.idle"
plugin_dir="$HOME/.config/omarchy/plugins/$plugin_id"
config="$HOME/.config/omarchy/shell.json"
hypr_config="$HOME/.config/hypr/hyprland.lua"

[[ -x $binary ]] || {
  echo "Build first: cargo build --release" >&2
  exit 1
}
[[ -f $config ]] || {
  echo "Omarchy shell configuration was not found: $config" >&2
  exit 1
}
[[ -f $hypr_config ]] || {
  echo "Hyprland configuration was not found: $hypr_config" >&2
  exit 1
}

backup="$config.bak.$(date +%Y%m%d-%H%M%S)"
cp -a "$config" "$backup"
echo "Saved Omarchy configuration to $backup"

if [[ ! -d $plugin_dir ]]; then
  omarchy plugin clone omarchy.idle
fi

service_backup="$backup.idle-Service.qml"
cp -a "$plugin_dir/Service.qml" "$service_backup"
echo "Saved idle plugin to $service_backup"

install -Dm755 "$binary" "$HOME/.local/bin/wgputests-screensaver"
install -m755 "$repo_dir/integrations/omarchy/rotate.sh" "$plugin_dir/rotate.sh"
install -m755 "$repo_dir/integrations/omarchy/text.sh" "$plugin_dir/text.sh"

python3 - "$plugin_dir/Service.qml" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
source = path.read_text()
# User plugins are instantiated without the built-in serviceHost Item parent.
# Scope keeps the child IdleMonitor active in this headless service.
if "\nItem {\n" in source:
    source = source.replace("\nItem {\n", "\nScope {\n", 1)
elif "\nScope {\n" not in source:
    raise SystemExit("Unexpected idle service root type")
old = ' || omarchy-launch-screensaver")'
new = ' || bash \\"$HOME/.config/omarchy/plugins/${USER}.idle/rotate.sh\\"")'
if old in source:
    source = source.replace(old, new, 1)
elif new not in source:
    raise SystemExit("Could not locate the built-in screensaver launch command")
path.write_text(source)
PY

if ! grep -Fq -- '-- wgputests interactive gallery idle inhibition' "$hypr_config"; then
  hypr_backup="$hypr_config.bak.$(date +%Y%m%d-%H%M%S)"
  cp -a "$hypr_config" "$hypr_backup"
  echo "Saved Hyprland configuration to $hypr_backup"
  cat >> "$hypr_config" <<'LUA'

-- wgputests interactive gallery idle inhibition
-- Hyprland releases the inhibitor when the gallery window closes.
o.window({ title = "^Native WebGPU Experiences$" }, { idle_inhibit = "always" })
LUA
fi

hyprctl reload
hyprctl configerrors
omarchy restart shell
echo "Installed text and Rust scene rotation in $plugin_dir with gallery idle inhibition"
