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
                    "rdev" => {
                        handle_rdev_msg(ipc_event.json);
                    }
                    _ => {
                        println!("unknown action: {}", ipc_event.action);
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(e) => {
                    eprintln!("Error receiving from child process: {}", e);
                    match child.try_wait() {
                        Ok(Some(_)) => { /* 子进程已经退出，无需kill */ }
                        Ok(None) => {
                            // 子进程还活着，需要 kill
                            match child.kill() {
                                Ok(_) => eprintln!("Force killed child due to channel error"),
                                Err(err) => eprintln!("Fail to kill child (maybe gone): {:?}", err),
                            }
                        }
                        Err(err) => {
                            eprintln!("don't need kill, 检测出错: {}", err)
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("无需 kill, 检测出错: {}", e);
                break;
            }
        }
    }
}

fn handle_rdev_msg(json: String) {
    println!("[main] got: {}", json)
}
