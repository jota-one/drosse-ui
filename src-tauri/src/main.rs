#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::mpsc;
use std::thread;
use chrono::Utc;
use tauri::Window;

mod discover;

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
fn init_discover(window: Window) {
  println!("init_discover on window {:?}", window.label());
  let (tx, rx) = mpsc::channel::<discover::LiveInstanceEvent>();
  discover::listen(tx);
  thread::spawn(move || {
    for received in rx {
      match received.event {
        discover::InstanceEvent::Down => window.emit("awesome", Payload {
          message: format!(
            "=> notify FE -> {}:down",
            received.uuid
          )
        }).unwrap(),
        discover::InstanceEvent::Log => window.emit("awesome", Payload {
          message: format!(
            "=> notify FE -> {}:{:?}",
            received.uuid,
            received.data
          )
        }).unwrap(),
        discover::InstanceEvent::Unknown => window.emit("awesome", Payload {
          message: format!(
            "=> unknown event... {:?}",
            received.data
          )
        }).unwrap(),
        discover::InstanceEvent::Up => window.emit("awesome", Payload {
          message: format!(
            "=> update live instances -> {}:{}",
            received.uuid,
            Utc::now().timestamp()
          )
        }).unwrap()
      }
    }
  });
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![init_discover])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
