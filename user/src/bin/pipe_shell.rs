// https://www.joshmcguigan.com/blog/build-your-own-shell-rust/
// https://github.com/JoshMcguigan/bubble-shell
// https://github.com/psinghal20/rush
#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

use alloc::string::String;
use alloc::vec::Vec;
use user_lib::console::{getchar};
use user_lib::{close, dup, exec, fork, open, pipe, waitpid, OpenFlags};

const STDIN: usize = 0;
const STDOUT: usize = 1;

#[derive(Debug, Clone)]
enum IOType {
    File(String),
    // [read end, write end]
    Pipe([usize; 2]),
    Inherit,
}

impl IOType {
    pub fn new_pipe() -> Self {
        let mut pipe_fd: [usize; 2] = [0; 2];
        pipe(&mut pipe_fd);
        Self::Pipe(pipe_fd)
    }
}

#[derive(Debug, Clone)]
struct Process {
    pub pid: isize,
    pub output: IOType,
}

#[no_mangle]
pub fn main() -> i32 {
    println!("Rust user shell (with pipe)");
    let mut line: String = String::new();
    print!(">) ");
    //flush();
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    let mut commands = line.trim().split(" | ").peekable();

                    let mut previous_process: Option<Process> = None;
                    while let Some(command) = commands.next() {
                        let args: Vec<_> = command.trim().split(' ').collect();
                        let mut args_copy: Vec<String> = args
                            .iter()
                            .map(|&arg| {
                                let mut string = String::new();
                                string.push_str(arg);
                                string.push('\0');
                                string
                            })
                            .collect();

                        // redirect input
                        let input = if previous_process.is_some() {
                            previous_process.clone().unwrap().output
                        } else if let Some((idx, _)) = args_copy
                            .iter()
                            .enumerate()
                            .find(|(_, arg)| arg.as_str() == "<\0")
                        {
                            let tmp = IOType::File(args_copy[idx + 1].clone());
                            args_copy.drain(idx..=idx + 1);
                            tmp
                        } else {
                            IOType::Inherit
                        };

                        // redirect output
                        let output = if commands.peek().is_some() {
                            IOType::new_pipe()
                        } else if let Some((idx, _)) = args_copy
                            .iter()
                            .enumerate()
                            .find(|(_, arg)| arg.as_str() == ">\0")
                        {
                            let tmp = IOType::File(args_copy[idx + 1].clone());
                            args_copy.drain(idx..=idx + 1);
                            tmp
                        } else {
                            IOType::Inherit
                        };

                        let mut args_addr: Vec<*const u8> =
                            args_copy.iter().map(|arg| arg.as_ptr()).collect();
                        args_addr.push(0 as *const u8);
                        let pid = fork();
                        if pid == 0 {
                            // input redirection
                            match input.clone() {
                                IOType::File(filename) => {
                                    let input_fd = open(filename.as_str(), OpenFlags::RDONLY);
                                    if input_fd == -1 {
                                        println!("Error when opening file {}", filename);
                                        return -4;
                                    }
                                    let input_fd = input_fd as usize;
                                    close(STDIN);
                                    assert_eq!(dup(input_fd) as usize, STDIN);
                                    close(input_fd);
                                }
                                IOType::Pipe(pipes) => {
                                    close(pipes[1]); // close write end
                                    close(STDIN);
                                    assert_eq!(dup(pipes[0]) as usize, STDIN);
                                    close(pipes[0]);
                                }
                                _ => {}
                            }
                            // output redirection
                            match output.clone() {
                                IOType::File(filename) => {
                                    let output_fd = open(
                                        filename.as_str(),
                                        OpenFlags::CREATE | OpenFlags::WRONLY,
                                    );
                                    if output_fd == -1 {
                                        println!("Error when opening file {}", filename);
                                        return -4;
                                    }
                                    let output_fd = output_fd as usize;
                                    close(STDOUT);
                                    assert_eq!(dup(output_fd) as usize, STDOUT);
                                    close(output_fd);
                                }
                                IOType::Pipe(pipes) => {
                                    close(pipes[0]); // close read end
                                    close(STDOUT);
                                    assert_eq!(dup(pipes[1]) as usize, STDOUT);
                                    close(pipes[1]);
                                }
                                _ => {}
                            }
                            // run child process
                            if exec(args_copy[0].as_str(), args_addr.as_slice()) == -1 {
                                println!("Error when executing!");
                                return -4;
                            }
                            unreachable!("Should not happen!");
                        }
                        // old pipe was done, close it.
                        if let Some(p) = previous_process {
                            match p.output {
                                IOType::Pipe(pipes) => {
                                    close(pipes[0]);
                                    close(pipes[1]);
                                }
                                _ => {}
                            }
                        }
                        previous_process = Some(Process { pid, output });
                    }
                    line.clear();
                    if let Some(process) = previous_process {
                        let mut exit_code: i32 = 0;
                        let pid = process.pid;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                }
                print!(">) ");
                //flush();
            }
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char);
                    //flush();
                    line.pop();
                }
            }
            _ => {
                print!("{}", c as char);
                //flush();
                line.push(c as char);
            }
        }
    }
}
