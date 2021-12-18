#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod discover;
use chrono::Utc;
use home::home_dir;
use open::that_in_background;
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

const PREF_FOLDER: &str = ".drosse-ui";
const DROSSES_FILE: &str = "drosses.json";

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
  get_drosses()
}

#[tauri::command]
fn file(uuid: String, file: String) -> Value {
  let root = get_drosse_root_path(uuid);
  let content = fs::read_to_string(root).unwrap();
  serde_json::from_str(&["{content:\"", &content, "\"}"].join("")).unwrap()
}

#[tauri::command]
fn import(path: String) {
  println!("import {:?}", path);
}

#[tauri::command]
fn open(uuid: String, file: String) {
  let root = get_drosse_root_path(uuid);
  let file_path = Path::new(&root).join(file);
  
  that_in_background(file_path).join()
    .expect("Could not open file");
}

#[tauri::command]
fn restart(uuid: String) {
  println!("restart {:?}", uuid);
}

#[tauri::command]
fn save(drosses: Value) {
  write_drosses(drosses)
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
      restart,
      save,
      start,
      stop
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn get_drosse_root_path(uuid: String) -> String {
  let drosses = get_drosses();
  
  let root: &str = drosses[&uuid]["root"].as_str()
    .expect("Could not find root path");
  
  root.to_string()
}

fn get_drosses_file_path() -> String {
  Path::new(&home_dir().unwrap())
    .join(PREF_FOLDER)
    .join("drosses.json")
    .display().to_string()
}

fn get_drosses() -> Value {
  let drosses_file_path = get_drosses_file_path();
  let json = fs::read_to_string(drosses_file_path).unwrap();
  serde_json::from_str(&json).unwrap()
}

fn write_drosses(drosses: Value) {
  let drosses_file_path = get_drosses_file_path();
  let file = std::fs::OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(drosses_file_path)
    .unwrap();
  
  serde_json::to_writer(&file, &drosses);
}