#include "lvgl/lvgl.h"
#include "lvgl/src/widgets/buttonmatrix/lv_buttonmatrix_private.h"

void jb_keyboard_draw_key_border(lv_event_t *event, const char *label,
                                 lv_color_t color, int32_t border_width, int32_t radius)
{
    if (event == NULL || label == NULL || border_width <= 0) {
        return;
    }

    lv_obj_t *object = lv_event_get_target(event);
    if (object == NULL) {
        return;
    }
    lv_buttonmatrix_t *matrix = (lv_buttonmatrix_t *)object;
    if (matrix->button_areas == NULL || matrix->btn_cnt == 0) {
        return;
    }

    uint32_t found = LV_BUTTONMATRIX_BUTTON_NONE;
    for (uint32_t index = 0; index < matrix->btn_cnt; index++) {
        const char *text = lv_buttonmatrix_get_button_text(object, index);
        if (text != NULL && lv_strcmp(text, label) == 0) {
            found = index;
            break;
        }
    }
    if (found == LV_BUTTONMATRIX_BUTTON_NONE) {
        return;
    }

    lv_area_t object_area;
    lv_obj_get_coords(object, &object_area);
    lv_area_t area = matrix->button_areas[found];
    area.x1 += object_area.x1;
    area.x2 += object_area.x1;
    area.y1 += object_area.y1;
    area.y2 += object_area.y1;

    lv_layer_t *layer = lv_event_get_layer(event);
    if (layer == NULL) {
        return;
    }
    lv_draw_rect_dsc_t descriptor;
    lv_draw_rect_dsc_init(&descriptor);
    descriptor.bg_opa = LV_OPA_TRANSP;
    descriptor.border_color = color;
    descriptor.border_width = border_width;
    descriptor.border_opa = LV_OPA_COVER;
    descriptor.radius = radius;
    lv_draw_rect(layer, &descriptor, &area);
}
