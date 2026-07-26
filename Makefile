APP_PATH ?= $(CURDIR)/examples/hello_world
BUILD_DIR ?= build
BUILD_TYPE ?= Debug

.PHONY: configure build run clean

configure:
	cmake -S . -B $(BUILD_DIR) -DAPP_PATH="$(APP_PATH)" -DCMAKE_BUILD_TYPE=$(BUILD_TYPE)

build: configure
	cmake --build $(BUILD_DIR) -j

run: build
	./$(BUILD_DIR)/screen_desktop --fs-root "$(APP_PATH)/fs" --workspace "$(APP_PATH)"

clean:
	cmake -E remove_directory $(BUILD_DIR)
