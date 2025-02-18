use std::{
    fs,
    io::{prelude::*, Read, Write},
    net::{TcpListener, TcpStream},
};
use memchr::memmem;

// use std::thread;
use std::time::Duration;
// use std::path::Path;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    fs::create_dir_all("uploads").unwrap(); // Create uploads directory

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("stream1 = {:?}", stream);
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = vec![0u8; 2048]; // Fixed-size buffer
    let mut received_data = Vec::new(); // Growable vector (this is what u should give forward)
                                        // stream.set_read_timeout(Some(Duration::from_millis(4000)));
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        received_data.extend_from_slice(&buffer[..bytes_read]);

        // Check if we've received the full headers
        // if received_data.windows(4).any(|window| {
        //                                         println!("window?    {:?}",window );
        //                                         window == b"\r\n\r\n"}) {
        //     println!("in bytes = {:?} \n", b"\r\n\r\n done" );
        //     break;
        // }

        // println!("raw info from buffer:\n {:?}", String::from_utf8_lossy(&buffer[..]));
        // println!("\n\n\nraw info from received data:\n {:?}", String::from_utf8_lossy(&received_data[..]));
        if received_data[received_data.len() - 4..] == *b"\r\n\r\n" {
            get_method(stream, received_data);
            break;
        }
        if received_data[received_data.len() - 2..] == *b"\r\n" {
            println!("\n\n\n\n\n\nGot the post request, chill man\n\n");
            post_method(stream, received_data);
            break;
        }
    }
    // stream.read(&mut buffer).unwrap();

    // println!("\n\n\n\n\n\n\n\nRequest: {} done with data", String::from_utf8_lossy(&buffer[..]));

    // println!("Raw request: {:?}", &buffer[..]);
    // get_method(stream);
    //u did a simple web server
    //now make it a file server
}

fn get_method(mut stream: TcpStream, mut buffer: Vec<u8>) {
    let mut file = fs::File::open("index.html").unwrap();
    let status_line = "HTTP/1.1 200 OK\r\n\r\n";
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    println!("Done with the GET request my guy");

    let response = format!("{}{}", status_line, contents);
    // println!("{}", response);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn post_method(mut stream: TcpStream, mut buffer: Vec<u8>) {
    let mut file = fs::File::open("POST.html").unwrap();

    let status_line = "HTTP/1.1 200 OK\r\n\r\n";
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    // println!("file: {:?}", file);
    // println!("content of file: {}", contents);
    /*
    let contents = String::from("
        <html lang=\"en\">
        <head>
        <meta charset=\"UTF-8\">
        <title>File Upload</title>
        </head>
        <body>
        <h1>Hello!</h1>
        <p>Hi from Rust</p>
        <h1>POST REQUEST DONE</h1>

        <form action=\"/upload\" method=\"POST\" enctype=\"multipart/form-data\">
            <input type=\"file\" name=\"file\" required>
            <button type=\"submit\">Upload</button>
        </form>
        <h3> this is interesting blud </h3>
        </body>
        </html>"); */

    // println!(
    //     "\n\n\n\nRequest raw: {}",
    //     String::from_utf8_lossy(&buffer[..])
    // );
    println!("\n\nDone with the POST request my guy");
    let bytes_buffer = &buffer[..];
    let buffer = String::from_utf8_lossy(&buffer[..]);

    let boundary_b = memmem::find(bytes_buffer, b"boundary=").map(|pos| pos as usize).unwrap();
    let boundary_b = &bytes_buffer[boundary_b + "boundary=".len()..];
    let boundary_right = memmem::find(boundary_b, b"\r\n").map(|pos| pos as usize).unwrap();
    let boundary = &boundary_b[..boundary_right];
    let boundary = format!("--{}", String::from_utf8_lossy(&boundary[..])).into_bytes();

    println!("boundary in bytes = {:?}", String::from_utf8_lossy(&boundary[..]));
    println!("\n\ncontent = {}", String::from_utf8_lossy(&bytes_buffer[..]));

    let mut content_start = memmem::find_iter(bytes_buffer, &boundary).map(|p| p as usize).next().unwrap();
    let content = &bytes_buffer[content_start + boundary.len()..];


    println!("\n\n\ncontent? {}",String::from_utf8_lossy(&content[..]));
    
    
    
    // let boundary = buffer
    //                 .split("boundary=")
    //                 .nth(1)
    //                 .unwrap()
    //                 .split("\r\n")
    //                 .nth(0)
    //                 .unwrap();
    // let boundary = format!("--{}", boundary);

    // let file_content = buffer
    //                     .split(&boundary) // doesnt accept a String so we give a pointer
    //                     .nth(1)
    //                     .unwrap();
    // let data = file_content
    //             .split("\r\n\r\n")
    //             .nth(0)
    //             .unwrap();

    // println!("boundary = {:#?}", boundary);
    // println!("data = {}\n\n", data);
    // // println!("content = {:#?}", file_content);
    // let title = data
    //             .split("filename=")
    //             .nth(1)
    //             .unwrap()
    //             .split("\"")
    //             .nth(1)
    //             .unwrap();
    // println!("title = {}", title);
    // let mut file = fs::File::create(format!("uploads/{}", title)).unwrap();
    
    // file.write_all(bytes_buffer);

    let haystack = b"foo bar foo baz foo";
    let mut it = memmem::find_iter(haystack, b"foo");
    println!("found = {:?}", it.next());
    println!("found = {:?}", it.next());

    let response = format!("{}{}", status_line, contents);
    // println!("{}", response);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
