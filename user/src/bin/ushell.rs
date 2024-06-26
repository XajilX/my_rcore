#![no_std]
#![no_main]

#[macro_use]
extern crate user;
extern crate alloc;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::{ffi::CString, string::String, vec::Vec};
use user::{close, console::getchar, dup, exec, fork, open, waitpid, OpenFlags};

#[no_mangle]
fn main() -> i32 {
    println!("[ushell] start");
    let mut line = String::new();
    print!("> ");
    loop {
        match getchar() {
            LF | CR => {
                println!("");
                if line.is_empty() {
                    print!("> ");
                    continue;
                }
                let mut args: Vec<_> = line.split(' ').map(|str| {
                    CString::new(str).unwrap()
                }).collect();

                // IO redirect
                let mut input: Option<CString> = None;
                if let Some((idx, _)) = args.iter().enumerate()
                    .find(|(_, cstr)| cstr.to_str() == Ok("<")) {
                        input = Some(args[idx + 1].clone());
                        args.drain(idx..=idx+1);
                }
                let mut output: Option<CString> = None;
                if let Some((idx, _)) = args.iter().enumerate()
                    .find(|(_, cstr)| cstr.to_str() == Ok(">")) {
                        output = Some(args[idx + 1].clone());
                        args.drain(idx..=idx+1);
                }

                let mut args_addr: Vec<_> = args.iter().map(|cstr| {
                    cstr.as_ptr() as *const u8
                }).collect();
                args_addr.push(0 as *const u8);
                match fork() {
                    0 => {

                        if let Some(path) = input {
                            let input_fd = open(path.to_str().unwrap(), OpenFlags::RDONLY);
                            if input_fd == -1 {
                                println!("Error when opening file {}! ", path.to_str().unwrap());
                                return -4;
                            }
                            let input_fd = input_fd as usize;
                            close(0);
                            assert_eq!(dup(input_fd), 0);
                            close(input_fd);
                        }
                        if let Some(path) = output {
                            let output_fd = open(path.to_str().unwrap(), OpenFlags::CREATE | OpenFlags::WRONLY);
                            if output_fd == -1 {
                                println!("Error when opening file {}! ", path.to_str().unwrap());
                                return -4;
                            }
                            let output_fd = output_fd as usize;
                            close(1);
                            assert_eq!(dup(output_fd), 1);
                            close(output_fd);
                        }
                        
                        if exec(args[0].to_str().unwrap(), &args_addr) == -1 {
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
                print!("> ");
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
                if !c.is_ascii_control() {
                    print!("{}", c as char);
                    line.push(c as char);
                }
            }
        }
    }
}