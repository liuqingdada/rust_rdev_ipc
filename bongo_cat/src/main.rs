mod protocol;
mod rdev_proc;
mod server_proc;

use crate::rdev_proc::run_child;
use crate::server_proc::start_child_loop;
use std::env;

fn main() {
    if env::args().any(|s| s == "--rdev") {
        run_child();
        return;
    }

    start_child_loop();
}
