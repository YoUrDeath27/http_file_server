
use std::{
    io::{prelude::*, Read, Write},
    net::{TcpListener, TcpStream},
    fs,
    path::Path,
    time::Duration
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    fs::create_dir_all("uploads").unwrap(); // Create uploads directory

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0u8; 4096];
    let mut received_data = Vec::new();
    stream.set_read_timeout(Some(Duration::from_secs(30))).unwrap();

    // Read full request
    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 { break; }
        received_data.extend_from_slice(&buffer[..bytes_read]);
        
        // Check for end of headers
        if received_data.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }

    // Determine request type
    if received_data.starts_with(b"POST") {
        post_method(stream, &received_data);
    } else {
        get_method(stream, &received_data);
    }
}

fn get_method(mut stream: TcpStream, request: &[u8]) {
    let path = request
        .splitn(3, |&b| b == b' ')
        .nth(1)
        .and_then(|p| std::str::from_utf8(p).ok())
        .unwrap_or("/");
    
    let file_path = if path == "/" { 
        "index.html" 
    } else { 
        &path[1..] // Remove leading slash
    };

    let (status, content) = match fs::read(file_path) {
        Ok(data) => ("200 OK", data),
        Err(_) => ("404 NOT FOUND", fs::read("404.html").unwrap_or_default())
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n",
        status,
        content.len()
    ).into_bytes();

    stream.write_all(&response).unwrap();
    stream.write_all(&content).unwrap();
    stream.flush().unwrap();
}
fn post_method(mut stream: TcpStream, request: &[u8]) {
    // Debug: Print raw request
    println!("RAW POST REQUEST:\n{}", String::from_utf8_lossy(request));

    // Extract boundary
    let boundary = request
        .split(|&b| b == b'\r')
        .find(|slice| slice.starts_with(b"Content-Type: multipart/form-data; boundary="))
        .and_then(|header| {
            header.splitn(2, |&b| b == b'=')
                .nth(1)
                .map(|b| String::from_utf8_lossy(b).trim_matches('"').to_string())
        });

    let boundary = match boundary {
        Some(b) => b,
        None => {
            send_error(stream, 400, "Missing boundary");
            return;
        }
    };

    // Extract filename
    let filename = request
        .split(|&b| b == b'\r')
        .find(|slice| slice.starts_with(b"Content-Disposition: form-data; name=\"file\"; filename="))
        .and_then(|line| {
            line.split(|&b| b == b'"')
                .nth(1)
                .map(|b| sanitize_filename(&String::from_utf8_lossy(b)))
        });

    // Extract file content
    let content = request
        .split(|&b| b == b'\r')
        .skip_while(|slice| !slice.is_empty())
        .nth(1)
        .unwrap_or_default();

    match filename {
        Some(filename) => {
            if filename.contains("..") {
                send_error(stream, 400, "Invalid filename");
                return;
            }
            
            match fs::write(format!("uploads/{}", filename), content) {
                Ok(_) => send_response(stream, 200, "File uploaded successfully"),
                Err(e) => send_error(stream, 500, &format!("Server error: {}", e)),
            }
        }
        None => send_error(stream, 400, "Missing filename"),
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .replace("..", "")
        .replace("/", "_")
        .replace("\\", "_")
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect()
}

fn send_response(mut stream: TcpStream, code: u16, message: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status_line(code),
        message.len(),
        message
    );
    stream.write_all(response.as_bytes()).unwrap();
}

fn send_error(mut stream: TcpStream, code: u16, message: &str) {
    let body = format!("<h1>Error {}: {}</h1>", code, message);
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status_line(code),
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).unwrap();
}

fn status_line(code: u16) -> &'static str {
    match code {
        200 => "200 OK",
        400 => "400 Bad Request",
        404 => "404 Not Found",
        500 => "500 Internal Server Error",
        _ => "500 Internal Server Error"
    }
}


