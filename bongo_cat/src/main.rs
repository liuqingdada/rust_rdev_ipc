use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use rdev::{Event, EventType, listen};
use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::{env, thread};

fn start_child_loop() {
    loop {
        let (server, server_name) = IpcOneShotServer::<String>::new().unwrap();
        println!("Starting server on {}", server_name);
        let exe = env::current_exe().unwrap();
        let mut child = Command::new(&exe)
            .arg("--child")
            .arg(server_name)
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to spawn child");

        let (rx, _) = server.accept().unwrap();

        let (kill_tx, kill_rx): (Sender<()>, Receiver<()>) = mpsc::channel();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(30));
            let _ = kill_tx.send(());
        });

        loop {
            if kill_rx.try_recv().is_ok() {
                println!("Timeout, killing child...");
                let _ = child.kill();
                let _ = child.wait();
                println!("Child killed & collected");
                break;
            }

            match rx.try_recv_timeout(Duration::from_millis(2000)) {
                Ok(msg) => handle_rdev_msg(msg),
                Err(e) => {
                    println!("Error receiving from child process: {}", e);
                    continue;
                }
            }
        }
    }
}

fn handle_rdev_msg(msg: String) {
    println!("[main] got: {}", msg)
}

fn main() {
    if env::args().any(|s| s == "--child") {
        run_child();
        return;
    }

    start_child_loop();
}

fn run_child() {
    let args: Vec<String> = env::args().collect();
    let server_name = args.get(2).expect("no server name").to_string();
    let tx: IpcSender<String> = IpcSender::connect(server_name).unwrap();

    let callback = move |event: Event| {
        match event.event_type {
            EventType::ButtonPress(button) => {
                let json = json!(format!("{:?}", button));
                tx.send(json.to_string()).unwrap();
            }
            EventType::ButtonRelease(button) => {
                let json = json!(format!("{:?}", button));
                tx.send(json.to_string()).unwrap();
            }
            EventType::MouseMove { x, y } => {
                let json = json!({ "x": x, "y": y });
                tx.send(json.to_string()).unwrap();
            }
            EventType::KeyPress(key) => {
                let json = json!(format!("{:?}", key));
                tx.send(json.to_string()).unwrap();
            }
            EventType::KeyRelease(key) => {
                let json = json!(format!("{:?}", key));
                tx.send(json.to_string()).unwrap();
            }
            _ => {}
        };
    };

    if let Err(e) = listen(callback) {
        eprintln!("Device listening error: {:?}", e);
    }
}
