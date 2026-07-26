# screen-desktop

Public SDL/LVGL host for developing Jetbeep screen applications on a desktop.
Applications are separate Rust repositories and are never stored in this repository.

## Requirements

- CMake 3.20 or newer
- A current Rust toolchain
- A C/C++ compiler
- VS Code with the CodeLLDB extension (`vadimcn.vscode-lldb`)

Initialize the C dependencies once:

```bash
git submodule update --init --recursive
```

## VS Code setup

Ready-to-install tasks, settings, and CodeLLDB launch configurations are in
`vscode_setup/`. Install them into the ignored local `.vscode/` directory:

```bash
mkdir -p .vscode
cp vscode_setup/*.json .vscode/
```

Set `screenDesktop.appPath`, `screenDesktop.width`, and
`screenDesktop.height` in `.vscode/settings.json`. The app path must be
absolute and must contain the app's `Cargo.toml`.

Use these VS Code tasks and launch configurations for routine development.
They provide the required CMake options, build directories, and runtime paths;
do not invoke CMake or `screen_desktop` manually.

## Build and debug

For the selected app configured by `screenDesktop.appPath`:

1. Run **Tasks: Run Build Task** and select **Build** (or press
   <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>B</kbd>).
2. Open **Run and Debug**, select **Debug selected app**, and start debugging.
   The launch configuration runs the **Build** task automatically.

Bundled examples are available through the same workflow:

- `examples/hello_world`: set `screenDesktop.appPath` to its absolute path,
  then use **Build** or **Debug selected app**.
- `examples/api-examples`: run the **Build api-examples** task or select
  **Debug api-examples**. This does not change the selected external app build.

Other provided tasks are **Clean**, which removes generated build directories,
and **Create release**, which creates a Release build and packages the selected
app in `releases/`.

See `vscode_setup/README.md` for the complete task and release workflow.

## External application contract

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
