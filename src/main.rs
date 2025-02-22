use std::{
    fs,
    io::{prelude::*, Read, Write},
    net::{TcpListener, TcpStream},
};
use percent_encoding::percent_decode_str;
use htmlescape::decode_html;
use memchr::memmem;
use delete::{delete_file};

// use std::thread;
use std::time::Duration;
// use std::path::Path;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    fs::create_dir_all("uploads").unwrap(); // Create uploads directory
    fs::create_dir_all("data").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // println!("stream1 = {:?}", stream);
        handle_connection(stream);
    }
    println!("idfk what happens here");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = vec![0u8; 4096]; // Fixed-size buffer
    let mut received_data = Vec::new(); // Growable vector (this is what u should give forward)
                                        // stream.set_read_timeout(Some(Duration::from_millis(4000)));
    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        println!("bytes_read = {}", bytes_read);

        if bytes_read == 0 {
            break;
        }

        received_data.extend_from_slice(&buffer[..bytes_read]);
        println!("Request: {}", String::from_utf8_lossy(&received_data[..]));

        if received_data[..3] == *b"GET" && bytes_read < buffer.len() {
            get_method(stream, received_data);
            break;
        }
        if received_data[..4] == *b"POST" && bytes_read < buffer.len() {
            post_method(stream, received_data);
            break;
        }
    }
}

fn get_method(mut stream: TcpStream, mut buffer: Vec<u8>) {
  
    if &buffer[..6] == b"GET / " ||  
       &buffer[..16] == b"GET /favicon.ico" ||
       &buffer[..11] == b"GET /upload"{
        let status_line = "HTTP/1.1 200 OK\r\n";
    
        println!("Done with the GET request my guy");
    
        let response = format!("{}{}", status_line, web());
        // println!("{}", response);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
    // else {
    //     // (mut stream: TcpStream, mut buffer: Vec<u8>, entries: Vec<String>)
    //     send_file(stream, buffer);
    // }

    
}

fn post_method(mut stream: TcpStream, mut buffer: Vec<u8>) {  

    if let Some(action) = memmem::find(&buffer[..], b"action=").map(|p| p as usize){
        post_action(stream, buffer, action);
    }
    else {
        upload_file(stream, buffer);
    }
    
}

fn upload_file(mut stream: TcpStream, mut buffer: Vec<u8>) {
    let buffer = &buffer[..];
    
    // println!("buffer in upload_file={}", String::from_utf8_lossy(&buffer[..]));

    let boundary_b = memmem::find(buffer, b"boundary=").map(|pos| pos as usize).unwrap();
    let boundary_b = &buffer[boundary_b + "boundary=".len()..];
    let boundary_right = memmem::find(boundary_b, b"\r\n").map(|pos| pos as usize).unwrap();
    let boundary = &boundary_b[..boundary_right];
    let boundary = format!("--{}", String::from_utf8_lossy(&boundary[..])).into_bytes();

    println!("boundary={}", String::from_utf8_lossy(&boundary[..]));
    let mut content_boundary = memmem::find_iter(buffer, &boundary).map(|p| p as usize).next().unwrap();
    let info = &buffer[content_boundary + boundary.len()..];

    let contents_find = memmem::find_iter(info, b"\r\n\r\n").map(|p| p as usize).next().unwrap();
    let content = &info[contents_find + b"\r\n\r\n".len().. info.len() - (boundary.len() + 4)];
    let info = &info[..contents_find];

    let mut content_type = memmem::find_iter(buffer, b"Content-Type:").map(|p| p as usize);
    let _ = content_type.next();
    let content_type = content_type.next().unwrap();
    let content_type = &buffer[content_type + "Content-Type:\"".len()..];
    let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let content_type = &content_type[..end];

    println!("Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

    let filename = memmem::find_iter(info, b"filename=").map(|p| p as usize).next().unwrap();
    let filename_data = &info[filename + "filename=".len()..];
    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
    let filename_1 = filename1.next().unwrap();
    let filename_2 = filename1.next().unwrap();
    let filename = &filename_data[filename_1 + 1.. filename_2];
    let filename_upload = format!("uploads/{}",String::from_utf8_lossy(&filename[..]));
    let mut file = fs::File::create(&filename_upload).unwrap();
    file.write_all(content);

    let filename_data = format!("data/{}",String::from_utf8_lossy(&filename[..]) );
    let mut file2 = fs::File::create(format!("{}.txt", &filename_data)).unwrap();

    file2.write_all(&format!("Content-Type:{}",String::from_utf8_lossy(&content_type[..])).into_bytes()[..]);//idk how this works
    //till here we saved the file on the server (hopefully)

    let status_line = "HTTP/1.1 200 OK\r\n";    

    println!("\n\nDone with the POST request my guy");
    let response = format!("{}{}", status_line, web());
    
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn web() ->  String {
    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    let mut html = String::from("    
<!DOCTYPE html>
<html lang=\"en\">
<head>
<meta charset=\"UTF-8\">
<title>File Upload</title>
</head>

<style>
    li{
        display: flex;
        padding: 10px;
    }


    li > form > button{
        margin: 0 10px;
    }
</style>

<body>
<h1>Hello!</h1>
<p>Hi from Rust</p>
<h1>POST REQUEST DONE</h1>

<form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">
    <input type=\"file\" name=\"file\" required>
    <button type=\"submit\">Upload</button>
</form>
<h2> Saved Files:</h2>
<ul>");
    
    for i in 0..file_names.len(){
        html.push_str(&*format!(
            "<li> 
                {}
                <form action=\"/\" method =\"POST\">
                    <input type=\"hidden\" name=\"action\" value=\"DELETE\">
                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                    <button type=\"submit\">Delete</button>
                </form>
                <form action=\"/\" method =\"POST\">
                    <input type=\"hidden\" name=\"action\" value=\"DOWNLOAD\">
                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                    <button type=\"submit\">DOWNLOAD</button>
                </form>
            </li>\n", 
            file_names[i],
            file_names[i],
            file_names[i]
        ));         
    }

    html.push_str("
        </ul>
        </body>
        </html>");

    return html
}

fn post_action(mut stream: TcpStream, mut buffer: Vec<u8>, action: usize){
    let action = &buffer[action + "action=".len()..];

    let filename = memmem::find(action, b"&").map(|p| p as usize).unwrap();
    let filename = &action[filename + 1 + "filename=".len()..];

    let filename = percent_decode_str(&*String::from_utf8_lossy(&filename[..]))
                    .decode_utf8_lossy()
                    .replace("+", " ");

    if action[..6] == *b"DELETE" {
        delet(stream, filename);
    }
    else {
        download(stream, filename, buffer);
        // stream.write(response.as_bytes()).unwrap();
        // stream.write(&read[..]).unwrap();
        // stream.flush().unwrap();
    }

}

fn delet(mut stream: TcpStream, filename: String){
    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names = Vec::new();

    let mut delete = 0;
    for i in entries {
        let entry = i.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    for i in 0..file_names.len(){
        if filename == file_names[i] {
            delete = i;
        }
    }

    delete_file(&*format!("uploads/{}", file_names[delete]) ).unwrap();
    delete_file(&*format!("data/{}.txt", file_names[delete]) ).unwrap();

    let status_line = "HTTP/1.1 200 OK\r\n";    

    println!("\n\nDone with the POST request my guy");
    let response = format!("{}{}", status_line, web());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn download(mut stream: TcpStream, filename: String, buffer: Vec<u8>){
    let buffer = &buffer[..];

    let entries = fs::read_dir("uploads").unwrap();

    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
        println!("entry = {:#?}", entry)
    }

    let mut f = "idk";

    for i in 0..file_names.len() {
        if filename == file_names[i] {
            f = &file_names[i];
        }
    } 

    let mut file = fs::File::open(format!("uploads/{}", f )).unwrap();
    let mut data = fs::File::open(format!("data/{}.txt", f )).unwrap();

    println!("{}", format!("uploads/{}", filename));
    

    // if file == None || data == None {
    //     panic!("Something went wrong with getting your data");
    // }
    
    let mut read = Vec::new();
    file.read_to_end(&mut read).unwrap();

    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    let status_line = "HTTP/1.1 200 OK\r\n";

    let response = format!("{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
            status_line, 
            content_type,
            decode_html(&filename).unwrap(),
            read.len()
            
    );

    stream.write(response.as_bytes()).unwrap();
    stream.write(&read[..]).unwrap();
    stream.flush().unwrap();

}

//this function works fine as it is, DONT U DARE CHANGE SMTH ABT IT
fn send_file(mut stream: TcpStream, mut buffer: Vec<u8>) {
    let buffer = &buffer[..];

    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
        println!("entry = {:#?}", entry)
    }

    //return file \/
    let index = memmem::find(buffer, b"/").map(|p| p as usize).unwrap();
    let index = &buffer[index + 1 ..];
    let end = memmem::find(index, b" ").map(|p| p as usize).unwrap();
    let index = String::from_utf8_lossy(&index[..end]);
    let index = index.parse::<usize>().unwrap();
    
    let mut file = fs::File::open(format!("uploads/{}",file_names[index - 1])).unwrap();
    let mut data = fs::File::open(format!("data/{}.txt", file_names[index - 1])).unwrap();

    // println!("try = {}", file_names[index - 1]);
    let mut read = Vec::new();
    file.read_to_end(&mut read).unwrap();

    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    println!("type of file = {:?}", content_type);

    let status_line = "HTTP/1.1 200 OK\r\n";

    let response = format!("{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
            status_line, 
            content_type,
            file_names[index - 1],
            read.len()
    );

    // println!("\n\nresponse = {:?}", response);
    println!("Done with the GET file request my guy\r\n\r\n");

    // when i click on the text it simply displays the  text in browser instead of saving it 
    //and also change how the link works, instead to save it as (index) save as (filename)
    
    stream.write(response.as_bytes()).unwrap();
    stream.write(&read[..]).unwrap();
    stream.flush().unwrap();
}


