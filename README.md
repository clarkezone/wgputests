# Native 3D App

A small native Rust/WebGPU starter for Windows, macOS, and Omarchy Linux. It uses
`winit` for native windows and input, `wgpu` for GPU rendering, and WGSL shaders.
There is no browser runtime or Electron dependency.

## Run

Install the current stable Rust toolchain, then:

```text
cargo run
```

Controls:

- Drag with the left mouse button to rotate the cube.
- Use WASD or the arrow keys to rotate it.
- Press Escape to exit.

## Graphics backends

`wgpu` selects an appropriate native backend automatically:

- Windows: Direct3D 12 or Vulkan
- macOS: Metal
- Omarchy/Arch Linux: Vulkan, with GLES available as a fallback

Omarchy should have a working Mesa or proprietary GPU driver installed. If
Vulkan is unavailable, install the Vulkan packages appropriate for the GPU.

Set `WGPU_BACKEND` to force a backend while diagnosing graphics issues, for
example `WGPU_BACKEND=vulkan cargo run` on Linux.
