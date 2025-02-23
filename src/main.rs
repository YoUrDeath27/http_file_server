use std::{
    fs,
    io::{prelude::*, Read, Write},
    net::{TcpListener, TcpStream},
};
use percent_encoding::percent_decode_str;
use htmlescape::decode_html;
use memchr::memmem;
use delete::{delete_file, delete_folder};

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
        // println!("Request: {}", String::from_utf8_lossy(&received_data[..]));

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

    // println!("boundary={}", String::from_utf8_lossy(&boundary[..]));
    let mut content_boundary = memmem::find_iter(buffer, &boundary).map(|p| p as usize).next().unwrap();
    let info = &buffer[content_boundary + boundary.len()..];

    let mut contents_find = memmem::find_iter(info, b"\r\n\r\n").map(|p| p as usize);
    let _ = contents_find.next();
    let contents_find = contents_find.next().unwrap();
    let content = &info[contents_find + b"\r\n\r\n".len().. info.len() - (boundary.len() + 4)];
    let info = &info[..contents_find];

    let mut content_type = memmem::find_iter(buffer, b"Content-Type:").map(|p| p as usize);
    if let Some(_) = memmem::find(buffer, b"folder=").map(|p| p as usize) {
        let _ = content_type.next();
    }
    let content_type = content_type.next().unwrap();
    let content_type = &buffer[content_type + "Content-Type:\"".len()..];
    let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let content_type = &content_type[..end];

    println!("Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

    // println!("\n\ninfo = {}", String::from_utf8_lossy(&info[..]));
    let filename = memmem::find_iter(info, b"filename=").map(|p| p as usize).next().unwrap();
    let filename_data = &info[filename + "filename=".len()..];

    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
    let filename_1 = filename1.next().unwrap();
    let filename_2 = filename1.next().unwrap();
    let filename = &filename_data[filename_1 + 1.. filename_2];
    println!("filename = {:?}", String::from_utf8_lossy(&filename[..]));

    if let Some(folder) = memmem::find(buffer, b"name=\"folder\"").map(|p| p as usize) {
        let folder = &buffer[folder + "name=\"folder\"\r\n\r\n".len()..];
        let end = memmem::find(folder, b"\r\n").map(|p| p as usize).unwrap();
        let folder = &folder[..end];
        println!("folder = {}", String::from_utf8_lossy(&folder[..]));
        let filename = format!("{}/{}",
            String::from_utf8_lossy(&folder[..]),
            String::from_utf8_lossy(&filename[..])
        );

        println!("file path = {}", filename);
        
        let filename_upload = format!("uploads/{}",filename);
        let mut file = fs::File::create(&filename_upload).unwrap();
        file.write_all(content);

        let filename_data = format!("data/{}",filename);
        let mut file2 = fs::File::create(format!("{}.txt", &filename_data)).unwrap();

        let end = memmem::find(content_type, b";").map(|p| p as usize).unwrap();
        let content_type = &content_type[..end];

        println!("data ={}", String::from_utf8_lossy(&content_type[..]));
        file2.write_all(&format!("Content-Type:{}",String::from_utf8_lossy(&content_type[..])).into_bytes()[..]);//idk how this works
        //till here we saved the file on the server (hopefully)
        
        let status_line = "HTTP/1.1 200 OK\r\n";    
    
        println!("\n\nDone with the POST request my guy");
        let response = format!("{}{}", status_line, web());
        
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {

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
}

fn post_action(mut stream: TcpStream, mut buffer: Vec<u8>, action: usize){
    let action = &buffer[action + "action=".len()..];

    let filename = memmem::find(action, b"&").map(|p| p as usize).unwrap();
    let filename = &action[filename + 1 + "filename=".len()..];

    let filename = percent_decode_str(&*String::from_utf8_lossy(&filename[..]))
                    .decode_utf8_lossy()
                    .replace("+", " ");

    println!("\n1action ={}", String::from_utf8_lossy(&action[..10]));

    if action[..6] == *b"DELETE" {
        println!("Deleted something");
        delet(stream, filename, buffer);
    } else if action[..10] == *b"ADD_FOLDER" {
        println!("Added a folder");
        add_folder(stream, filename); // implement this 
    } else if action[..8] == *b"DOWNLOAD" {
        println!("Downloaded a file");
        download(stream, filename, buffer);
    }   

    // println!("did one of the requests");

}

fn delet(mut stream: TcpStream, filename: String, buffer: Vec<u8>){
    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names = Vec::new();

    let mut delete = 0;
    for i in entries {
        let entry = i.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    // for i in 0..file_names.len(){
    //     if filename == file_names[i] {
    //         delete = i;
    //     }
    // }

    println!("\nfile? {}", filename);
    // println!("file = {}", file_names[delete]); //action=DELETE&filename=anime%2Ftest.txt

    if let Some(folder) = memmem::find(&buffer[..], b"folder=") {
        let file = memmem::find(&buffer[..], b"filename=").map(|p| p as usize).unwrap();
        let file = &buffer[file + "filename=".len()..];
        let filename = String::from_utf8_lossy(&file[..]);
        println!("folder ={}", filename);
        delete_folder(&*format!("uploads/{}", filename) ).unwrap();
        delete_folder(&*format!("data/{}", filename) ).unwrap();
    } else {
        delete_file(&*format!("uploads/{}", filename) ).unwrap();
        delete_file(&*format!("data/{}.txt", filename) ).unwrap();
    }

    let status_line = "HTTP/1.1 200 OK\r\n";    

    println!("\n\nDone with the POST request my guy");
    let response = format!("{}{}", status_line, web());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn download(mut stream: TcpStream, filename: String, buffer: Vec<u8>){
    let buffer = &buffer[..];

    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
        println!("entry = {:#?}", entry)
    }

    let mut file = fs::File::open(format!("uploads/{}", filename )).unwrap();
    let mut data = fs::File::open(format!("data/{}.txt", filename )).unwrap();

    println!("{}", format!("uploads/{}", filename));
    
    let mut read = Vec::new();
    file.read_to_end(&mut read).unwrap();

    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    let status_line = "HTTP/1.1 200 OK\r\n";

    println!("filename={}", decode_html(&filename).unwrap());

    let start = memmem::find(filename.as_bytes(), b"/").map(|p| p as usize).unwrap();
    let filename = String::from_utf8_lossy(&filename.as_bytes()[start + 1..]);

    println!("filename={}", filename);
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

fn add_folder(mut stream: TcpStream, filename: String) {
    fs::create_dir_all(format!("data/{}", filename)).unwrap();
    fs::create_dir_all(format!("uploads/{}", filename)).unwrap();

    println!("upload/{}", filename);
    let status_line = "HTTP/1.1 200 OK\r\n";
    
    let response = format!("{}{}", status_line, web());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}


fn web() ->  String {
    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names = Vec::new();
    let mut files = Vec::new();

    for entry in entries{
        let entry = entry.unwrap();
        files.push(entry.path());
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

<form action\"/\" method=\"POST\">
    <input type=\"hidden\" name=\"action\" value=\"ADD_FOLDER\">
    <input type=\"text\" name=\"filename\" required>
    <button type=\"submit\">Add Folder </button>
</form>

<h2> Saved Files:</h2>
<ul>");
    
    for i in 0..file_names.len(){
        if !files[i].is_file() {
            html.push_str(&*format!(
                "<li> 
                    {}
                    <form action=\"/\" method =\"POST\">
                        <input type=\"hidden\" name=\"action\" value=\"DELETE\">
                        <input type=\"hidden\" name=\"folder\" value=\"{}\">
                        <input type=\"hidden\" name=\"filename\" value=\"{}\">
                        <button type=\"submit\">Delete</button>
                    </form>

                    <form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">
                        <input type=\"hidden\" name=\"folder\" value=\"{}\">
                        <input type=\"file\" name=\"file\" required>
                        <button type=\"submit\">Upload</button>
                    </form>

                </li>\n", 
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i]
            )); 
            let folder = &file_names[i];
            let entries = fs::read_dir(format!("uploads/{}",folder)).unwrap();
            let mut file_names = Vec::new();
            let mut files = Vec::new();

            for entry in entries{
                let entry = entry.unwrap();
                files.push(entry.path());
                let file_name = entry.file_name().into_string().unwrap();
                file_names.push(file_name);
            }

            html.push_str("<ul>");
            for i in 0..file_names.len() {
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
                    format!("{}/{}", folder, file_names[i]),
                    format!("{}/{}", folder, file_names[i])
                ));   
            }

            html.push_str("</ul>\n");
        } else {
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
    }

    html.push_str("
        </ul>
        </body>
        </html>");

    return html
}


