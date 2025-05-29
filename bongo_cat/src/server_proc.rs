use crate::protocol::IpcEvent;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, TryRecvError};
use std::env;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

pub fn start_child_loop() {
    loop {
        let (server, server_name) = IpcOneShotServer::<IpcEvent>::new().unwrap();
        println!("Starting server on {}", server_name);

        let exe = env::current_exe().unwrap();
        let child = Command::new(&exe)
            .arg("--rdev")
            .arg(server_name)
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to spawn child");

        println!("started rdev proc");
        let (rx, _) = server.accept().unwrap();
        println!("rdev proc has connected");
        wait_rdev_exit(rx, child);
    }
}

fn wait_rdev_exit(rx: IpcReceiver<IpcEvent>, mut child: Child) {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("子进程已退出，状态码：{}", status);
                break;
            }
            Ok(None) => match rx.try_recv_timeout(Duration::from_millis(100)) {
                Ok(ipc_event) => match ipc_event.action.as_str() {
                    "exit" => {
                        eprintln!("rdev exiting");
                        child.kill().unwrap();
                        break;
                    }
                    _ => {
                        handle_rdev_msg(ipc_event.json);
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(e) => {
                    eprintln!("Error receiving from child process: {}", e);
                }
            },
            Err(e) => {
                eprintln!("检测子进程出错: {}", e);
                break;
            }
        }
    }
}

fn handle_rdev_msg(json: String) {
    println!("[main] got: {}", json)
}
