use delete::{delete_file, delete_folder};
use htmlescape::decode_html;
use memchr::memmem;
use percent_encoding::percent_decode_str;
use std::{
    fs,
    sync::Mutex,
    path::{Path, PathBuf},
    io::{prelude::*, Read, Write},
    net::{TcpListener, TcpStream},
};

use lazy_static::lazy_static;
use zip::write::SimpleFileOptions;
use walkdir::WalkDir;

use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};


// use std::thread;
// use std::time::Duration;
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
    "application/octet-stream",
]; // idk how this works

//------------------------------------------------------------------------
//keep testing your server blud
//------------------------------------------------------------------------

lazy_static!{
    static ref SHOW_FOLDER: Mutex<String> = Mutex::new(String::from(""));
}

fn decode_Windows_1255(bytes: &[u8]) -> String{
    // Try UTF-8 first
    if let Ok(utf8_str) = String::from_utf8(bytes.to_vec()) {
        return utf8_str;
    }
    
    // Fall back to Windows-1252
    WINDOWS_1252.decode(bytes, DecoderTrap::Replace).unwrap_or_else(|_| String::from("Invalid encoding"))
}
fn main() {
    let mut port = String::new();

    println!("Choose on which ip the server to listen to \n(e.g. 127.0.0.1:7878)");
    println!("ps: press enter to go with the default");
    std::io::stdin()
        .read_line(&mut port)
        .expect("U are one son of a bitch");

    // Trim whitespace and newlines from the input
    let port = port.trim();

    // Use a default address if the input is empty
    let port = if port == "" {
        println!("No input provided. Using default address: 127.0.0.1:7878");
        "127.0.0.1:7878"
    } else {
        port
    };
    println!("The address you selected is: {:?}", port);

    let listener = match TcpListener::bind(port) {
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
            get_method(stream, received_data);
            break;
        }
        if received_data[..4] == *b"POST" && bytes_read < buffer.len() {
            post_method(stream, received_data);
            break;
        }
    }
}

fn get_method(mut stream: TcpStream, buffer: Vec<u8>) {
    let buffer = &buffer[..];

    let connected = if let Some(_) = memmem::find(&buffer[..], b"Cookie: Auth").map(|p| p as usize) {
        true
    }   else {
        false
    };

    println!("connected ={:?}", connected);


    if connected == false {
        let status_line =  "HTTP/1.1 200 OK\r\n";
        let response = format!("{}{}",status_line, login_signup());

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }else if buffer[..6] == *b"GET / "{
        let status_line = "HTTP/1.1 200 OK\r\n";

        {
            let mut folder = SHOW_FOLDER.lock().unwrap();
            let user = memmem::find(buffer, b"Cookie: Auth=\"user-").map(|p| p as usize).unwrap();
            let end = memmem::find(buffer, b"-token").map(|p| p as usize).unwrap();
            let user = &buffer[user + "Cookie: Auth=\"user-".len() ..end];
            *folder = String::from_utf8_lossy(&user[..]).to_string(); //wtffffffff
        }
        println!("Done with the normal GET request my guy");

        let response = format!("{}{}", status_line, web(&buffer[..]));

        // println!("{}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
        }


    } else if buffer[..17] == *b"GET /open_folder/"{

        println!("buffer = {}", String::from_utf8_lossy(&buffer[..]));
        let status_line = "HTTP/1.1 200 OK\r\n"; 

        let mut end = memmem::find_iter(&buffer[..], b" ").map(|p| p as usize);
        let _ = end.next();
        let inner = &buffer[b"GET /open_folder/".len()..end.next().unwrap()];
        let inner = String::from_utf8_lossy(&inner[..]);
        {    
            let mut folder = SHOW_FOLDER.lock().unwrap();
            if *folder != "" {
                let url = format!("{}/{}", folder, inner);
                *folder = url.to_string();
            } else {
                *folder = inner.to_string();
            }

            println!("folder that im supposed to show= {}", *folder);
        }

        println!("Done with the GET request my guy");
    
        let response = format!("{}{}", status_line, web(&buffer[..]));
        
        // println!("should get a response?");
        // println!("{}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
        }
    } 
    
    
}

fn post_method(mut stream: TcpStream, buffer: Vec<u8>) {
    if let Some(action) = memmem::find(&buffer[..], b"action=").map(|p| p as usize) {
        post_action(stream, buffer, action);
    } else if let Some(_) = memmem::find(&buffer[..], b"account=").map(|p| p as usize){
        auth_user(stream, buffer);
    } else if let Some(_) = memmem::find(&buffer[..], b"password=").map(|p| p as usize) {
        auth_pass(stream, buffer);
    } else {
        upload_file(stream, buffer);
    }
}

fn auth_user(mut stream: TcpStream, buffer: Vec<u8>) {
    let name = memmem::find(&buffer[..], b"account=").map(|p| p as usize).unwrap();
    let name = &buffer[name + "account=".len()..];
    let name = String::from_utf8_lossy(&name[..]);

    let status_line = "HTTP/1.1 200 OK\r\n";
    let response = format!("{}{}", status_line, password(name.to_string(), None::<usize>));//dont ask homie

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn auth_pass(mut stream: TcpStream, buffer: Vec<u8>) {
    let user = memmem::find(&buffer[..], b"user=").map(|p| p as usize).unwrap();
    let user = &buffer[user + "user=".len()..];
    let end = memmem::find(&user[..], b"&").map(|p| p as usize).unwrap();
    let user = &user[..end];

    let user = String::from_utf8_lossy(&user[..]);

    let pass = memmem::find(&buffer[..], b"password=").map(|p| p as usize).unwrap();
    let pass = &buffer[pass + "password=".len()..];
    let pass = String::from_utf8_lossy(&pass[..]);

    let mut text = Vec::new();
    {
        let mut file = match fs::File::open("users.txt") {
            Ok(c) => c,
            Err(_) => fs::File::create_new("users.txt").unwrap(),
        };

        file.read_to_end(&mut text);
    }

    let search = format!("{}: {} ",user, pass);
    let search = search.as_bytes();


    println!("text in da file ={:?}", text);
    println!("text in string form = {}", String::from_utf8_lossy(&text[..]));
    println!("user = {:?}", user);
    println!("pass = {:?}", pass);
    println!("search = {:?}", String::from_utf8_lossy(&search[..]));
    println!("\n\n\n\n\n");
    

    // check if the person is in the file 
    // else add user and pass
    // but if user is but pass isnt
    // make the user retry pass
    
    match memmem::find(&text[..], user.as_bytes()).map(|p| p as usize){
        Some(user) => {
            // this works well 
            let search_boundary = &text[user..];
            let mut end = memmem::find_iter(&search_boundary, " ").map(|p| p as usize);
            end.next(); 
            let search_boundary = &search_boundary[..end.next().unwrap() + 1];
            println!("search_boundary = {:?}", String::from_utf8_lossy(&search_boundary[..])); 
            println!("search = {:?}", String::from_utf8_lossy(&search[..])); 
            if !(search_boundary == search) {
                let status_line =  "HTTP/1.1 200 OK\r\n";
                let response = format!("{}{}",status_line, password(user.to_string(), Some("try to remember the password u used when creating this account".to_string())));
        
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
                return;
            } 
            //else if it matches do nothing
            ()
        },
        None => {
            let _ = { 
                let metadata = fs::metadata(Path::new("users.txt")).unwrap();
                if metadata.permissions().readonly() {
                    println!("Write permission denied for {:?}", metadata);
                } else {
                    println!("Write permission accepted? for {:?}", metadata);
                }

                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("users.txt")
                    .map_err(|e| {
                        eprintln!("Failed to open users.txt: {}", e);
                        send_error_response(&mut stream, 500, "Server configuration error");
                    }).unwrap();

                writeln!(file, "{}: {} ", user, pass)
                .map_err(|e| {
                    eprintln!("Failed to write to users.txt: {}", e);
                    send_error_response(&mut stream, 500, "Failed to create account");
                }).unwrap();
            };
            ()
        }, //do some shit
        //add user with pass
    }     
    //if the user and pass match show the corresponding 


    {
        let mut folder = SHOW_FOLDER.lock().unwrap();
        *folder = (&user).to_string(); 
        println!("folder ={}", *folder);
        // fs::read_dir(format!("uploads/{}", *folder)).unwrap();
        match fs::read_dir(format!("uploads/{}", *folder)) {
            Err(_) => {
                fs::create_dir_all(format!("uploads/{}", folder));
                fs::create_dir_all(format!("data/{}", folder));
            }
            _ => println!("everything is allright"),
        }
    }

    

    let status_line = "HTTP/1.1 200 OK\r\n";
    let response = format!("{}Set-Cookie: Auth=\"user-{}-token\"; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\n{}", status_line, user, web(&buffer[..]));

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
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

    let (content, content_type, filename) = match parse_file(&mut stream, buffer, &boundary) {
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

    add_file(stream, buffer, content, content_type, filename);
    return;
}

fn get_boundary(buffer: &Vec<u8>) -> Option<Vec<u8>> {
    let buffer = &buffer[..];
    let boundary_b = memmem::find(buffer, b"boundary=")
        .map(|pos| pos as usize)
        .unwrap();
    let boundary_b = &buffer[boundary_b + "boundary=".len()..];
    let boundary_right = memmem::find(boundary_b, b"\r\n")
        .map(|pos| pos as usize)
        .unwrap();
    let boundary = &boundary_b[..boundary_right];
    let boundary = format!("--{}", String::from_utf8_lossy(&boundary[..])).into_bytes();
    // println!("got the boundary as: {}", String::from_utf8_lossy(&buffer[..]));
    Some(boundary)
}

fn parse_file<'a>(
    stream: &mut TcpStream,
    buffer: &'a [u8],
    boundary: &[u8],
) -> Result<(&'a [u8], &'a str, String), &'static str> {
    let content_boundary = match memmem::find_iter(buffer, &boundary)
        .map(|p| p as usize)
        .next()
    {
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
    let content = &info[contents_find + b"\r\n\r\n".len()..info.len() - (boundary.len() + 4)];
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
        let end = memmem::find(&content_type, b"\r\n\r\n")
            .map(|p| p as usize)
            .unwrap();
        let content_type = &content_type[..end];

        //2

        // println!("Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

        //filename part
        let filename = memmem::find_iter(info, b"filename=")
            .map(|p| p as usize)
            .next()
            .unwrap();
        let filename_data = &info[filename + "filename=".len()..];

        let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
        let filename_1 = filename1.next().unwrap();
        let filename_2 = filename1.next().unwrap();
        let filename = &filename_data[filename_1 + 1..filename_2];
        //3
        // println!("filename = {:?}", String::from_utf8_lossy(&filename[..]));
        let file = String::from_utf8_lossy(&filename[..]).to_string();

        return Ok((
            content,
            std::str::from_utf8(content_type).unwrap_or("application/octet-stream"),
            file,
        ));
    }
    let content_type = content_type.next().unwrap();
    let content_type = &buffer[content_type + "Content-Type:\"".len()..];

    // println!("content-type is equal to ={}", String::from_utf8_lossy(&content_type[..]));

    // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let end = memmem::find(&content_type, b"\r\n\r\n")
        .map(|p| p as usize)
        .unwrap();
    let content_type = &content_type[..end];

    //2

    // println!("2Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

    //filename part
    let filename = memmem::find_iter(info, b"filename=")
        .map(|p| p as usize)
        .next()
        .unwrap();
    let filename_data = &info[filename + "filename=".len()..];

    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
    let filename_1 = filename1.next().unwrap();
    let filename_2 = filename1.next().unwrap();
    let filename = &filename_data[filename_1 + 1..filename_2];
    //3
    println!("Parse Upload filename = {:?}", String::from_utf8_lossy(&filename_data[..]));
    println!("Parse Upload filename = {:?}", filename_data);
    println!("Parse Upload filename = {:?}", decode_Windows_1255(&filename[..]));

    // upload filename =uploads/"What’s the craziest way you’ve seen someone get humbled_&#129300;.mp4"
    // Content-Type: video/mp4

    Ok((
        content,
        std::str::from_utf8(content_type).unwrap_or("application/octet-stream"),
        decode_Windows_1255(&filename[..]),
    ))
}

fn post_action(mut stream: TcpStream, buffer: Vec<u8>, action: usize) {
    let data = &buffer[action + "action=".len()..];
    let mut end = memmem::find_iter(data, b"&").map(|p| p as usize);
    let end1 = end.next().unwrap();
    let action = &data[..end1];

    println!("\n0.5action: {:?}", String::from_utf8_lossy(&action[..]));

    let f = memmem::find(data, b"filename=")
        .map(|p| p as usize)
        .unwrap();
    let filename = &data[f + "filename=".len()..];

    println!("\nfile: {:?}", String::from_utf8_lossy(&filename[..]));

    let filename = percent_decode_str(&*String::from_utf8_lossy(&filename[..]))
        .decode_utf8_lossy()
        .replace("+", " ");

    println!("\n1action: {:?}", String::from_utf8_lossy(&action[..]));

    if action[..] == *b"DELETE" {
        println!("Deleted something");
        delet(stream, filename, buffer);
    } else if action[..] == *b"ADD_FOLDER" {
        println!("Added a folder");
        add_folder(stream, &buffer[..], filename);
    } else if action[..] == *b"DOWNLOAD" {
        println!("Downloaded a file");
        download(stream, filename);
    } else if action[..] == *b"RENAME_FOLDER" {

        let end2 = end.next().unwrap();
        let filename = &data[end1 + 1 + "filename=".len()..end2];

        let filename = percent_decode_str(&*String::from_utf8_lossy(&filename[..]))
        .decode_utf8_lossy()
        .replace("+", " ");

        println!("Renaming a folder");
        let new_filename =
            percent_decode_str(&*String::from_utf8_lossy(&data[end2 + "&newFile=".len()..]))
                .decode_utf8_lossy()
                .replace("+", " ");
        println!("new:{}", new_filename);

        rename_folder(stream, buffer, filename, new_filename);
    } else if action[..] == *b"DOWNLOAD_FOLDER" {
        println!("Downloading folder as ZIP");
        download_folder(stream, filename);
    } else {
        send_error_response(&mut stream, 404, "Action not found to perform");
    }

    // println!("did one of the requests");
}

fn download_folder(mut stream: TcpStream, folder_name: String) {

    let folder = SHOW_FOLDER.lock().unwrap();
    let zip_path = format!("{}.zip", folder_name);

    if *folder != "" {
        let folder_path = format!("uploads/{}/{}", folder, folder_name);

        // Create temporary ZIP
        if let Err(e) = zip_folder(Path::new(&folder_path), Path::new(&zip_path)) {
            send_error_response(&mut stream, 500, &format!("ZIP creation failed: {}", e));
            return;
        }
    } else {
        let folder_path = format!("uploads/{}", folder_name);
        let zip_path = format!("{}.zip", folder_name);

        // Create temporary ZIP
        if let Err(e) = zip_folder(Path::new(&folder_path), Path::new(&zip_path)) {
            send_error_response(&mut stream, 500, &format!("ZIP creation failed: {}", e));
            return;
        }
    }
    
    // Send ZIP to client
    let mut file = match fs::File::open(&zip_path) {
        Ok(f) => f,
        Err(e) => {
            send_error_response(&mut stream, 500, &format!("Failed to open ZIP: {}", e));
            return;
        }
    };

    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        send_error_response(&mut stream, 500, &format!("Read error: {}", e));
        return;
    }

    // Clean up temporary ZIP
    if let Err(e) = fs::remove_file(&zip_path) {
        eprintln!("Failed to clean up ZIP file: {}", e);
    }

    // Prepare response
    let status_line = "HTTP/1.1 200 OK\r\n";
    let headers = format!(
        "Content-Type: application/zip\r\n\
         Content-Disposition: attachment; filename=\"{}.zip\"\r\n\
         Content-Length: {}\r\n\r\n",
        folder_name,
        buffer.len()
    );

    let response = format!("{}{}", status_line, headers);
    
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to send headers: {}", e);
    }
    
    if let Err(e) = stream.write_all(&buffer) {
        eprintln!("Failed to send ZIP content: {}", e);
    }
}

fn zip_folder(folder_path: &Path, zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let options= SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let base_path = folder_path.parent().unwrap_or_else(|| Path::new(""));

    for entry in WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(base_path)?.to_str().unwrap();

        if path.is_file() {
            zip.start_file(name, options)?;
            let mut f = fs::File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        } else if !name.is_empty() {
            zip.add_directory(name, options)?;
        }
    }

    zip.finish()?;
    Ok(())
}

fn rename_folder(mut stream: TcpStream, buffer: Vec<u8>, old_folder: String, new_folder: String) {
    {
        let folder = SHOW_FOLDER.lock().unwrap();
        if *folder != "" {
            println!("before uploads/{}/{}", folder, old_folder);
            println!("after uploads/{}/{}", folder, new_folder);
            fs::rename(format!("uploads/{}/{}", folder, old_folder), format!("uploads/{}/{}", folder, new_folder));
            fs::rename(format!("data/{}/{}", folder, old_folder), format!("data/{}/{}", folder, new_folder));
        } else {
            println!("uploads/{}", old_folder);
            println!("uploads/{}", new_folder);
            fs::rename(format!("uploads/{}", old_folder), format!("uploads/{}", new_folder));
            fs::rename(format!("data/{}", old_folder), format!("data/{}", new_folder));
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";

    println!("\n\nDone with the POST RENAME_FOLDER action request my guy");

    let response = format!("{}{}", status_line, web(&buffer[..]));
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn delet(mut stream: TcpStream, filename: String, buffer: Vec<u8>) {
        
        {
        let folder1 = SHOW_FOLDER.lock().unwrap();
        let folder1 = percent_decode_str(&*folder1)
                        .decode_utf8_lossy()
                        .replace("+", " ")
                        .to_owned();
        if &*folder1 != "" {

            println!("\n\nbuffer={}", String::from_utf8_lossy(&buffer[..]));
            if let Some(folder) = memmem::find(&buffer[..], b"folder=") {
                let file = memmem::find(&buffer[..], b"filename=")
                    .map(|p| p as usize)
                    .unwrap();
                let file = &buffer[file + "filename=".len()..];
                let filename = String::from_utf8_lossy(&file[..]);

                let filename = percent_decode_str(&filename)
                                    .decode_utf8()
                                    .unwrap();

                let filename = decode_html(&filename)
                                    .unwrap()
                                    .replace("+", " ");

                println!("filename suppoised to get deleted= uploads/{}/{}", folder1, filename);

                fs::remove_dir_all(&*format!("uploads/{}/{}", folder1, filename));
                fs::remove_dir_all(&*format!("data/{}/{}", folder1, filename));
            } else {
                
                println!("deleting file={}/{}",folder1, filename);
                fs::remove_file(&*format!("uploads/{}/{}", folder1, filename)); //dont u dare change this shi
                fs::remove_file(&*format!("data/{}/{}", folder1, filename));
            }
        } else {
            send_error_response(&mut stream, 403, "Somehow you are not logged in");
        }
        
    }

    let status_line = "HTTP/1.1 200 OK\r\n";

    println!("\n\nDone with the POST delete action request my guy");
    let response = format!("{}{}", status_line, web(&buffer[..]));
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn download(mut stream: TcpStream, filename: String) {
    let entries = fs::read_dir("uploads").unwrap();
    let mut file_names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    let mut file = fs::File::open(format!("uploads/{}", filename)).unwrap();
    let mut data = fs::File::open(format!("data/{}.txt", filename)).unwrap();

    println!("{}", format!("download uploads/{}", filename));

    let mut read = Vec::new();
    file.read_to_end(&mut read).unwrap();

    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("filename={}", decode_html(&filename).unwrap());
    if filename.contains("/"){
        let start = memmem::find(filename.as_bytes(), b"/")
                                .map(|p| p as usize)
                                .unwrap();

        let filename = String::from_utf8_lossy(&filename.as_bytes()[start + 1..]);

        let response = format!(
            "{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
            status_line,
            content_type,
            decode_html(&filename).unwrap(),
            read.len()
        );

        println!("Done with the POST download action my guy");
        stream.write(response.as_bytes()).unwrap();
        stream.write(&read[..]).unwrap();
        stream.flush().unwrap();
        return;
    }


    let response = format!(
        "{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_type,
        decode_html(&filename).unwrap(),
        read.len()
    );

    println!("Done with the POST download action my guy");
    stream.write(response.as_bytes()).unwrap();
    stream.write(&read[..]).unwrap();
    stream.flush().unwrap();
    // println!("filename={}", filename);
    
}

fn add_folder(mut stream: TcpStream, buffer: &[u8], filename: String) {
    if filename.contains("../") {
        println!("Caught u red handed");
        println!("filename={}", filename);

        send_error_response(&mut stream, 404, "Dont try to go out of bounds, mister");
        return;
    }

    println!("path ={}", filename);

    {
        let folder = SHOW_FOLDER.lock().unwrap();
        if *folder != "" {
            if Path::new(&format!("uploads/{}/{}", folder, filename)).exists() {
                send_error_response(&mut stream, 403, "Folder already exists");
                return;
            }
            fs::create_dir_all(format!("uploads/{}/{}", folder, filename)).unwrap(); // handle gracefully
            fs::create_dir_all(format!("data/{}/{}", folder, filename)).unwrap();
        
            println!("uploads/{:?}\n\n", filename);
            
        } else {
            if Path::new(&format!("uploads/{}", filename)).exists() {
                send_error_response(&mut stream, 403, "Folder already exists");
                return;
            }
            fs::create_dir_all(format!("uploads/{}", filename)).unwrap(); // handle gracefully
            fs::create_dir_all(format!("data/{}", filename)).unwrap();
        
            println!("uploads/{:?}\n\n", filename);
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";
        
    let response = format!("{}{}", status_line, web(buffer));
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    
}

fn add_file_in_folder(
    mut stream: TcpStream,
    buffer: &[u8],
    content: &[u8],
    content_type: &str,
    filename: String,
) {
    let folder = match memmem::find(&buffer[..], b"name=\"folder\"").map(|p| p as usize) {
        Some(f) => f,
        None => {
            send_error_response(&mut stream, 404, "Folder not found");
            return;
        }
    };

    println!("should add a file in da folder");
    let folder = &buffer[folder + "name=\"folder\"".len() + "\r\n\r\n".len()..];

    // println!("folder? = {}", String::from_utf8_lossy(&folder[..]));
    let end = memmem::find(folder, b"\r\n").map(|p| p as usize).unwrap();
    let folder = &folder[..end];

    // println!("filename before change = {}", filename);

    let filename = format!("{}/{}", String::from_utf8_lossy(&folder[..]), filename);

    // println!("filename after change = {}", filename);

    add_file(stream, buffer, content, content_type, filename);
}

fn add_file(
    mut stream: TcpStream,
    buffer: &[u8],
    content: &[u8],
    content_type: &str,
    filename: String,
) {
    // do some shady shit
    {
        let folder = SHOW_FOLDER.lock().unwrap();
        println!("folder im supposed to save the file={:?}", folder);
        if *folder != "" {
            let filename_upload = format!("uploads/{}/{}", 
                percent_decode_str(&folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned(),
                percent_decode_str(&filename)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned());
            println!("upload filename ={}\n\n", filename_upload);

            

            let mut file = fs::File::create(&filename_upload).unwrap();
            file.write_all(content);

            let filename_data = format!("data/{}/{}.txt", 
                percent_decode_str(&folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned(),
                filename);
            // println!("filename_data = {}", filename_data);
            println!("filename_data = {}", filename_data);
            let mut file2 = fs::File::create(&filename_data).unwrap();

            file2.write_all(&format!("Content-Type:{}", content_type).into_bytes()[..]); //idk how this works
                                                                                        //till here we saved the file on the server (hopefully)

        } else {
            let filename_upload = format!("uploads/{}", filename);
            println!("upload filename ={}", filename_upload);

            let mut file = fs::File::create(&filename_upload).unwrap();
            file.write_all(content);

            let filename_data = format!("data/{}.txt", filename);
            // println!("filename_data = {}", filename_data);
            println!("filename_data = {}", filename_data);
            let mut file2 = fs::File::create(&filename_data).unwrap();

            file2.write_all(&format!("Content-Type:{}", content_type).into_bytes()[..]); //idk how this works
                                                                                        //till here we saved the file on the server (hopefully)
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";

    println!("\n\nDone with the POST add_file request my guy");
    let response = format!("{}{}", status_line, web(buffer));

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn web(buffer: &[u8]) -> String {
    let folder = SHOW_FOLDER.lock().unwrap();
    // transform this uploads/figet/smashbros/dump%20me 
    // in this uploads/figet/smashbros/dump me 
    
    println!("WEB\n\n\nDefinetly able to enter this folder: uploads/{}", 
            percent_decode_str(&*folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned()
            );
    let folder2 = folder.clone();
    let entries = fs::read_dir(format!("uploads/{}", 
        percent_decode_str(&folder2)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned()
    )).unwrap();
    let mut file_names = Vec::new();

    let mut files = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        files.push(entry.path());
        let file_name = entry.file_name().into_string().unwrap();
        file_names.push(file_name);
    }

    let mut html = String::from(
        "    
    <!DOCTYPE html>
    <html lang=\"en\">
    <head>
    <meta charset=\"UTF-8\">
    <title>File Upload</title>
    </head>

    <style>
        li{
            display: flex;
            margin: auto;
            width: 300px;
            height: 50px;
            padding: 10px;
            justify-content: center;
            align-items: center;
            font-size: 30px;
        }

        li > form {
            margin: 0;
        }

        li > form > button{
            margin: 0 10px;
            font-size: 25px;
        }

        li > button {
            font-size: 25px;
        }

        li > div:nth-child(2) {
            margin:0 0 0 50px;
        }
    </style>

    <body>
    <h1>Hello!</h1>
    <p>Welcome to your file server :)</p>

    <form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">
        <input type=\"file\" name=\"file\"  required>
        <button type=\"submit\">Upload</button>
    </form>

    <form action\"/\" method=\"POST\">
        <input type=\"hidden\" name=\"action\" value=\"ADD_FOLDER\">
        <input type=\"text\" name=\"filename\" required>
        <button type=\"submit\">Add Folder </button>
    </form> "
    );
    
    if let Some(user) =  memmem::find(buffer, b"Cookie: Auth=\"user-").map(|p| p as usize) {
        
        let folder = &*folder.as_bytes();
        let user = &buffer[user + "Cookie: Auth=\"user-".len() ..];
        let end = memmem::find(user, b"-token").map(|p| p as usize).unwrap();
        let user = &user[..end];

        let folder = &folder[user.len()..];
        let folder = String::from_utf8_lossy(&folder[..]);

        if &folder != ""  {         //911 joke incoming
            html.push_str(&*format!(
                "
                Location: {}
                <br>
                <button onclick=\"window.location.href='/'\">Go back to home</button>
                ",
                percent_decode_str(&folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned()
            ));
        }
    }
    

    html.push_str("
        <h2> Saved Files:</h2>
        <ul>
    ");

    for i in 0..file_names.len() {
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
                    <form action=\"/\" method =\"POST\">
                        <input type=\"hidden\" name=\"action\" value=\"RENAME_FOLDER\">
                        <input type=\"hidden\" name=\"filename\" value=\"{}\">
                        <input type=\"text\" name=\"newFile\">
                        <button type=\"submit\">Rename</button>
                    </form>
                    <form action=\"/\" method=\"POST\">
                        <input type=\"hidden\" name=\"action\" value=\"DOWNLOAD_FOLDER\">
                        <input type=\"hidden\" name=\"filename\" value=\"{}\">
                        <button type=\"submit\">Download as ZIP</button>
                    </form>
                    <button onclick=\"window.location.href='/open_folder/{}'\">Open folder</button>
                </li>",
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i]
            ));
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
                file_names[i], file_names[i], file_names[i]
            ));
        }
    }

    html.push_str(
        "
        </ul>
        </body>
        </html>",
    );

    return html;
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
    let mut html = String::from(
        "
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
    ",
    );
    html.push_str(&*format!("<h1> {} </h1>", message));
    html.push_str(
        "
        <button onclick=\"window.location.href='/'\"> Go back to the main page </button>
    ",
    );

    html.push_str(" </body> </html>");

    html
}

fn login_signup() -> String {
    let html = String::from("
        <!DOCTYPE html>
        <html>
        <head>
        <title> Login / Signup bro </title>
        </head>
        <body>
            <h1> Welcome to your File Manager Server </h1>

            <h3> It seems you are not currently connected to an account <br> Please Signup or Login to use this platform<h3>
        
        <form action=\"/\" method=\"POST\">
            <input type=\"text\" name=\"account\">
            <button type=\"submit\"> Continue </button>
        </form>
        </body>
        </html>
    ");
    html
}

fn password<T>(name: String, extra_info: Option<T>) -> String { 
    let html = String::from(format!("
        <!DOCTYPE html>
        <html>
        <head>
        <title> Login / Signup bro </title>
        </head>
        <body>
            <h1> Welcome to your File Manager Server </h1>

            <h3> Enter the password for your account<h3>
        
        <form action=\"/\" method=\"POST\">
            <input type=\"hidden\" name=\"user\" value=\"{}\">
            <input type=\"text\" name=\"password\">
            <button type=\"submit\"> Login/Signup </button>
        </form>

        </body>
        </html>
    ",
    name
    ));
    html
}

// 217193383