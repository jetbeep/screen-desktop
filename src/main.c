#include "lvgl/lvgl.h"
#include <SDL2/SDL.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>

#include "version.h"

extern void rust_event_loop(int argc, char *argv[]);
extern void rust_request_shutdown(void);

static void term_signal_handler(int sig)
{
    (void)sig;
    _Exit(0);
}

static int sdl_event_filter(void *userdata, SDL_Event *event)
{
    (void)userdata;
    if (event->type == SDL_KEYDOWN && event->key.keysym.sym == SDLK_ESCAPE) {
        rust_request_shutdown();
        return 0;
    }
    return 1;
}

int main(int argc, char *argv[])
{
    printf("screen-desktop v%s\n", APP_VERSION);
    signal(SIGINT, term_signal_handler);
    signal(SIGTERM, term_signal_handler);

    lv_init();
    lv_display_t *display = lv_sdl_window_create(SDL_HOR_RES, SDL_VER_RES);
    if (display == NULL) {
        fprintf(stderr, "Failed to create SDL display\n");
        return 1;
    }

    SDL_AddEventWatch(sdl_event_filter, NULL);
    lv_indev_t *mouse = lv_sdl_mouse_create();
    lv_indev_t *mouse_wheel = lv_sdl_mousewheel_create();
    lv_indev_t *keyboard = lv_sdl_keyboard_create();
    lv_indev_set_display(mouse, display);
    lv_indev_set_display(mouse_wheel, display);
    lv_indev_set_display(keyboard, display);

    printf("LVGL + SDL2 initialized (%dx%d)\n", SDL_HOR_RES, SDL_VER_RES);
    rust_event_loop(argc, argv);
    return 0;
}
