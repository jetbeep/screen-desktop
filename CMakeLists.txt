cmake_minimum_required(VERSION 3.20)

file(READ "${CMAKE_SOURCE_DIR}/VERSION.txt" PROJECT_VER)
string(STRIP "${PROJECT_VER}" PROJECT_VER)
project(screen_desktop VERSION ${PROJECT_VER} LANGUAGES C CXX)

set(APP_VERSION "${PROJECT_VER}")
configure_file(
    "${CMAKE_SOURCE_DIR}/src/version.h.in"
    "${CMAKE_BINARY_DIR}/generated/version.h"
)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(APP_PATH "" CACHE PATH "Path to the external project_app crate")
set(SDL_HOR_RES 1280 CACHE STRING "Horizontal display resolution")
set(SDL_VER_RES 800 CACHE STRING "Vertical display resolution")
set(RUST_LIBS_PATH "" CACHE PATH "Optional local rust-libs checkout for SDK development")
set(SERVICE_MENU_PATH "" CACHE PATH "Optional local service-menu checkout for development")
option(APP_FORCE_HTTP_API "Enable the selected app's force-http-api feature" OFF)
option(APP_DEV_PREVIEW "Enable the selected app's dev-screen-preview feature" OFF)

if(APP_PATH STREQUAL "")
    message(FATAL_ERROR
        "APP_PATH is required and must point to an external project_app crate.\n"
        "For the bundled fixture use: -DAPP_PATH=${CMAKE_SOURCE_DIR}/tests/simulator-test")
endif()
get_filename_component(APP_DIR "${APP_PATH}" REALPATH BASE_DIR "${CMAKE_SOURCE_DIR}")
if(NOT EXISTS "${APP_DIR}/Cargo.toml")
    message(FATAL_ERROR "APP_PATH='${APP_PATH}' does not contain Cargo.toml")
endif()
get_filename_component(APP_NAME "${APP_DIR}" NAME)
message(STATUS "Selected external app: ${APP_NAME} (${APP_DIR})")

if(NOT EXISTS "${CMAKE_SOURCE_DIR}/third_party/SDL/CMakeLists.txt"
   OR NOT EXISTS "${CMAKE_SOURCE_DIR}/third_party/lvgl/CMakeLists.txt")
    message(FATAL_ERROR "Missing SDL/LVGL submodules. Run: git submodule update --init --recursive")
endif()

set(SDL_SHARED OFF CACHE BOOL "Build SDL shared library" FORCE)
set(SDL_STATIC ON CACHE BOOL "Build SDL static library" FORCE)
set(SDL_TEST_LIBRARY OFF CACHE BOOL "Build SDL tests" FORCE)
set(SDL_TESTS OFF CACHE BOOL "Build SDL tests" FORCE)
set(SDL_DISABLE_INSTALL ON CACHE BOOL "Disable SDL install target" FORCE)
add_subdirectory(third_party/SDL EXCLUDE_FROM_ALL)

set(SDL2_TARGET "")
foreach(_candidate SDL2::SDL2-static SDL2::SDL2 SDL2-static SDL2)
    if(TARGET ${_candidate})
        set(SDL2_TARGET ${_candidate})
        break()
    endif()
endforeach()
if(SDL2_TARGET STREQUAL "")
    message(FATAL_ERROR "Failed to resolve SDL2 target")
endif()

set(LV_CONF_PATH "${CMAKE_SOURCE_DIR}/lv_conf.h" CACHE STRING "" FORCE)
add_compile_definitions(
    LV_CONF_INCLUDE_SIMPLE=1
    SDL_HOR_RES=${SDL_HOR_RES}
    SDL_VER_RES=${SDL_VER_RES}
    SDL_MAIN_HANDLED
    LV_USE_DRAW_SW_ASM=LV_DRAW_SW_ASM_NONE
    LV_USE_NATIVE_HELIUM_ASM=0
    LV_USE_LODEPNG=1
    LV_USE_FS_STDIO=1
    "LV_FS_STDIO_LETTER='S'"
    "LV_FS_STDIO_PATH=\"${APP_DIR}/fs/lfs1/\""
)
set(LV_CONF_BUILD_DISABLE_DEMOS ON CACHE BOOL "" FORCE)
set(LV_CONF_BUILD_DISABLE_EXAMPLES ON CACHE BOOL "" FORCE)
add_subdirectory(third_party/lvgl)
target_include_directories(lvgl PUBLIC ${CMAKE_SOURCE_DIR})
target_link_libraries(lvgl PUBLIC ${SDL2_TARGET})
set_target_properties(lvgl PROPERTIES CXX_STANDARD 17 CXX_STANDARD_REQUIRED ON)
target_compile_features(lvgl PUBLIC cxx_std_17)
get_target_property(_lvgl_sources lvgl SOURCES)
list(FILTER _lvgl_sources EXCLUDE REGEX "\\.[sS]$")
set_target_properties(lvgl PROPERTIES SOURCES "${_lvgl_sources}")

execute_process(
    COMMAND rustc -vV
    OUTPUT_VARIABLE RUSTC_VERSION_OUTPUT
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
string(REGEX MATCH "host: ([^ \n]+)" _ "${RUSTC_VERSION_OUTPUT}")
set(RUST_HOST_TARGET "${CMAKE_MATCH_1}")
if(RUST_HOST_TARGET STREQUAL "")
    message(FATAL_ERROR "Could not determine the host Rust target")
endif()

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(RUST_PROFILE release)
    set(CARGO_BUILD_FLAGS --release)
else()
    set(RUST_PROFILE debug)
    set(CARGO_BUILD_FLAGS "")
endif()
if(APP_FORCE_HTTP_API)
    list(APPEND CARGO_BUILD_FLAGS --features force-http-api)
endif()
if(APP_DEV_PREVIEW)
    list(APPEND CARGO_BUILD_FLAGS --features dev-screen-preview)
endif()

set(CARGO_OVERRIDE_CONFIG "${CMAKE_BINARY_DIR}/cargo-local-overrides.toml")
file(WRITE "${CARGO_OVERRIDE_CONFIG}" "")
if(NOT RUST_LIBS_PATH STREQUAL "")
    get_filename_component(_rust_libs "${RUST_LIBS_PATH}" REALPATH BASE_DIR "${CMAKE_SOURCE_DIR}")
    foreach(_crate jetbeep-core jkv lvgl-dsl fonts-cache)
        if(NOT EXISTS "${_rust_libs}/${_crate}/Cargo.toml")
            message(FATAL_ERROR "RUST_LIBS_PATH is missing ${_crate}/Cargo.toml")
        endif()
    endforeach()
    file(APPEND "${CARGO_OVERRIDE_CONFIG}"
        "[patch.\"https://github.com/jetbeep/rust-libs.git\"]\n"
        "jetbeep-core = { path = \"${_rust_libs}/jetbeep-core\" }\n"
        "jkv = { path = \"${_rust_libs}/jkv\" }\n"
        "lvgl-dsl = { path = \"${_rust_libs}/lvgl-dsl\" }\n"
        "fonts-cache = { path = \"${_rust_libs}/fonts-cache\" }\n\n")
endif()
if(NOT SERVICE_MENU_PATH STREQUAL "")
    get_filename_component(_service_menu "${SERVICE_MENU_PATH}" REALPATH BASE_DIR "${CMAKE_SOURCE_DIR}")
    if(NOT EXISTS "${_service_menu}/Cargo.toml")
        message(FATAL_ERROR "SERVICE_MENU_PATH does not contain Cargo.toml")
    endif()
    file(APPEND "${CARGO_OVERRIDE_CONFIG}"
        "[patch.\"https://github.com/jetbeep/service-menu.git\"]\n"
        "project_app = { path = \"${_service_menu}\" }\n")
endif()

set(APP_SYMLINK "${CMAKE_SOURCE_DIR}/rust/app_selected")
file(REMOVE "${APP_SYMLINK}")
file(CREATE_LINK "${APP_DIR}" "${APP_SYMLINK}" SYMBOLIC RESULT APP_LINK_RESULT)
if(NOT APP_LINK_RESULT STREQUAL "0")
    message(FATAL_ERROR "Failed to link ${APP_SYMLINK} to ${APP_DIR}: ${APP_LINK_RESULT}")
endif()

set(RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/rust/target")
set(RUST_LIBRARY "${RUST_TARGET_DIR}/${RUST_HOST_TARGET}/${RUST_PROFILE}/librustapp.a")
get_target_property(LVGL_INC_DIRS lvgl INTERFACE_INCLUDE_DIRECTORIES)
list(APPEND LVGL_INC_DIRS "${CMAKE_SOURCE_DIR}" "${CMAKE_SOURCE_DIR}/third_party/lvgl")
list(JOIN LVGL_INC_DIRS " " LVGL_INCLUDE_DIRS_STR)

add_custom_target(rustapp ALL
    BYPRODUCTS ${RUST_LIBRARY}
    COMMAND ${CMAKE_COMMAND} -E env
        SELECTED_APP_PATH=${APP_DIR}
        LVGL_INCLUDE_DIRS=${LVGL_INCLUDE_DIRS_STR}
        LV_CONF_DIR=${CMAKE_SOURCE_DIR}
        cargo build --config "${CARGO_OVERRIDE_CONFIG}" ${CARGO_BUILD_FLAGS}
            --target ${RUST_HOST_TARGET}
            --target-dir "${RUST_TARGET_DIR}"
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}/rust"
    COMMENT "Building external Rust app: ${APP_NAME}"
    USES_TERMINAL
    VERBATIM
)

set(PROFILES_SRC "${APP_DIR}/config/profiles.json")
set(PROFILES_JKV "${APP_DIR}/fs/lfs1/profiles.jkv")
if(EXISTS "${PROFILES_SRC}")
    add_custom_command(
        OUTPUT "${PROFILES_JKV}"
        COMMAND python3 "${CMAKE_SOURCE_DIR}/tools/build_profiles.py"
            --src "${PROFILES_SRC}" --out "${PROFILES_JKV}"
        DEPENDS "${PROFILES_SRC}" "${CMAKE_SOURCE_DIR}/tools/build_profiles.py"
        COMMENT "Encoding profiles JKV: ${APP_NAME}"
    )
    add_custom_target(profiles_jkv ALL DEPENDS "${PROFILES_JKV}")
    add_dependencies(rustapp profiles_jkv)
endif()

add_executable(screen_desktop src/main.c src/lv_fs_j_register.c src/jb_kbd_shim.c)
add_dependencies(screen_desktop rustapp lvgl)
set_target_properties(screen_desktop PROPERTIES LINKER_LANGUAGE CXX)
target_link_libraries(screen_desktop PRIVATE lvgl ${SDL2_TARGET} ${RUST_LIBRARY})
if(APPLE)
    target_link_libraries(screen_desktop PRIVATE
        "-framework CoreFoundation" "-framework Security" "-framework SystemConfiguration")
endif()
target_include_directories(screen_desktop PRIVATE
    ${CMAKE_SOURCE_DIR}/third_party ${CMAKE_SOURCE_DIR} ${CMAKE_BINARY_DIR}/generated)

if(EXISTS "${APP_DIR}/fonts/fonts.cmake")
    include("${APP_DIR}/fonts/fonts.cmake")
    if(APP_FONT_SOURCES)
        target_sources(screen_desktop PRIVATE ${APP_FONT_SOURCES})
    endif()
    if(APP_FONT_INCLUDE_DIRS)
        target_include_directories(screen_desktop PRIVATE ${APP_FONT_INCLUDE_DIRS})
    endif()
endif()

foreach(_cfg simulator_config.json agent_config.json)
    if(EXISTS "${APP_DIR}/${_cfg}")
        configure_file("${APP_DIR}/${_cfg}" "${CMAKE_BINARY_DIR}/${_cfg}" COPYONLY)
    endif()
endforeach()
file(REMOVE_RECURSE "${CMAKE_BINARY_DIR}/simulator_layouts")
if(EXISTS "${APP_DIR}/simulator_layouts")
    file(COPY "${APP_DIR}/simulator_layouts" DESTINATION "${CMAKE_BINARY_DIR}")
endif()
