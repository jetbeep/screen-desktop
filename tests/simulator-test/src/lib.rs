use jetbeep_core::lvgl::*;
use jetbeep_core::{bus, executor};

static mut RESULT_LABEL: *mut lv_obj_t = core::ptr::null_mut();

fn set_result(text: &str) {
    unsafe {
        let ptr = RESULT_LABEL;
        if ptr.is_null() {
            return;
        }
        let label = LvObj::from_raw(ptr);
        lv_label_set_text(&label, text);
        core::mem::forget(label);
    }
}

unsafe extern "C" fn open_lock(_: *mut lv_event_t) {
    executor::run(async {
        match bus::lock_open(1, 1).await {
            Ok(()) => set_result("Lock 1:1 opened"),
            Err(error) => set_result(&format!("Open failed: {error}")),
        }
    });
}

unsafe extern "C" fn read_status(_: *mut lv_event_t) {
    executor::run(async {
        match bus::lock_statuses_get(1).await {
            Ok(statuses) => set_result(&format!("Board 1 statuses: {statuses:?}")),
            Err(error) => set_result(&format!("Status failed: {error}")),
        }
    });
}

fn add_button(parent: &LvObj, text: &str, callback: lv_event_cb_t) {
    let button = lv_button_create(parent);
    lv_obj_set_size(&button, 220, 54);
    let label = lv_label_create(&button);
    lv_label_set_text(&label, text);
    lv_obj_align(&label, LvAlign::Center, 0, 0);
    lv_obj_add_event_cb(&button, callback, LV_EVENT_CLICKED, core::ptr::null_mut());
}

pub unsafe fn app_main() {
    let screen = lv_screen_active();
    lv_obj_set_style_bg_color(&screen, lv_color_hex_fn(0x18222D), 0);

    let panel = lv_obj_create(&screen);
    lv_obj_set_size(&panel, 560, 320);
    lv_obj_align(&panel, LvAlign::Center, 0, 0);
    lv_obj_set_flex_flow(&panel, LV_FLEX_FLOW_COLUMN);
    lv_obj_set_style_pad_all(&panel, 28, 0);
    lv_obj_set_style_pad_row(&panel, 18, 0);

    let title = lv_label_create(&panel);
    lv_label_set_text(&title, "screen-desktop simulator test");
    lv_obj_set_style_text_font(&title, &lv_font_montserrat_30(), 0);

    add_button(&panel, "Open lock 1:1", open_lock);
    add_button(&panel, "Read board 1 status", read_status);

    let result = lv_label_create(&panel);
    lv_label_set_text(&result, "Ready");
    lv_obj_set_width(&result, 500);
    unsafe {
        RESULT_LABEL = result.as_raw();
    }
}

pub unsafe fn app_teardown() {
    unsafe {
        RESULT_LABEL = core::ptr::null_mut();
    }
    let screen = lv_screen_active();
    lv_obj_clean(&screen);
}

pub unsafe fn app_init_screen() -> i32 {
    0
}
