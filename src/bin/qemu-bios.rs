use std::process;
use std::process::Command;

fn main() {
    let mut command = Command::new("qemu-system-x86_64");
    command.arg("-drive");
    command.arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));
    let exit_status = command.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}