#include "lvgl.h"
#include <stdio.h>
#include <string.h>

#if !defined(_WIN32)
#include <dirent.h>
#endif

static char base_path[1024];
static lv_fs_drv_t driver;

static void *file_open(lv_fs_drv_t *drv, const char *path, lv_fs_mode_t mode)
{
    (void)drv;
    const char *flags = mode == LV_FS_MODE_WR ? "wb" :
                        mode == LV_FS_MODE_RD ? "rb" : "rb+";
    char full_path[2048];
    snprintf(full_path, sizeof(full_path), "%s%s", base_path, path);
    return fopen(full_path, flags);
}

static lv_fs_res_t file_close(lv_fs_drv_t *drv, void *file)
{
    (void)drv;
    return fclose(file) == 0 ? LV_FS_RES_OK : LV_FS_RES_UNKNOWN;
}

static lv_fs_res_t file_read(lv_fs_drv_t *drv, void *file, void *buf, uint32_t count, uint32_t *read)
{
    (void)drv;
    FILE *stream = file;
    *read = (uint32_t)fread(buf, 1, count, stream);
    return ferror(stream) ? LV_FS_RES_UNKNOWN : LV_FS_RES_OK;
}

static lv_fs_res_t file_seek(lv_fs_drv_t *drv, void *file, uint32_t pos, lv_fs_whence_t whence)
{
    (void)drv;
    int origin = whence == LV_FS_SEEK_SET ? SEEK_SET :
                 whence == LV_FS_SEEK_CUR ? SEEK_CUR : SEEK_END;
    return fseek(file, (long)pos, origin) == 0 ? LV_FS_RES_OK : LV_FS_RES_UNKNOWN;
}

static lv_fs_res_t file_tell(lv_fs_drv_t *drv, void *file, uint32_t *pos)
{
    (void)drv;
    long current = ftell(file);
    if (current < 0) {
        return LV_FS_RES_UNKNOWN;
    }
    *pos = (uint32_t)current;
    return LV_FS_RES_OK;
}

#if !defined(_WIN32)
typedef struct {
    DIR *directory;
} directory_handle_t;

static void *directory_open(lv_fs_drv_t *drv, const char *path)
{
    (void)drv;
    char full_path[2048];
    snprintf(full_path, sizeof(full_path), "%s%s", base_path, path);
    directory_handle_t *handle = lv_malloc(sizeof(*handle));
    if (handle == NULL) {
        return NULL;
    }
    handle->directory = opendir(full_path);
    if (handle->directory == NULL) {
        lv_free(handle);
        return NULL;
    }
    return handle;
}

static lv_fs_res_t directory_read(lv_fs_drv_t *drv, void *dir, char *name, uint32_t name_len)
{
    (void)drv;
    directory_handle_t *handle = dir;
    struct dirent *entry;
    do {
        entry = readdir(handle->directory);
        if (entry == NULL) {
            lv_strlcpy(name, "", name_len);
            return LV_FS_RES_OK;
        }
        if (entry->d_type == DT_DIR) {
            lv_snprintf(name, name_len, "/%s", entry->d_name);
        } else {
            lv_strlcpy(name, entry->d_name, name_len);
        }
    } while (lv_strcmp(name, "/.") == 0 || lv_strcmp(name, "/..") == 0);
    return LV_FS_RES_OK;
}

static lv_fs_res_t directory_close(lv_fs_drv_t *drv, void *dir)
{
    (void)drv;
    directory_handle_t *handle = dir;
    closedir(handle->directory);
    lv_free(handle);
    return LV_FS_RES_OK;
}
#endif

void lvgl_fs_j_set_base_path(const char *path)
{
    lv_strlcpy(base_path, path, sizeof(base_path));
    size_t len = strlen(base_path);
    if (len > 0 && base_path[len - 1] != '/' && len < sizeof(base_path) - 1) {
        base_path[len] = '/';
        base_path[len + 1] = '\0';
    }
}

void lvgl_fs_j_register(void)
{
    lv_fs_drv_init(&driver);
    driver.letter = 'J';
    driver.open_cb = file_open;
    driver.close_cb = file_close;
    driver.read_cb = file_read;
    driver.seek_cb = file_seek;
    driver.tell_cb = file_tell;
#if !defined(_WIN32)
    driver.dir_open_cb = directory_open;
    driver.dir_read_cb = directory_read;
    driver.dir_close_cb = directory_close;
#endif
    lv_fs_drv_register(&driver);
}
