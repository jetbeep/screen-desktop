# API example assets

This example keeps editable sources under `assets/` and generated runtime files
under `fs/lfs1/app/`. The runtime filesystem is mounted as LVGL drive `J:`, so
`fs/lfs1/app/img/test-image.bin` is opened as `J:app/img/test-image.bin`.

Run the commands below from the `screen-desktop` repository root.

## JKV data

Generate the binary data file and convert it back to readable JSON:

```sh
python3 tools/json_to_jkv.py \
  examples/api-examples/assets/test.json \
  examples/api-examples/fs/lfs1/app/data/test.jkv

python3 tools/jkv_to_json.py \
  examples/api-examples/fs/lfs1/app/data/test.jkv \
  /tmp/test.json
```

Application filesystem APIs omit the LVGL drive prefix and read this file as
`lfs1/app/data/test.jkv`.

## Binary font

Generate a 22 px, 4-bpp ASCII subset with `lv_font_conv`:

```sh
lv_font_conv \
  --font examples/api-examples/assets/fonts/Poppins-Regular.ttf \
  --size 22 --bpp 4 --format bin --range 0x20-0x7e \
  --no-compress \
  --output examples/api-examples/fs/lfs1/app/fonts/poppins_regular_22.bin
```

Install the converter with `npm install -g lv_font_conv` when it is not already
available.

## Binary image

Rasterize the editable SVG, install the converter's isolated Python
dependencies, and generate an opaque RGB565 LVGL image:

```sh
rsvg-convert \
  --output examples/api-examples/assets/test-image.png \
  examples/api-examples/assets/test-image.svg \

python3 -m venv tools/.venv
tools/.venv/bin/pip install pypng lz4
tools/.venv/bin/python third_party/lvgl/scripts/LVGLImage.py \
  --ofmt BIN --cf RGB565 \
  --output examples/api-examples/fs/lfs1/app/img \
  --name test-image \
  examples/api-examples/assets/test-image.png
```