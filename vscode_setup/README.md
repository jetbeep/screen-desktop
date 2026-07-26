# VS Code Setup Bundle

This folder contains reusable VS Code configuration for building, cleaning,
packaging, and debugging an external screen application with screen-desktop.

## Install

From the screen-desktop repository root:

```bash
mkdir -p .vscode
cp vscode_setup/tasks.json .vscode/tasks.json
cp vscode_setup/launch.json .vscode/launch.json
cp vscode_setup/settings.json .vscode/settings.json
```

The local `.vscode/` directory is ignored by Git, so machine-specific paths are
never committed.

## Configure

Edit `.vscode/settings.json`:

- `screenDesktop.appPath`: absolute path to the external `project_app` crate.
- `screenDesktop.width`: simulator window width in pixels.
- `screenDesktop.height`: simulator window height in pixels.

The app path must contain `Cargo.toml`. Optional runtime resources are loaded
from the same path (`fs/`, `simulator_config.json`, `simulator_layouts/`, and
`agent_config.json`).

## Tasks

- `Build`: configures `build/` in Debug mode and builds `screen_desktop`.
- `Build api-examples`: configures `build-api-examples/` in Debug mode and
  builds the bundled `examples/api-examples` app.
- `Clean`: removes `build/`, `build-api-examples/`, and `build-release/`.
- `Create release`: builds `build-release/` in Release mode and writes a ZIP to
  `releases/` containing the executable and available app runtime resources.

## Debug

Install the CodeLLDB extension (`vadimcn.vscode-lldb`), then select
`Debug selected app`. The launch configuration runs the `Build` task first and
starts the simulator with the selected app's filesystem and workspace paths.

Select `Debug api-examples` to build and launch the bundled API example without
changing `screenDesktop.appPath` or replacing the selected external-app build.