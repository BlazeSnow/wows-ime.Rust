// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 与 lib.rs 一致：静音 MSVC 链接器的例行输出
#![allow(linker_messages)]

fn main() {
    wows_ime_lib::run()
}
