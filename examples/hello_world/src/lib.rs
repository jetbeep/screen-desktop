use lvgl_dsl::prelude::*;

pub unsafe fn app_main() {
    let screen = Screen::active();
    let root = Obj::new(&screen);
    root.width(Size::Pct(100))
        .height(Size::Pct(100))
        .bg_color(Color::hex(0xF4F6F8))
        .border_width(0)
        .flex_col()
        .flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
        .remove_flag(LvObjFlag::SCROLLABLE);

    let greeting = Label::new(&root);
    greeting
        .text("Hello, world!")
        .text_font(&Font::montserrat_48())
        .text_color(Color::hex(0x123B5D));
}

pub unsafe fn app_teardown() {
    let screen = Screen::active();
    screen.clean();
}

pub unsafe fn app_init_screen() -> i32 {
    0
}
