// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use enigo::{
    Direction::{Press, Release},
    Enigo, Key, Keyboard, Settings,
};

fn main() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let _ = enigo.key(Key::Control, Press);
    let _ = enigo.key(Key::F23, Press);
    let _ = enigo.key(Key::F23, Release);
    let _ = enigo.key(Key::Control, Release);
}