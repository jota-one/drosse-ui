#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod discover;
use chrono::Utc;
use home::home_dir;
use open::that_in_background;
use serde::{Serialize, Deserialize};
use serde_json::{
  json,
  Value
};
use std::{
  collections::HashMap,
  fs,
  path::Path,
  sync::mpsc,
  thread
};
use tauri::Window;

const PREF_FOLDER: &str = ".drosse-ui";
const DROSSES_FILE: &str = "drosses.json";
const DROSSE_CONFIG_FILE: &str = ".drosserc.js";
const DROSSE_UUID_FILE: &str = ".uuid";

#[derive(Debug, Serialize, Deserialize)]
struct Drosse {
  available: Option<bool>,
  collectionsPath: String,
  hosts: Vec<String>,
  lastSeen: String,
  name: String,
  open: bool,
  port: u32,
  proto: String,
  root: String,
  routes: Option<Vec<Value>>,
  routesFile: String,
  selected: bool,
  uuid: String,
  version: String
}

#[derive(Clone, Serialize)]
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
        "selectable": Path::new(&dir_path).join(DROSSE_CONFIG_FILE).exists()
      })
    })
    .collect();

  return entries
}

#[tauri::command]
fn init_discover(window: Window) {
  println!("init_discover on window {:?}", window.label());
  let discover = discover::Discover::new();

  let (tx, rx) = mpsc::channel::<discover::DiscoverEvent>();
  discover.listen(tx);
  thread::spawn(move || {
    for received in rx {
      match received.event {
        discover::InstanceEvent::Down => window.emit("drosse.down", Payload {
          message: format!(
            "=> notify FE -> {}:down",
            received.uuid
          )
        }).unwrap(),
        discover::InstanceEvent::Log => window.emit("drosse.log", Payload {
          message: format!(
            "=> notify FE -> {}:{:?}",
            received.uuid,
            received.data
          )
        }).unwrap(),
        discover::InstanceEvent::Unknown =>
          println!("=> unknown event... {:?}", received.data),
        discover::InstanceEvent::Up => window.emit("drosse.up", Payload {
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
fn list() -> HashMap<String, Drosse> {
  get_drosses()
}

#[tauri::command]
fn file(uuid: String, file: String) -> Value {
  let drosses = get_drosses();
  let root = get_drosse_root_path(&drosses, &uuid);
  let content = fs::read_to_string(root).unwrap();
  serde_json::from_str(&["{content:\"", &content, "\"}"].join("")).unwrap()
}

#[tauri::command]
fn import(path: String) {
  println!("import {:?}", path);
}

#[tauri::command]
fn open(uuid: String, file: String) {
  let drosses = get_drosses();
  let root = get_drosse_root_path(&drosses, &uuid);
  let file_path = Path::new(&root).join(file);
  
  that_in_background(file_path).join()
    .expect("Could not open file");
}

#[tauri::command]
fn restart(uuid: String) {
  println!("restart {:?}", uuid);
}

#[tauri::command]
fn save(drosses: HashMap<String, Drosse>) {
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

fn get_drosse_root_path(drosses: &HashMap<String, Drosse>, uuid: &String) -> String {
  let drosse = drosses.get(uuid).expect("Could not get drosse");
  drosse.root.to_string()
}

fn get_drosses_file_path() -> String {
  Path::new(&home_dir().unwrap())
    .join(PREF_FOLDER)
    .join(DROSSES_FILE)
    .display().to_string()
}

fn get_drosses() -> HashMap<String, Drosse> {
  let drosses_file_path = get_drosses_file_path();
  let content = fs::read_to_string(drosses_file_path).unwrap();
  
  let mut drosses: HashMap<String, Drosse> = serde_json::from_str(&content).unwrap();

  let uuids: Vec<String> = drosses.keys().cloned().collect();
  
  for uuid in uuids {
    let drosse = drosses.get(&uuid).unwrap();
    let is_available = Path::new(&drosse.root).join(DROSSE_CONFIG_FILE).exists();
    drosses.entry(uuid).and_modify(|drosse| {
      drosse.available = Some(is_available);
    });
  }

  drosses
}

fn write_drosses(drosses: HashMap<String, Drosse>) {
  let drosses_file_path = get_drosses_file_path();
  let file = std::fs::OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(drosses_file_path)
    .unwrap();

  match serde_json::to_writer(&file, &drosses) {
    Ok(v) => println!("Drosses saved!"),
    Err(e) => println!("Error writing drosses: {}", e)
  };
}