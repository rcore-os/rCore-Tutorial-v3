#![no_std]
#![no_main]

use alloc::string::{String, ToString};

#[macro_use]
extern crate user_lib;
#[macro_use]
extern crate alloc;

// use http://localhost:6201/ to access the http server

use user_lib::{accept, listen, read, write};

// get url from the tcp request list.
fn get_url_from_tcp_request(req: &[u8]) -> String {
    let mut index = 0;
    for i in 4..req.len() {
        if req[i] == 0x20 {
            index = i;
            break;
        }
    }

    String::from_utf8_lossy(&req[4..index]).to_string()
}

// just receive GET requests
fn handle_tcp_client(client_fd: usize) -> bool {
    // a buf to receive the data from the server
    let mut buf = vec![0u8; 1024];

    let len = read(client_fd as usize, &mut buf);

    println!("receive {} bytes", len);
    hexdump(&buf[..len as usize]);

    // verify whether it is a valid HTTP request simply, [0x47,0x45,0x54, 0x20] is GET
    if len < 4 || buf[..4] != [0x47, 0x45, 0x54, 0x20] {
        println!("it's not a valid http request");
        return false;
    }

    let url = get_url_from_tcp_request(&buf);

    if url == "/close" {
        let content = r#"<!DOCTYPE html>
        <html>
        <head>
        <title></title>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link href="https://cdn.staticfile.org/twitter-bootstrap/5.1.1/css/bootstrap.min.css" rel="stylesheet">
        <script src="https://cdn.staticfile.org/twitter-bootstrap/5.1.1/js/bootstrap.bundle.min.js"></script>
        </head>
        <body>
        
        <div class="container-fluid p-5 bg-danger text-white text-center">
        <h1>server closed</h1>
        </div>
        </body>
        </html>"#;
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnecion: Close\r\n\r\n{}", content.len(),content);
        write(client_fd, response.as_bytes());
        // terminate the connection immediately.
        return true;
    }

    let content = r#"<!DOCTYPE html>
        <html>
        <head>
        <title></title>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link href="https://cdn.staticfile.org/twitter-bootstrap/5.1.1/css/bootstrap.min.css" rel="stylesheet">
        <script src="https://cdn.staticfile.org/twitter-bootstrap/5.1.1/js/bootstrap.bundle.min.js"></script>
        </head>
        <body>
        
        <div class="container-fluid p-5 bg-primary text-white text-center">
        <h1>rCore-tutorial-V3</h1>
        <p>rCore-tutorial-V3 是一个 基于 RISC-V 架构的 类 Unix 内核.</p> 
        </div>
        
        <div class="container mt-5">
        <div class="row">
            <div class="col-sm-4">
            <h3>Rust</h3>
            <p>Rust</p>
            <p>Rust是一门系统编程语言，专注于安全，尤其是并发安全，支持函数式和命令式以及泛型等编程范式的多范式语言</p>
            </div>
            <div class="col-sm-4">
            <h3>仓库地址</h3>        
            <p>repo url</p>
            <p>https://github.com/rcore-os/rCore-Tutorial-v3</p>
            </div>
            <div class="col-sm-4">
            <h3>QQ 群号</h3>        
            <p>Official QQ group number</p>
            <p>735045051</p>
            </div>
        </div>
        </div>
        
        <div class="container p-5 text-black text-center d-grid col-sm-4">
        <p>点击下列按钮即可关闭服务器。</p>
        <button type="button" class="btn btn-warning btn-block p-3" onclick="close_server()">关闭 server</button>
        </div>
        <script>
        function close_server() {
            location.href = "/close";
        }
        </script>
        </body>
        </html>"#;

    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnecion: Close\r\n\r\n{}", content.len(),content);

    // write a response
    write(client_fd, response.as_bytes());

    false
}

#[no_mangle]
pub fn main() -> i32 {
    println!("This is a very simple http server");

    let tcp_fd = listen(80);

    if tcp_fd < 0 {
        println!("Failed to listen on port 80");
        return -1;
    }

    loop {
        let client = accept(tcp_fd as usize);
        println!("client connected: {}", client);

        if client < 1 {
            println!("Failed to accept a client on port 80");
            return -1;
        }

        if handle_tcp_client(client as usize) {
            break;
        }
    }

    println!("finish tcp test");

    // String::from_utf8_lossy(&buf[..len as usize])

    0
}

#[allow(unused)]
pub fn hexdump(data: &[u8]) {
    const PRELAND_WIDTH: usize = 70;
    println!("{:-^1$}", " hexdump ", PRELAND_WIDTH);
    for offset in (0..data.len()).step_by(16) {
        for i in 0..16 {
            if offset + i < data.len() {
                print!("{:02x} ", data[offset + i]);
            } else {
                print!("{:02} ", "");
            }
        }

        print!("{:>6}", ' ');

        for i in 0..16 {
            if offset + i < data.len() {
                let c = data[offset + i];
                if c >= 0x20 && c <= 0x7e {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            } else {
                print!("{:02} ", "");
            }
        }

        println!("");
    }
    println!("{:-^1$}", " hexdump end ", PRELAND_WIDTH);
}
