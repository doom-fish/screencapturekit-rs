// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Import the library crate
mod lib;

fn main() {
    lib::run()
}
