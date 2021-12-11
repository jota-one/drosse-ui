#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use chrono::Utc;
use home::home_dir;
use serde_json::{
  json,
  Value
};
use std::{
  fs,
  path::Path,
  sync::mpsc,
  thread
};
use tauri::Window;
mod discover;

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
fn browse(dir: String) -> Vec<Value> {
  let entries: Vec<Value> = fs::read_dir(dir).unwrap()
    .filter(|e| e.as_ref().unwrap().path().is_dir())
    .map(|e| {
      let dir_path = e.unwrap().path().display().to_string();
      json!({
        "path": dir_path,
        "selectable": Path::new(&dir_path).join(".drosserc.js").exists()
      })
    })
    .collect();

  return entries
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

#[tauri::command]
fn list() -> Value {
  let drosses_file = Path::new(&home_dir().unwrap()).join(".drosse-ui").join("drosses.json");
  let json = fs::read_to_string(drosses_file).unwrap();
  return serde_json::from_str(&json).unwrap();
}

#[tauri::command]
fn file(uuid: String, file: String) {
  println!("file {:?} {:?}", uuid, file);
}

#[tauri::command]
fn import(path: String) {
  println!("import {:?}", path);
}

#[tauri::command]
fn open(uuid: String, file: String) {
  println!("open {:?} {:?}", uuid, file);
}

#[tauri::command]
fn save(drosses: Value) {
  println!("save {:?}", drosses);
}

#[tauri::command]
fn start(uuid: String) {
  println!("start {:?}", uuid);
}

#[tauri::command]
fn stop(uuid: String) {
  println!("stop {:?}", uuid);
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      init_discover,
      browse,
      list,
      file,
      import,
      open,
      save,
      start,
      stop
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
