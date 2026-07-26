use core::ffi::c_void;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;

use jetbeep_core::bus::{self, JkvValue, ServerRequestParams};
use jetbeep_core::proto::bus::LockStatus;
use jetbeep_core::{executor, workq};
use lvgl_dsl::lv_font_t;
use lvgl_dsl::prelude::*;

const BACKGROUND: u32 = 0xF4F6F8;
const CARD: u32 = 0xFFFFFF;
const PRIMARY: u32 = 0x0F4C81;
const TEXT: u32 = 0x1F2937;
const MUTED: u32 = 0x5B6472;

static STATUS_LABEL: AtomicUsize = AtomicUsize::new(0);
static CLOCK_LABEL: AtomicUsize = AtomicUsize::new(0);
static DATA_LABEL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" {
    fn rust_request_shutdown();
}

fn update_label(handle: &AtomicUsize, text: &str) {
    let pointer = handle.load(Ordering::Relaxed);
    if pointer == 0 {
        return;
    }

    let label = unsafe { Label::from_raw(pointer as *mut _) };
    label.text(text);
}

fn set_status(text: &str) {
    update_label(&STATUS_LABEL, text);
}

fn format_utc_time() -> String {
    let total_seconds = (jetbeep_core::unix_time() / 1_000_000) as i64;
    let day_seconds = total_seconds.rem_euclid(86_400);
    let hour = day_seconds / 3_600;
    let minute = (day_seconds % 3_600) / 60;
    let second = day_seconds % 60;
    format!("{:02}:{:02}:{:02} UTC", hour, minute, second)
}

async fn clock_task() {
    loop {
        update_label(&CLOCK_LABEL, &format_utc_time());
        workq::delay(Duration::from_secs(1)).await;
    }
}

async fn load_data_task() {
    match jetbeep_core::jkv_file::read_value("lfs1/app/data/test.jkv").await {
        Ok(value) => match (
            value.get_key("title"),
            value.get_key("message"),
            value.get_key("sample_number"),
        ) {
            (
                Some(JkvValue::String(title)),
                Some(JkvValue::String(message)),
                Some(JkvValue::Int(number)),
            ) => update_label(
                &DATA_LABEL,
                &format!("{title}: {message} (sample_number = {number})"),
            ),
            _ => update_label(&DATA_LABEL, &format!("Decoded JKV: {value:?}")),
        },
        Err(error) => update_label(&DATA_LABEL, &format!("JKV read failed: {error}")),
    }
}

fn on_open_lock(_: Event) {
    set_status("Opening lock 0:1...");
    executor::run(async {
        match bus::lock_open(0, 1).await {
            Ok(()) => set_status("Lock 0:1 opened"),
            Err(error) => set_status(&format!("Open lock failed: {error}")),
        }
    });
}

fn on_get_status(_: Event) {
    set_status("Reading status for lock 0:1...");
    executor::run(async {
        match bus::lock_statuses_get(0).await {
            Ok(statuses) => {
                let state = statuses.get(1).copied().unwrap_or(LockStatus::Disabled);
                let state = match state {
                    LockStatus::Opened => "OPEN",
                    LockStatus::Closed => "CLOSED",
                    LockStatus::Disabled => "DISABLED",
                };
                set_status(&format!("Lock 0:1 status: {state}"));
            }
            Err(error) => set_status(&format!("Get status failed: {error}")),
        }
    });
}

fn on_server_request(_: Event) {
    set_status("Checking server connectivity...");
    executor::run(async {
        let result = bus::server_request_ex(
            JkvValue::Collection(vec![]),
            ServerRequestParams {
                request_type: 104,
                timeout_ms: 10_000,
                ..Default::default()
            },
        )
        .await;

        match result {
            Ok(response) => set_status(&format!(
                "Server request succeeded: HTTP {}",
                response.status_code
            )),
            Err(error) => set_status(&format!("Server request failed: {error}")),
        }
    });
}

fn on_service_menu(_: Event) {
    set_status("Launching service menu...");
    jetbeep_core::app_launcher::request_service_menu();
}

fn on_get_battery_percent(_: Event) {
    set_status("Reading battery level...");
    executor::run(async {
        match bus::battery_get_info().await {
            Ok(info) => set_status(&format!("Battery level: {}%", info.percent)),
            Err(error) => set_status(&format!("Get battery level failed: {error}")),
        }
    });
}

fn on_get_settings_crc(_: Event) {
    set_status("Reading settings CRC hash...");
    executor::run(async {
        match bus::get_device_settings().await {
            Ok(settings) => set_status(&format!(
                "Settings CRC hash: 0x{:08X}",
                settings.crc32_hash
            )),
            Err(error) => set_status(&format!("Get settings CRC failed: {error}")),
        }
    });
}

fn on_exit(_: Event) {
    set_status("Exiting...");
    unsafe { rust_request_shutdown() };
}

fn add_button(parent: &Obj, text: &str, callback: fn(Event), primary: bool) {
    let button = Button::new(parent);
    button
        .width(Size::Px(248))
        .height(Size::Px(52))
        .bg_color(Color::hex(if primary { PRIMARY } else { CARD }))
        .border_width(if primary { 0 } else { 2 })
        .border_color(Color::hex(0xD5DCE4))
        .radius(8)
        .on_click(callback)
        .text(text)
        .text_color(if primary {
            Color::white()
        } else {
            Color::hex(TEXT)
        });
}

fn app_font() -> Font {
    let fallback = Font::montserrat_20();
    let proxy = fonts_cache::get(
        "poppins_regular_22",
        fallback.as_ptr() as *const c_void,
    );
    unsafe { Font::from_raw(proxy as *const lv_font_t) }
}

pub unsafe fn app_main() {
    fonts_cache::init("J:app/fonts/", 32 * 1024);
    let font = app_font();
    let screen = Screen::active();
    let root = Obj::new(&screen);
    root.width(Size::Pct(100))
        .height(Size::Pct(100))
        .bg_color(Color::hex(BACKGROUND))
        .border_width(0)
        .pad_all(18)
        .text_font(&font)
        .flex_col()
        .flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
        .gap(8)
        .remove_flag(LvObjFlag::SCROLLABLE);

    let title = Label::new(&root);
    title
        .text("Screen API examples")
        .text_font(&font)
        .text_color(Color::hex(TEXT));

    let clock = Label::new(&root);
    clock.text(&format_utc_time()).text_color(Color::hex(MUTED));
    CLOCK_LABEL.store(clock.raw_ptr(), Ordering::Relaxed);

    let content = Obj::new(&root);
    content
        .width(Size::Pct(100))
        .height(Size::Px(248))
        .bg_opa(0)
        .border_width(0)
        .flex_row()
        .flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
        .gap(24)
        .remove_flag(LvObjFlag::SCROLLABLE);

    if let Ok(source) = ImageSrc::file("J:app/img/test-image.bin") {
        Image::new(&content).set_src(source);
    }

    let actions = Obj::new(&content);
    actions
        .width(Size::Px(504))
        .height(Size::Px(232))
        .bg_opa(0)
        .border_width(0)
        .pad_all(0)
        .flex_row_wrap()
        .gap(8)
        .remove_flag(LvObjFlag::SCROLLABLE);

    add_button(&actions, "Open lock 0:1", on_open_lock, true);
    add_button(&actions, "Get lock 0:1 status", on_get_status, false);
    add_button(&actions, "Make server request", on_server_request, false);
    add_button(&actions, "Launch service menu", on_service_menu, false);
    add_button(&actions, "Get battery percent", on_get_battery_percent, false);
    add_button(&actions, "Get settings CRC hash", on_get_settings_crc, false);
    add_button(&actions, "Exit", on_exit, false);

    let data = Label::new(&root);
    data.width(Size::Pct(90))
        .long_mode(LvLabelLongMode::Wrap)
        .text("Loading lfs1/app/data/test.jkv...")
        .text_color(Color::hex(MUTED));
    DATA_LABEL.store(data.raw_ptr(), Ordering::Relaxed);

    let status = Label::new(&root);
    status
        .width(Size::Pct(90))
        .long_mode(LvLabelLongMode::Wrap)
        .text("Ready")
        .text_color(Color::hex(TEXT));
    STATUS_LABEL.store(status.raw_ptr(), Ordering::Relaxed);

    executor::run(clock_task());
    executor::run(load_data_task());
}

pub unsafe fn app_teardown() {
    STATUS_LABEL.store(0, Ordering::Relaxed);
    CLOCK_LABEL.store(0, Ordering::Relaxed);
    DATA_LABEL.store(0, Ordering::Relaxed);
    let screen = Screen::active();
    screen.clean();
}

pub unsafe fn app_init_screen() -> i32 {
    0
}
