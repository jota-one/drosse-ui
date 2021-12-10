use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Serialize, Deserialize)]
struct UdpCommand {
  data: String,
  event: String,
}

#[derive(Debug)]
pub enum InstanceEvent {
  Down,
  Log,
  Unknown,
  Up,
}

impl str::FromStr for InstanceEvent {
  type Err = ();

  fn from_str(s: &str) -> Result<InstanceEvent, ()> {
    println!("{}", s);
    match s {
      "down" => Ok(InstanceEvent::Down),
      "log" => Ok(InstanceEvent::Log),
      "up" => Ok(InstanceEvent::Up),
      _ => Ok(InstanceEvent::Unknown),
    }
  }
}

pub struct LiveInstanceEvent {
  pub uuid: String,
  pub event: InstanceEvent,
  pub data: Option<String>,
}

pub fn send_cmd(socket: &Socket, addr: &SockAddr, uuid: &str, cmd: &str) {
  let command = UdpCommand {
    data: uuid.to_string(),
    event: cmd.to_string(),
  };

  let json = serde_json::to_string(&command).unwrap();
  let r = socket.send_to(json.as_bytes(), addr);

  println!("{:?}", r);
}

pub fn listen(tx: mpsc::Sender<LiveInstanceEvent>) {
  let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();

  // TODO make address + port configurable
  let addr = &"0.0.0.0:12345".parse::<SocketAddr>().unwrap().into();

  // TODO panic if reuse address fails
  socket.set_reuse_address(true).unwrap();

  // TODO panic if reuse port fails
  #[cfg(target_family = "windows")]
  socket.set_broadcast(true).unwrap();

  #[cfg(target_family = "unix")]
  socket.set_reuse_port(true).unwrap();

  // TODO panic if bind address fails
  socket.bind(addr).unwrap();

  let mut live_instances = HashMap::<String, i64>::new();
  // let (tx, rx) = mpsc::channel::<LiveInstanceEvent>();

  println!("{:?}", "up".parse::<InstanceEvent>().unwrap());

  thread::spawn(move || {
    loop {
      // TODO improve buffer size mgmt
      let mut buf = [0; 65507];
      // TODO handle Result
      let (size, _) = socket.recv_from(&mut buf).unwrap();

      let data = match str::from_utf8(&buf[..size]) {
        Ok(v) => v,
        // TODO don't panic here
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
      };

      let json: Value = serde_json::from_str(data).expect("Could not parse JSON");

      let advertisement = json["data"]["advertisement"].as_object();

      if advertisement != None && !advertisement.unwrap().is_empty() {
        // TODO ensure uuid exists and is well formatted
        let uuid = advertisement.unwrap()["uuid"].to_string();

        println!("=> update live instances -> {}:{}", uuid, Utc::now().timestamp());
        // live_instances.insert(uuid, Utc::now().timestamp());
        tx.send(LiveInstanceEvent {
          uuid: uuid,
          event: InstanceEvent::Up,
          data: None,
        })
        .unwrap();
      } else {
        let event = json["event"]
          .as_str()
          .unwrap()
          .parse::<InstanceEvent>()
          .unwrap();
        
        if matches!(event, InstanceEvent::Unknown) {
          continue
        }

        // TODO ensure uuid exists and is well formatted
        println!("=> data {:?}, event {:?}", json["data"], event);
        let uuid = json["data"]["uuid"].as_str().unwrap().to_string();
        let data = Some(serde_json::to_string(&json["data"]).unwrap());

        println!("=> notify FE -> {}:{:?}:{:?}", uuid, event, data);
        tx.send(LiveInstanceEvent { uuid, event, data }).unwrap();
      }
    }
  });
}
