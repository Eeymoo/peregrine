// Peregrine Tauri 入口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    peregrine_tauri::run();
}
