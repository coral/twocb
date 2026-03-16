# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is twocb

A real-time pattern execution engine for volumetric LED installations. Rust core with V8 JavaScript for hot-reloadable patterns, audio-reactive at 100+ FPS. Inspired by Pixelblaze but targeting higher-performance hardware.

## Build & Run

```bash
cargo build                                    # builds default member (twocb)
cargo build --release
cargo run -- --config files/config.json        # run main app
RUST_LOG=debug cargo run                       # with logging
cargo test                                     # run tests
cargo build -p cubeclient --target=armv7-unknown-linux-gnueabihf  # ARM build
```

Workspace has two crates: `twocb` (main app, default member) and `cubeclient` (ARM LED output client). Edition is 2024, resolver v2.

## Architecture

### Data Flow

```
Audio (cpal) → Analysis (aubio tempo/onset + rustchord colorchord)
                              ↓
Producer (100 FPS) → Frame (timing, audio data, pixel mapping)
                              ↓
Compositor → Links (layers) → Steps (patterns) → Blend modes
                              ↓
OutputManager → OPC over TCP → LED hardware
```

### Engine System

Two pattern engine types behind the `Pattern` trait (`engines/pattern.rs`):

- **DynamicEngine** (`engines/dynamic_engine.rs`): V8 JavaScript. Each pattern gets its own OS thread + V8 isolate. Hot-reloads on file change via `notify`. Patterns live in `files/dynamic/*.js` with `files/support/global.js` providing utilities.
- **RSEngine** (`engines/rs_engine/`): Built-in Rust patterns (strobe, foldeddemo). No hot reload.

JS patterns follow Pixelblaze conventions: `beforeRender(frame, delta)` + `render3D(index, x, y, z)`. Parameters discovered by function name prefixes (`sliderX`, `toggleX`, `hsvPickerX`, `rgbPickerX`, `numberX`).

### Compositor

`layers/compositor.rs` orchestrates rendering. Links (layers) contain steps (patterns). Each step has a blend mode (Add, Subtract, Screen). Patterns execute in parallel via tokio tasks, results are blended per-link then across links.

### Threading Model

- **tokio** multi-thread runtime for async (frame timing, compositor, output)
- **Dedicated OS threads** per V8 pattern isolate, communicating via crossbeam channels
- **std::thread** for actix-web API server
- **spawn_blocking** for aubio audio analysis

### State & Persistence

sled embedded DB (`data.rs`) stores pattern state, layer configs, and global settings. Trees: `state`, `layers`, `global`.

### API

actix-web server (default `127.0.0.1:3030`). Endpoints for layers CRUD, pattern state, render order, opacity, and parameter get/set.

### Output

`Vector4<f64>` RGBA (0.0-1.0) throughout the pipeline, converted to u8 RGB at OPC output boundary. OPC protocol over TCP sockets.

## Key Files

| File | Purpose |
|------|---------|
| `twocb/src/main.rs` | Bootstrap, channel setup, render loop |
| `twocb/src/controller.rs` | Pattern instantiation, link management |
| `twocb/src/engines/dynamic_engine.rs` | V8 integration (829 lines, most complex file) |
| `twocb/src/layers/compositor.rs` | Multi-pattern blending & render orchestration |
| `twocb/src/producer.rs` | Frame timing & generation |
| `twocb/src/world_state.rs` | WorldStateBuffer macro for V8 interop |
| `twocb/src/audio/` | cpal input, aubio processing, rustchord colorchord |
| `files/config.json` | Runtime config (fps, endpoints, audio, API) |
| `files/dynamic/*.js` | User JS patterns (hot-reloadable) |
| `files/support/global.js` | Shared JS utilities (color, waveforms) |
| `files/mappings/v6.json` | 3D pixel coordinates and normals |

## Conventions

- Pixel data is `Vec<Vector4<f64>>` (RGBA, 0.0-1.0 range) everywhere except final output
- Patterns return magenta `[1.0, 0.0, 1.0, 1.0]` on error for visual debugging
- V8 buffers are currently JSON-serialized per frame (zero-copy is a goal, not yet implemented)
- Config loaded from JSON file path passed via `--config` CLI arg
