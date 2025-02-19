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
    let bytes_buffer = &buffer[..];

    let boundary_b = memmem::find(bytes_buffer, b"boundary=").map(|pos| pos as usize).unwrap();
    let boundary_b = &bytes_buffer[boundary_b + "boundary=".len()..];
    let boundary_right = memmem::find(boundary_b, b"\r\n").map(|pos| pos as usize).unwrap();
    let boundary = &boundary_b[..boundary_right];
    let boundary = format!("--{}", String::from_utf8_lossy(&boundary[..])).into_bytes();

    let mut content_boundary = memmem::find_iter(bytes_buffer, &boundary).map(|p| p as usize).next().unwrap();
    let info = &bytes_buffer[content_boundary + boundary.len()..];

    let contents_find = memmem::find_iter(info, b"\r\n\r\n").map(|p| p as usize).next().unwrap();
    
    let content = &info[contents_find + b"\r\n\r\n".len().. info.len() - (boundary.len() + 4)];
    let info = &info[..contents_find];

    // println!("contents = {}", String::from_utf8_lossy(&content[..]));
    // println!("info = {}", String::from_utf8_lossy(&info[..]));

    let filename = memmem::find_iter(info, b"filename=").map(|p| p as usize).next().unwrap();
    let filename_data = &info[filename + "filename=".len()..];
    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);

    let filename_1 = filename1.next().unwrap();
    let filename_2 = filename1.next().unwrap();

    let filename = &filename_data[filename_1 + 1.. filename_2];

    let filename = format!("uploads/{}",String::from_utf8_lossy(&filename[..]));

    let mut file = fs::File::create(filename).unwrap();
    
    file.write_all(content);

    let mut f = fs::File::open("POST.html").unwrap();

    let status_line = "HTTP/1.1 200 OK\r\n\r\n";
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();

    println!("\n\nDone with the POST request my guy");
    let response = format!("{}{}", status_line, contents);
    // println!("{}", response);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
