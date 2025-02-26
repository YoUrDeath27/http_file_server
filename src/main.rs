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

const MAX_UPLOAD_SIZE: usize = 40 * 1024 * 1024; // 40MB
const ALLOWED_MIME_TYPES: &[&str] = &[
    "audio/wav",
    "audio/mp3",
    "application/x-zip-compressed",
    "video/mp4",
    "text/plain",
    "image/jpeg",
    "image/png",
    "application/pdf",
    "application/octet-stream"
]; // idk how this works


fn main() {

    let mut port = String::new();

    println!("Choose on which ip the server to listen to \n(e.g. 127.0.0.1:7878)");
    println!("ps: press enter to go with the default");
    std::io::stdin()
                .read_line(&mut port)
                .expect("U are one son of a bitch");

            
        // Trim whitespace and newlines from the input
    let port = port.trim();

    println!("port? {port:?}");

    // Use a default address if the input is empty
    let port = if port == "" {
        println!("No input provided. Using default address: 127.0.0.1:7878");
        "127.0.0.1:7878"
    } else {
        port
    };
    println!("The address you selected is: {:?}", port);


    let listener = match TcpListener::bind(port){
        Ok(p) => p,
        Err(err) => {
            println!("Could not bind to {} : {}", port, err);
            println!("Ensure the port is clear to be used");
            return;
        }
    };
    fs::create_dir_all("uploads").unwrap(); // Create uploads directory
    fs::create_dir_all("data").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // println!("stream1 = {:?}", stream);
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = vec![0u8; 4096]; // Fixed-size buffer
    let mut received_data = Vec::new(); // Growable vector (this is what u should give forward)
                                        // stream.set_read_timeout(Some(Duration::from_millis(4000)));
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(b) => b,
            Err(_) => {
                send_error_response(&mut stream, 500, "Failed to read the request");
                break;
            }
        };

        // println!("bytes_read = {}", bytes_read);

        if received_data.len() > MAX_UPLOAD_SIZE && received_data.len() < buffer.len() {
            println!("File is too big, just like your lil fella");
            send_error_response(&mut stream, 413, "File too large");
            return; // if return breaks the code, change with break;
        }

        if bytes_read == 0 {
            break;
        }

        received_data.extend_from_slice(&buffer[..bytes_read]);
        // println!("Request: {}", String::from_utf8_lossy(&received_data[..]));

        if received_data[..3] == *b"GET" && bytes_read < buffer.len() {
            get_method(stream);
            break;
        }
        if received_data[..4] == *b"POST" && bytes_read < buffer.len() {
            post_method(stream, received_data);
            break;
        }
    }
}

fn get_method(mut stream: TcpStream) {
    let status_line = "HTTP/1.1 200 OK\r\n";

    println!("Done with the GET request my guy");

    let response = format!("{}{}", status_line, web());
    // println!("{}", response);
    if let Err(e) = stream.write_all(response.as_bytes()){
        eprintln!("Write error: {}", e);
    }
    if let Err(e) = stream.flush() {
        eprintln!("Error flushing: {}", e);
    }

    
}

fn post_method(stream: TcpStream, buffer: Vec<u8>) {  

    if let Some(action) = memmem::find(&buffer[..], b"action=").map(|p| p as usize){
        post_action(stream, buffer, action);
    }
    else {
        upload_file(stream, buffer);
    }
    
}

fn upload_file(mut stream: TcpStream, buffer: Vec<u8>) {    
    // println!("buffer in upload_file={}", String::from_utf8_lossy(&buffer[..]));

    let boundary = match get_boundary(&buffer) {
        Some(b) => b,
        None => {
            send_error_response(&mut stream, 400, "Invalid request format");
            return;
        }
    };

    let buffer = &buffer[..];


    // println!("boundary={}", String::from_utf8_lossy(&boundary[..]));

    let (content, content_type, filename) = match parse_file(&mut stream, buffer, &boundary){
        Ok(data) => data,
        Err(e) => {
            send_error_response(&mut stream, 400, &format!("Failed to parse request, {}", e));
            return;
        }
    };

    if !ALLOWED_MIME_TYPES.contains(&content_type) {
        // println!("sontent_type ={}", content_type);
        send_error_response(&mut stream, 400, "Unsuported file type");
        return;
    }

    if let Some(_) = memmem::find(buffer, b"name=\"folder\"").map(|p| p as usize) {
        add_file_in_folder(stream, buffer, content, content_type, filename);
        return;
    }

    add_file(stream, buffer,  content, content_type, filename);
    return;
}

fn get_boundary(buffer: &Vec<u8>) -> Option<Vec<u8>>{
    let buffer = &buffer[..];
    let boundary_b = memmem::find(buffer, b"boundary=").map(|pos| pos as usize).unwrap();
    let boundary_b = &buffer[boundary_b + "boundary=".len()..];
    let boundary_right = memmem::find(boundary_b, b"\r\n").map(|pos| pos as usize).unwrap();
    let boundary = &boundary_b[..boundary_right];
    let boundary = format!("--{}", String::from_utf8_lossy(&boundary[..])).into_bytes();
    // println!("got the boundary as: {}", String::from_utf8_lossy(&buffer[..]));
    Some(boundary)
}

fn parse_file<'a>(stream: &mut TcpStream, buffer:&'a [u8], boundary: &[u8]) -> Result<(&'a [u8], &'a str, String), &'static str>{    
    let content_boundary = match memmem::find_iter(buffer, &boundary).map(|p| p as usize).next(){
        Some(c) => c,
        None => {
            send_error_response(stream, 400, "Content not found");
            return Err("fuck head, cant find the content");
        }
    };
    let info = &buffer[content_boundary + boundary.len()..];

    //the content part
    let mut contents_find = memmem::find_iter(info, b"\r\n\r\n").map(|p| p as usize);
    if let Some(_) = memmem::find(buffer, b"name=\"folder\"").map(|p| p as usize) {
        let _ = contents_find.next();
    }
    let contents_find = match contents_find.next() {
        Some(c) => c,
        None => {
            return Err("Couldn't find the content of the file");
        }
    };
    let content = &info[contents_find + b"\r\n\r\n".len().. info.len() - (boundary.len() + 4)];
    //1
    let info = &info[..contents_find];

    // content-type part
    let mut content_type = memmem::find_iter(buffer, b"Content-Type:").map(|p| p as usize);
    let _ = content_type.next();

    // println!("buffer = {}", String::from_utf8_lossy(&buffer[..]));

    if let Some(_) = memmem::find(buffer, b"name=\"folder\"").map(|p| p as usize) {

        let content_type = content_type.next().unwrap();
        let content_type = &buffer[content_type + "Content-Type:\"".len()..];
    
        // println!("content-type is equal to IDFKK ={}\n\n\n\n", String::from_utf8_lossy(&content_type[..]));

        // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
        let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
        let content_type = &content_type[..end];

        //2 

        // println!("Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

        //filename part
        let filename = memmem::find_iter(info, b"filename=").map(|p| p as usize).next().unwrap();
        let filename_data = &info[filename + "filename=".len()..];

        let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
        let filename_1 = filename1.next().unwrap();
        let filename_2 = filename1.next().unwrap();
        let filename = &filename_data[filename_1 + 1.. filename_2];
        //3
        // println!("filename = {:?}", String::from_utf8_lossy(&filename[..]));
        let file = String::from_utf8_lossy(&filename[..]).to_string();
        
        return Ok((
            content, 
            std::str::from_utf8(content_type).unwrap_or("application/octet-stream"),
            file
        ));
    }
    let content_type = content_type.next().unwrap();
    let content_type = &buffer[content_type + "Content-Type:\"".len()..];

    // println!("content-type is equal to ={}", String::from_utf8_lossy(&content_type[..]));

    // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let content_type = &content_type[..end];

    //2 

    // println!("2Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

    //filename part
    let filename = memmem::find_iter(info, b"filename=").map(|p| p as usize).next().unwrap();
    let filename_data = &info[filename + "filename=".len()..];

    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
    let filename_1 = filename1.next().unwrap();
    let filename_2 = filename1.next().unwrap();
    let filename = &filename_data[filename_1 + 1.. filename_2];
    //3
    // println!("filename = {:?}", String::from_utf8_lossy(&filename_data[..]));
    let file = String::from_utf8_lossy(&filename[..]).to_string();
    
    
    Ok((
        content, 
        std::str::from_utf8(content_type).unwrap_or("application/octet-stream"),
        file
    ))
    
}

fn post_action(stream: TcpStream, buffer: Vec<u8>, action: usize){
    let action = &buffer[action + "action=".len()..];

    let filename = memmem::find(action, b"filename=").map(|p| p as usize).unwrap();
    let filename = &action[filename + "filename=".len()..];

    let filename = percent_decode_str(&*String::from_utf8_lossy(&filename[..]))
                    .decode_utf8_lossy()
                    .replace("+", " ");

    // println!("\n1action ={}", String::from_utf8_lossy(&action[..10]));

    if action[..6] == *b"DELETE" {
        println!("Deleted something");
        delet(stream, filename, buffer);
    } else if action[..10] == *b"ADD_FOLDER" {
        println!("Added a folder");
        add_folder(stream, &buffer[..], filename); // implement this 
    } else if action[..8] == *b"DOWNLOAD" {
        println!("Downloaded a file");
        download(stream, filename);
    }   

    // println!("did one of the requests");

}

fn delet(mut stream: TcpStream, filename: String, buffer: Vec<u8>){
    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names = Vec::new();

    for i in entries {
        let entry = i.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    // println!("\nfile? {}", filename);
    // println!("file = {}", file_names[delete]); //action=DELETE&filename=anime%2Ftest.txt

    // println!("\n\nbuffer={}", String::from_utf8_lossy(&buffer[..]));
    if let Some(folder) = memmem::find(&buffer[..], b"folder=") {

        let file = memmem::find(&buffer[..], b"filename=").map(|p| p as usize).unwrap();
        let file = &buffer[file + "filename=".len()..];
        let filename = String::from_utf8_lossy(&file[..]);
        let filename = decode_html(&*filename)
                .unwrap()
                .replace("+", " ");

        fs::remove_dir_all(&*format!("uploads/{}", filename));
        fs::remove_dir_all(&*format!("data/{}", filename));

    } else { // a folder to delete
        // let filename = decode_html(&*filename).unwrap();
        match delete_file(&*format!("uploads/{}", filename) ){
            Ok(ok) => ok,
            Err(err) => {
                println!("error = {}", err);
                send_error_response(&mut stream, 401, &*format!("(uploads)Unable to delete folder because\r\n {}", err));
                return;
            }
        }
        match delete_file(&*format!("data/{}.txt", filename) ){
            Ok(ok) => ok,
            Err(err) => {
                println!("error = {}", err);
                send_error_response(&mut stream, 401, &*format!("(data)Unable to delete folder because\r\n{}", err));
                return;
            }
        };

    }

    let status_line = "HTTP/1.1 200 OK\r\n";    

    println!("\n\nDone with the POST delete action request my guy");
    let response = format!("{}{}", status_line, web());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn download(mut stream: TcpStream, filename: String){

    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    let mut file = fs::File::open(format!("uploads/{}", filename )).unwrap();
    let mut data = fs::File::open(format!("data/{}.txt", filename )).unwrap();

    println!("{}", format!("download uploads/{}", filename));
    
    let mut read = Vec::new();
    file.read_to_end(&mut read).unwrap();

    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("filename={}", decode_html(&filename).unwrap());

    let start = memmem::find(filename.as_bytes(), b"/").map(|p| p as usize).unwrap();
    let filename = String::from_utf8_lossy(&filename.as_bytes()[start + 1..]);

    // println!("filename={}", filename);
    let response = format!("{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
            status_line, 
            content_type,
            decode_html(&filename).unwrap(),
            read.len()
            
    );
    
    println!("Done with the POST download action my guy");
    stream.write(response.as_bytes()).unwrap();
    stream.write(&read[..]).unwrap();
    stream.flush().unwrap();

}

fn add_folder(mut stream: TcpStream, buffer: &[u8], filename: String) {

    if let Some(nested) = memmem::find(buffer, b"nested=").map(|p| p as usize) {
        let nested = &buffer[nested + 7..];
        println!("nested?: {}", String::from_utf8_lossy(&nested[..]));
        let end = memmem::find(nested, b"&").map(|p| p as usize).unwrap();
        let nested =&nested[..end];

        let filename = format!("{}/{}", String::from_utf8_lossy(&nested[..]), filename);   
        fs::create_dir_all(format!("uploads/{}", filename)).unwrap();
        fs::create_dir_all(format!("data/{}", filename)).unwrap();

        println!("uploads/{}\n\n", filename);
        let status_line = "HTTP/1.1 200 OK\r\n";
        
        let response = format!("{}{}", status_line, web());
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        return;
    };


    fs::create_dir_all(format!("uploads/{}", filename)).unwrap();
    fs::create_dir_all(format!("data/{}", filename)).unwrap();
    

    println!("uploads/{}\n\n", filename);
    let status_line = "HTTP/1.1 200 OK\r\n";
    
    let response = format!("{}{}", status_line, web());
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn add_file_in_folder(mut stream:TcpStream, buffer: &[u8], content: &[u8], content_type: &str, filename: String) {

    let folder = match memmem::find(&buffer[..], b"name=\"folder\"").map(|p| p as usize) {
        Some(f) => f,
        None => {
            send_error_response(&mut stream, 404, "Folder not found");
            return;
        }
    };

    println!("should add a file in da folder");
    let folder = &buffer[folder + "name=\"folder\"".len() + "\r\n\r\n".len()..];

    println!("folder? = {}", String::from_utf8_lossy(&folder[..]));
    let end = memmem::find(folder, b"\r\n").map(|p| p as usize).unwrap();
    let folder = &folder[..end];


    // println!("filename before change = {}", filename);

    let filename = format!("{}/{}", 
        String::from_utf8_lossy(&folder[..]),
        filename
    );

    // println!("filename after change = {}", filename);

    add_file(stream, buffer, content, content_type, filename);

}

fn add_file(mut stream: TcpStream, buffer: &[u8], content: &[u8], content_type: &str, filename: String) {
    // do some shady shit

        let filename_upload = format!("uploads/{}", filename);
        // println!("upload filename ={}", filename_upload);

        let mut file = fs::File::create(&filename_upload).unwrap();
        file.write_all(content);
        
        let filename_data = format!("data/{}",filename);
        // println!("filename_data = {}", filename_data);
        let mut file2 = fs::File::create(format!("{}.txt", &filename_data)).unwrap();
    
        file2.write_all(&format!("Content-Type:{}",content_type).into_bytes()[..]);//idk how this works
        //till here we saved the file on the server (hopefully)
        
        let status_line = "HTTP/1.1 200 OK\r\n";    
    
        println!("\n\nDone with the POST add_file request my guy");
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

        li > div:nth-child(2) {
            margin:0 0 0 50px;
        }
    </style>

    <body>
    <h1>Hello!</h1>
    <p>Welcome to your file server :)</p>

    <form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">
        <input type=\"file\" name=\"file\" id=\"fileInput\" required>
        <button>
            <label for=\"fileInput\" id=\"fileLabel\">Choose a file</label>
        </button>
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
                    <div>
                        {}
                    </div>    
                    <div>
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
                        {} files inside
                        <form action\"/\" method=\"POST\">
                            <input type=\"hidden\" name=\"action\" value=\"ADD_FOLDER\">
                            <input type=\"hidden\" name=\"nested\" value=\"{}\">
                            <input type=\"text\" name=\"filename\" required>
                            <button type=\"submit\">Add Folder </button>
                        </form>
                    </div>
                </li>\n", 
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i],
                {
                    let mut list = Vec::new();
                    // println!("uploads/{}", file_names[i]);
                    let entryes = fs::read_dir(format!("uploads/{}", file_names[i])).unwrap();
                    for entry in entryes {
                        let entry = entry.unwrap();
                        list.push(entry.path())
                    }

                    list.len()
                },
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

fn send_error_response(stream: &mut TcpStream, code: u16, message: &str) {
    let status_line = match code {
        400 => "HTTP/1.1 400 Bad Request",
        403 => "HTTP/1.1 403 Forbidden",
        404 => "HTTP/1.1 404 Not Found",
        // 413 => "HTTP/1.1 413 Payload Too Large",
        500 => "HTTP/1.1 500 Internal Server Error",
        _ => "HTTP/1.1 500 Internal Server Error",
    };

    let response = format!("{}\r\n\r\n{}", status_line, error_web(message));
    // println!("reponse =\n{}", response);
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn error_web(message: &str) -> String {
    let mut html =String::from("
    <!DOCTYPE html>
    <html>
    <head>
    <title> Error processing your request </title>
    </head>
    <style>

        button {
            display: flex;
            margin: auto;
            height: 100px;
            width: 343px;
            font-size: 32px;
            padding: 0;
        }
        
    </style>
    <body>
    ");
    html.push_str(&*format!("<h1> {} </h1>", message));
    html.push_str("
        <button onclick=\"window.location.href='/'\"> Go back to the main page </button>
    ");

    html.push_str(" </body> </html>");

    html
}