# screen-desktop

Public SDL/LVGL host for developing Jetbeep screen applications on a desktop.
Applications are separate Rust repositories and are never stored in this repository.

## Requirements

- CMake 3.20 or newer
- A current Rust toolchain
- A C/C++ compiler

Initialize the C dependencies once:

```bash
git submodule update --init --recursive
```

## Build the simulator fixture

```bash
cmake -S . -B build \
  -DAPP_PATH="$PWD/tests/simulator-test" \
  -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j
./build/screen_desktop \
  --fs-root "$PWD/tests/simulator-test/fs" \
  --workspace "$PWD/tests/simulator-test"
```

## Build an external application

`APP_PATH` must point to a crate named `project_app`. The crate is attached to the
host Cargo workspace and inherits the SDK dependencies selected by screen-desktop.

```toml
[package]
name = "project_app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[features]
default = ["simulator"]
simulator = ["jetbeep_core/simulator"]
force-http-api = []
dev-screen-preview = []

[dependencies]
jetbeep_core = { workspace = true, features = ["platform-desktop"] }
lvgl-dsl = { workspace = true }
jkv = { workspace = true }
fonts-cache = { workspace = true }
```

The app exports these lifecycle functions:

```rust
pub unsafe fn app_main() { /* create UI and start tasks */ }
pub unsafe fn app_teardown() { /* release UI and app state */ }
pub unsafe fn app_init_screen() -> i32 { 0 }
```

Build it by absolute path:

```bash
cmake -S . -B build \
  -DAPP_PATH=/absolute/path/to/project-app \
  -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j
```

Optional app resources are resolved relative to `APP_PATH`:

- `fs/lfs1/`
- `config/profiles.json`
- `fonts/fonts.cmake`
- `simulator_config.json` or `simulator_layouts/`
- `agent_config.json`

For local SDK or service-menu development, configure with `RUST_LIBS_PATH` and
`SERVICE_MENU_PATH`. These paths are configure-time overrides and are not stored
in the repository.

Firmware images are produced separately by the managed firmware build service.
