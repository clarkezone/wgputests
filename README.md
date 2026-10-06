# Orb mesh experiment (phase 1)

This branch opens **Mesh Orb**, a synthetic mesh visualization lab based on the
Orbital Sphere experience. Run `cargo run --locked`. Use the left panel to vary
nodes, sessions, workspaces, agents and agent states; counts animate arrivals
and departures. `H` hides controls, `Space` pauses and `P` triggers a pulse.

[Experiment design, controls and verification](docs/orb-mesh-phase1.md).

The original gallery remains available through the selector (keys 1–4).
This prototype has no herdr-mesh/RPC integration; that belongs to phase 2.

---

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
