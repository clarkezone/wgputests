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

Experiences:

- **Cube** preserves the original animated 3D starter.
- **Logic Core** is a native Rust/WGSL port of ThreeUI's open-source Three.js
  Logic Core scene, including its full-screen isometric platform, emissive
  central core, orbiting data nodes, pulse, and environmental drift.
- **Orbital Sphere** ports ThreeUI's particle sphere, six additive orbit
  rings, and glowing orbital nodes, with a deliberately brighter treatment.

Controls:

- Drag with the left mouse button to rotate the cube.
- Use WASD or the arrow keys to rotate it.
- Use the selector or press `1`, `2`, and `3` to switch experiences.
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
