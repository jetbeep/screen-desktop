use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use jetbeep_core::app_launcher::LaunchTarget;
use jetbeep_core::workq;

mod heap_track;

unsafe extern "C" {
    fn lv_timer_handler() -> u32;
}

fn lvgl_tick(task_id: workq::TaskId) {
    unsafe {
        lv_timer_handler();
    }
    workq::restart(task_id, Duration::from_millis(5), lvgl_tick);
}

const APP_NONE: i32 = -1;
const APP_SELECTED: i32 = LaunchTarget::Selected as i32;
const APP_SERVICE_MENU: i32 = LaunchTarget::ServiceMenu as i32;
static CURRENT_APP: AtomicI32 = AtomicI32::new(APP_NONE);

fn app_teardown() {
    match CURRENT_APP.swap(APP_NONE, Ordering::Relaxed) {
        APP_SERVICE_MENU => unsafe { service_menu_app::app_teardown() },
        APP_SELECTED => unsafe { project_app::app_teardown() },
        _ => {}
    }
}

fn launch_app(target: LaunchTarget) {
    let target_id = target as i32;
    if CURRENT_APP.load(Ordering::Relaxed) == target_id {
        return;
    }

    app_teardown();
    match target {
        LaunchTarget::ServiceMenu => {
            unsafe { service_menu_app::app_main() };
            CURRENT_APP.store(APP_SERVICE_MENU, Ordering::Relaxed);
        }
        _ => {
            unsafe { project_app::app_main() };
            CURRENT_APP.store(APP_SELECTED, Ordering::Relaxed);
        }
    }
}

struct CliArgs {
    fs_root: Option<String>,
    simulator_config: Option<String>,
    simulator_layout: Option<String>,
    agent_config: Option<String>,
    workspace: Option<String>,
}

fn parse_cli_args(argc: i32, argv: *const *const c_char) -> CliArgs {
    let mut result = CliArgs {
        fs_root: None,
        simulator_config: None,
        simulator_layout: None,
        agent_config: None,
        workspace: None,
    };
    if argc <= 1 || argv.is_null() {
        return result;
    }

    let args: Vec<String> = (0..argc)
        .filter_map(|index| {
            let ptr = unsafe { *argv.add(index as usize) };
            (!ptr.is_null()).then(|| {
                unsafe { CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned()
            })
        })
        .collect();

    let mut index = 1;
    while index < args.len() {
        let current = args[index].as_str();
        let value = args.get(index + 1).cloned();
        match current {
            "--fs-root" => result.fs_root = value,
            "--simulator-config" => result.simulator_config = value,
            "--simulator-layout" => result.simulator_layout = value,
            "--agent-config" => result.agent_config = value,
            "-w" | "--workspace" => result.workspace = value,
            _ => {
                if let Some(value) = current.strip_prefix("-w=") {
                    result.workspace = Some(value.to_owned());
                    index += 1;
                    continue;
                }
                for (prefix, destination) in [
                    ("--fs-root=", &mut result.fs_root),
                    ("--simulator-config=", &mut result.simulator_config),
                    ("--simulator-layout=", &mut result.simulator_layout),
                    ("--agent-config=", &mut result.agent_config),
                    ("--workspace=", &mut result.workspace),
                ] {
                    if let Some(value) = current.strip_prefix(prefix) {
                        *destination = Some(value.to_owned());
                        break;
                    }
                }
                index += 1;
                continue;
            }
        }
        if args.get(index + 1).is_some() {
            index += 1;
        }
        index += 1;
    }
    result
}

fn default_simulator_config(workspace: &std::path::Path) -> String {
    let layouts = workspace.join("simulator_layouts");
    if layouts.is_dir() {
        layouts.to_string_lossy().into_owned()
    } else {
        workspace
            .join("simulator_config.json")
            .to_string_lossy()
            .into_owned()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_event_loop(argc: i32, argv: *const *const c_char) {
    let cli = parse_cli_args(argc, argv);
    let fs_root = cli.fs_root.map(|path| {
        std::path::Path::new(&path)
            .canonicalize()
            .map(|canonical| canonical.to_string_lossy().into_owned())
            .unwrap_or(path)
    });
    let executable_dir = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| ".".to_owned());
    let workspace = cli.workspace.unwrap_or(executable_dir);
    let resolve = |path: String| {
        let path = std::path::Path::new(&path);
        if path.is_relative() {
            std::path::Path::new(&workspace)
                .join(path)
                .to_string_lossy()
                .into_owned()
        } else {
            path.to_string_lossy().into_owned()
        }
    };
    let simulator_config = cli
        .simulator_config
        .map(resolve)
        .unwrap_or_else(|| default_simulator_config(std::path::Path::new(&workspace)));
    let agent_config = resolve(
        cli.agent_config
            .unwrap_or_else(|| "agent_config.json".to_owned()),
    );

    jetbeep_core::init(fs_root.as_deref());
    if let Some(root) = fs_root.as_deref() {
        jetbeep_core::lvgl_fs_driver::init(root, None);
    }
    jetbeep_core::init_simulator(&simulator_config, cli.simulator_layout.as_deref());
    if std::path::Path::new(&agent_config).is_file() {
        jetbeep_core::init_agent(&agent_config);
    } else {
        log::info!("agent config not found; agent-backed requests are disabled");
    }
    jetbeep_core::app_launcher::set_launch_handler(launch_app);

    unsafe { project_app::app_main() };
    CURRENT_APP.store(APP_SELECTED, Ordering::Relaxed);
    lvgl_tick(workq::TASK_ID_INVALID);
    workq::run_loop();
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_request_shutdown() {
    workq::request_shutdown();
}
