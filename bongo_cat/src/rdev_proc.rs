use crate::protocol::IpcEvent;
use ipc_channel::ipc::IpcSender;
use rdev::{Event, EventType, listen};
use serde_json::json;
use std::env;
use std::process::exit;
use std::sync::{
    Arc,
    mpsc::{Sender, channel},
};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub fn run_child() {
    let cat = CatSender::connect();
    cat.send_ipc_event(IpcEvent::default());
    cat.send_ipc_event(IpcEvent::default());
    cat.send_ipc_event(IpcEvent::default());
    cat.send_ipc_event(IpcEvent::default());

    thread::spawn(move || {
        sleep(Duration::from_secs(10));
        exit(0);
    });

    let callback = move |event: Event| {
        let ipc_event = match event.event_type {
            EventType::ButtonPress(button) => {
                let json = json!(format!("{:?}", button));
                IpcEvent {
                    action: "rdev".to_string(),
                    json: json.to_string(),
                }
            }
            EventType::ButtonRelease(button) => {
                let json = json!(format!("{:?}", button));
                IpcEvent {
                    action: "rdev".to_string(),
                    json: json.to_string(),
                }
            }
            EventType::MouseMove { x, y } => {
                let json = json!({ "x": x, "y": y });
                IpcEvent {
                    action: "rdev".to_string(),
                    json: json.to_string(),
                }
            }
            EventType::KeyPress(key) => {
                let json = json!(format!("{:?}", key));
                IpcEvent {
                    action: "rdev".to_string(),
                    json: json.to_string(),
                }
            }
            EventType::KeyRelease(key) => {
                let json = json!(format!("{:?}", key));
                IpcEvent {
                    action: "rdev".to_string(),
                    json: json.to_string(),
                }
            }
            _ => return,
        };
        cat.send_ipc_event(ipc_event);
    };

    if let Err(e) = listen(callback) {
        eprintln!("Device listening error: {:?}", e);
    }
}

struct CatSender {
    sender: Arc<Sender<IpcEvent>>,
}

impl CatSender {
    fn connect() -> Self {
        let args: Vec<String> = env::args().collect();
        let server_name = args.get(2).expect("no server name").to_string();
        let ipc_sender: IpcSender<IpcEvent> = IpcSender::connect(server_name).unwrap();

        let (tx, rx) = channel::<IpcEvent>();
        thread::spawn(move || {
            for evt in rx {
                if let Err(e) = ipc_sender.send(evt) {
                    eprintln!("ipc_sender to server failed: {:?}", e);
                }
            }
        });
        Self {
            sender: Arc::new(tx),
        }
    }

    fn send_ipc_event(&self, event: IpcEvent) {
        let sender = self.sender.clone();
        if let Err(e) = sender.send(event) {
            eprintln!("failed to sync ipc_event: {:?}", e);
            exit(1);
        }
    }
}
