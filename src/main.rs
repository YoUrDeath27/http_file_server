use delete::{delete_file, delete_folder};
use htmlescape::decode_html;
use memchr::memmem;
use percent_encoding::{percent_decode_str, percent_encode, utf8_percent_encode, AsciiSet, CONTROLS};
use std::{
    fs,
    io::Error,
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

mod auth;
mod file_operations;
mod utils;
mod pages;
mod preview;

use auth::*;
use file_operations::*;
use utils::*;
use pages::*;
use preview::*;

    // use std::thread;
    // use std::time::Duration;
    // use std::path::Path;

/*
------------------------------------------------------------------------
    keep testing your server blud

    for now it works okish
    but still, keep testing so u can develop yourself

    dupa ce implementezi sa ai un image preview, MODULEAZA-TI proiectul acum cat inca e destul de mic
------------------------------------------------------------------------
*/


/*
    --------------------------------------------------------------------------------
    make a way to count how many failed attempts the user has when guessing the password
    and if says it wrong for too many times
    to have a countown of 1 minute till the user has 3 more attempts
    line 280

    make a function somewhere to go back one folder so you dont have to allways go back to root
    its gonna be fun


    14.03.2025  10:57
    --------------------------------------------------------------------------------
*/

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
    match fs::create_dir_all("uploads"){
        Ok(x) => x,
        Err(e) => println!("Failed to create the uploads directory\n{}", e) ,
    }; // Create uploads directory
    match fs::create_dir_all("data"){
        Ok(x) => x,
        Err(e) => println!("Failed to create the data directory\n{}", e) ,
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => println!("Failed to connect")
        }
    }
}

struct Request<'a>{
    header: &'a Vec<u8>,
    body: Option<Vec<u8>>
}

struct response {
    code: String,
    web_response: String 
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
        let mut rnrn = memmem::find_iter(&received_data[..], b"\r\n\r\n").map(|p| p as usize);
        let end = rnrn.next().unwrap(); //keep it like this for now  
        
        let mut request= Request{
                        header: &received_data[..end].to_vec(), 
                        body: Some(Vec::new())
        };

        let is_get = memmem::find(&request.header, b"GET").is_some();
        let is_post = memmem::find(&request.header, b"POST").is_some();

        println!("header: {}", String::from_utf8_lossy(&request.header[..]));

        let Content_count = match memmem::find(&received_data[..], b"Content-Length:").map(|p| p as usize) {
            Some(x) => x,
            None => 0,
        };

        let rn = memmem::find(&received_data[Content_count + "Content-Length: ".len()..], b"\r\n").map(|p| p as usize).unwrap();
        let Cc =&received_data[Content_count + "Content-Length: ".len()..] ;
        let content_count: usize =  match std::str::from_utf8(&Cc[..rn])
                                                .expect("Not a valid UTF-8")
                                                .trim()
                                                .parse() {
                                                    Ok(x) => x,
                                                    Err(e) => { println!("There is no content here\n\n"); 0}
                                                };
                                                


        request.body.as_mut().unwrap().extend_from_slice(&received_data[end + "\r\n\r\n".len()..]);

        // println!("head:{}\n\n", String::from_utf8_lossy(&request.header[..]));
        // println!("body: {}\n\n", String::from_utf8_lossy(&request.body.as_ref().unwrap().clone()[..]));
        // println!("length expected: {}", content_count);
        // println!("length got: {}", request.body.as_ref().unwrap().len());

        let finished = if is_get {
            true
        } else if is_post {
            memmem::find(&received_data, b"Content-Length").is_some() && 
            request.body.as_ref().unwrap().len() == content_count
        } else {
            false
            //check if there is a form for loging in
            //and at the last possible moment return false
        };

        if finished{ //
            if is_get && bytes_read < buffer.len() {
                get_method(stream, received_data);
            }
            else if is_post && bytes_read < buffer.len() {
                post_method(stream, received_data);
            }
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

        stream.write(response.as_bytes());
        stream.flush();
    }else if buffer[..6] == *b"GET / " && connected == true{
        let status_line = "HTTP/1.1 200 OK\r\n";

        {
            let mut folder = match SHOW_FOLDER.lock() {
                Ok(f) => f,
                Err(e) => {
                    send_error_response(&mut stream, 400, "Failed to open the folder Variable so i cant see who is conected");
                    return;
                },
            };
            let user = match memmem::find(buffer, b"Cookie: Auth=\"user-").map(|p| p as usize){
                Some(x) => x,
                None => {
                    send_error_response(&mut stream, 400, "I think you broke something in here");
                    return;
                },
            };
            let end = memmem::find(buffer, b"-token").map(|p| p as usize).unwrap();
            let user = &buffer[user + "Cookie: Auth=\"user-".len() ..end];
            *folder = String::from_utf8_lossy(&user[..]).to_string(); //wtffffffff
        }
        println!("Done with the normal GET request my guy");

        let site = web(&buffer[..]);
        if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
            send_error_response(&mut stream, 400, "There has been an error generating the webpage");
            return;
        }
        let response = format!("{}{}", status_line, site);

        // println!("{}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
        }


    } else if memmem::find(&buffer, b"/uploads/").map(|p| p as usize).is_some(){
        //for images
        web_send_image(stream, buffer.to_vec());
    } else if buffer[..17] == *b"GET /open_folder/"{

        // println!("buffer = {}", String::from_utf8_lossy(&buffer[..]));
        let status_line = "HTTP/1.1 200 OK\r\n"; 

        let mut end = memmem::find_iter(&buffer[..], b" ").map(|p| p as usize);
        let _ = end.next();
        let inner = &buffer[b"GET /open_folder/".len()..end.next().unwrap()];
        let inner = String::from_utf8_lossy(&inner[..]);
        {    
            let mut folder = match SHOW_FOLDER.lock() {
                Ok(f) => f,
                Err(e) => {
                    send_error_response(&mut stream, 400, "Failed to open the folder Variable so i cant see who is conected");
                    return;
                },
            };
            if *folder != "" {
                let url = format!("{}/{}", folder, inner);
                *folder = url.to_string();
            } else {
                *folder = inner.to_string();
            }

            println!("folder that im supposed to show= {}", *folder);
        }

        println!("Done with the GET request my guy");
    
        let site = web(&buffer[..]);
        if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
            send_error_response(&mut stream, 400, "There has been an error generating the webpage");
            return;
        }
        let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);
        
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

fn post_action(mut stream: TcpStream, buffer: Vec<u8>, action: usize) {
        let data = &buffer[action + "action=".len()..];
        let mut end = memmem::find_iter(data, b"&").map(|p| p as usize);
        let end1 = match end.next(){
            Some(x) => x,
            None => {
                println!("These errors get more rare");
                send_error_response(&mut stream, 400, "Just how");
                return;
            }
        };
        let action = &data[..end1];

        println!("\n0.5action: {:?}", String::from_utf8_lossy(&action[..]));

        let f = match memmem::find(data, b"filename=")
            .map(|p| p as usize){
                Some(x) => x,
                None => {
                    println!("Nope, ur cooked chat");
                    send_error_response(&mut stream, 400, "The file/request probably got corrupted during transmission");
                    return;
                }
            };
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
            download(stream, filename, buffer);
        } else if action[..] == *b"RENAME_FOLDER" {

            let end2 = match end.next(){
                Some(x) => x,
                None => {
                    println!("Nope, ur cooked chat");
                    send_error_response(&mut stream, 400, "The file/request probably got corrupted during transmission");
                    return;
                }
            };
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

// 217193383