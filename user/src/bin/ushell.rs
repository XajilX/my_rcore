#![no_std]
#![no_main]

#[macro_use]
extern crate user;
extern crate alloc;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::string::String;
use user::{console::getchar, fork, exec, waitpid};

#[no_mangle]
fn main() -> i32 {
    println!("[ushell] start");
    let mut line = String::new();
    print!("# ");
    loop {
        match getchar() {
            LF | CR => {
                println!("");
                if line.is_empty() {
                    print!("# ");
                    continue;
                }
                match fork() {
                    0 => {
                        if exec(line.as_str()) == -1 {
                            println!("Error when executing");
                            return -4;
                        }
                        unreachable!()
                    }
                    pid => {
                        let mut exit_code = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(exit_pid, pid);
                        println!(
                            "[ushell] Process {} exited with code {}",
                            pid, exit_code
                        )
                    }
                }
                line.clear();
                print!("# ");
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    line.pop();
                }
            }
            c => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
    }
}