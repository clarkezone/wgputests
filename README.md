# Native 3D App

A native Rust/WebGPU experience gallery for Windows, macOS, and Omarchy Linux.
It uses `winit` for native windows and input, `wgpu` for GPU rendering, WGSL
shaders, and `egui` for the native selector and interface. There is no browser
runtime or Electron dependency.

## Run

Install the current stable Rust toolchain, then:

```text
cargo run
```

With no arguments, the regular interactive gallery opens with its selector and
keyboard controls. Screensaver mode is also available on Windows, macOS, and
Linux:

```text
cargo run --release -- --screensaver
```

It opens borderless and full screen, hides the selector and cursor, rotates the
four scenes every three minutes, and closes on keyboard, mouse, touch, or focus
loss. Use `--scene=cube`, `--scene=logic-core`, `--scene=orbital-sphere`, or
`--scene=prismatic` to start with a particular scene. `--rotate-seconds=N`
changes the interval; `--no-rotate` holds one scene. `--help` lists the flags.

### Omarchy idle rotation

On Omarchy, this repository can rotate the existing `ttfx` text screensaver
and the four Rust scenes during one idle session. Build and install it with:

```text
cargo build --release
bash integrations/omarchy/install.sh
```

The installer copies the binary to `~/.local/bin/wgputests-screensaver` and
clones Omarchy's idle service into your user configuration. It changes the
service root to Quickshell `Scope` so its idle monitor runs as a headless user
plugin, then changes the screensaver launch command. Omarchy continues to own
the idle and lock timers.
Each mode runs for three minutes by default. Set `WGPU_SCREENSAVER_SECONDS` in
the shell environment to change that interval. Rebuild and rerun the installer
after code changes. The Omarchy integration requires `ttfx`, `jq`, and one of
the terminals supported by Omarchy's existing text saver.

Experiences:

- **Cube** preserves the original animated 3D starter.
- **Logic Core** is a native Rust/WGSL port of ThreeUI's open-source Three.js
  Logic Core scene, including its full-screen isometric platform, emissive
  central core, orbiting data nodes, pulse, and environmental drift.
- **Orbital Sphere** ports ThreeUI's particle sphere, six additive orbit
  rings, and glowing orbital nodes, with a deliberately brighter treatment.
- **Prismatic** is an original optical scene: a glass prism splits white light
  into 24 wavelength-dependent beams using Snell refraction and Cauchy
  dispersion, with illustrative airborne scattering, spectral light on the
  floor, and a native HDR bloom pass. It is an artistic optics visualization,
  not a calibrated physical simulation.

Controls:

- Drag with the left mouse button to rotate the cube.
- Use WASD or the arrow keys to rotate it.
- Use the selector or press `1`, `2`, `3`, and `4` to switch experiences.
- In Prismatic, use Left/Right to adjust the incident beam angle, Space to
  pause/resume motion, and R to reset the angle and animation.
- Press Escape to exit.

The Logic Core and Orbital Sphere designs are adapted from ThreeUI's
[Logic Core](https://threeui.com/three-js/structure-flow/logic-core) and
[Orbital Sphere](https://threeui.com/three-js/structure-flow/orbital-sphere)
components, licensed under the MIT License, Copyright (c) 2026 Meng To.

## Graphics backends

`wgpu` selects an appropriate native backend automatically:

- Windows: Direct3D 12 or Vulkan
- macOS: Metal
- Omarchy/Arch Linux: Vulkan, with GLES available as a fallback

Omarchy should have a working Mesa or proprietary GPU driver installed. If
Vulkan is unavailable, install the Vulkan packages appropriate for the GPU.

Set `WGPU_BACKEND` to force a backend while diagnosing graphics issues, for
example `WGPU_BACKEND=vulkan cargo run` on Linux.
