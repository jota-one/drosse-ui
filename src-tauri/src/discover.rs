use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::SocketAddr;
use std::str::{
  FromStr,
  from_utf8
};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug)]
pub enum InstanceEvent {
  Down,
  Log,
  Unknown,
  Up,
}

#[derive(Debug)]
pub struct DiscoverEvent {
  pub uuid: String,
  pub event: InstanceEvent,
  pub data: Option<String>,
}

impl FromStr for InstanceEvent {
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

pub struct Discover {
  socket: Socket,
  addr: SockAddr
}

#[derive(Debug, Serialize, Deserialize)]
struct UdpCommand {
  data: String,
  event: String,
}

impl Discover {
  pub fn new() -> Self {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp())).unwrap();
  
    // TODO make address + port configurable
    let addr = "0.0.0.0:12345".parse::<SocketAddr>().unwrap().into();
  
    // TODO panic if reuse address fails
    socket.set_reuse_address(true).unwrap();
  
    // TODO panic if reuse port fails
    #[cfg(target_family = "windows")]
    socket.set_broadcast(true).unwrap();
  
    #[cfg(target_family = "unix")]
    socket.set_reuse_port(true).unwrap();
  
    // TODO panic if bind address fails
    socket.bind(&addr).unwrap();
    
    Discover { addr, socket }
  }

  pub fn listen(self, tx: Sender<DiscoverEvent>) {
    thread::spawn(move || {
      loop {
        // TODO improve buffer size mgmt
        let mut buf = [0; 65507];
        // TODO handle Result
        let (size, _) = self.socket.recv_from(&mut buf).unwrap();
  
        let data = match from_utf8(&buf[..size]) {
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
          tx.send(DiscoverEvent {
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
          tx.send(DiscoverEvent { uuid, event, data }).unwrap();
        }
      }
    });
  }

  pub fn send_cmd(self, uuid: &str, cmd: &str) {
    let command = UdpCommand {
      data: uuid.to_string(),
      event: cmd.to_string(),
    };
  
    let json = serde_json::to_string(&command).unwrap();
    let r = self.socket.send_to(json.as_bytes(), &self.addr);
  
    println!("{:?}", r);
  }
}