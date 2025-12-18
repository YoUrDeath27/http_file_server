use htmlescape::decode_html;
use memchr::memmem;
use percent_encoding::percent_decode_str;
use std::{
    fs,
    sync::Mutex,
    path::{Path, PathBuf},
    io::{Read, Write, Error},
    net::{TcpListener, TcpStream},
    time::{Instant, Duration},  //time
    collections::HashMap,
};

use chrono::prelude::*;

use lazy_static::lazy_static;
use zip::write::SimpleFileOptions;
use walkdir::WalkDir;

use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};

use uuid::{Uuid};

mod auth;
mod file_operations;
mod utils;
mod pages;
mod preview;
mod security;

use auth::*;
use file_operations::*;
use utils::*;
use pages::*;
use preview::*;
use security::*;

// use std::any::type_name;


    // use std::thread;
    // use std::time::Duration;
    // use std::path::Path;
/*
    ----------------------------------------------------------------------------------------------------------------------------------------------------------------
    - NEXT STEPS TO IMPLEMENT (aka where you left off)

    so security.rs nici nu mai tre sa zic ce trebuie sa ii faci, poate implement si cv de genu 
        check_Auth, sau diverse

    si log everything in the file, that can be logged ofc
    so i can better see the workflow ig? but then there wil be a problem if i get 2 requests at the same time and both for different things but they get processed at the same time

    -make in web so that they get sorted based on filename in data nu cel salvat ca sa arate bine (cred ca s-ar incadra in utils sa fac functia si in web doar sa trimit lista)

    -i think i need to make the sorting function work with javascript.... wish me luck (or maybe not wait a sec)
    
    good luck

    ----------------------------------------------------------------------------------------------------------------------------------------------------------------  
*/

/*
*****************************************************************************************************
    MENTIONS
    -cand log da fail creeaza un alt log (good idea?)


*****************************************************************************************************
*/

fn main() {
    let mut port = String::new();

    // let datetime: DateTime<Local> = Local::now();
    // println!("time: {}", datetime);
    
    // bubble_sort();
    // println!("\n\n\nerror: {:?}", insert_sort());
    println!("Time? {:?}", Instant::now());
    println!("Time? {:?}", Duration::from_secs(2000));
    println!("Time? {:?}", Instant::now() + Duration::from_secs(2000));
    println!("Time? {:?}", Local::now());
    
    /*
        Time? Instant { t: 453.7383204s }
        Time? Instant { t: 474.4293796s }
        Time? Instant { t: 492.5384597s }

     */

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

    match log(&*String::from("Started the file server, ready to get requests"), 0){
        Ok(x) => x,
        Err(e) => {
            println!("error logging: {}", e);
        } 
    }
    
    /*
    match log(&*String::from("THis is a warning log"), 1){
        Ok(x) => x,
        Err(e) => {
            println!("error logging: {}", e);
            return;  
        } 
    }
    match log(&*String::from("This is a server side error"), 2){
        Ok(x) => x,
        Err(e) => {
            println!("error logging: {}", e);
            return;  
        } 
    }
    match log(&*String::from("THis is a client side error"), 3){
        Ok(x) => x,
        Err(e) => {
            println!("error logging: {}", e);
            return;  
        } 
    }
    match log(&*String::from("This is a fatal error"), 4){
        Ok(x) => x,
        Err(e) => {
            println!("error logging: {}", e);
            return;  
        } 
    } */

    /*
    time: "21:58:01 UTC"
    whatever: File { handle: 0x140, path: "\\\\?\\C:\\$Recycle.Bin\\S-1-5-21-820130014-3556285722-1672997054-1001\\$RG5XGF3.txt" }
    not supposed to be like this
     */
   
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            },
            Err(e) => println!("Failed to connect {}", e  )
        }
    }
}
#[derive(Clone, Debug)]
pub struct Request<'a>{
    header: &'a Vec<u8>,
    body: Option<Vec<u8>>
}

struct Response { // i should implement this later
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

        let content_count = match memmem::find(&received_data[..], b"Content-Length:").map(|p| p as usize) {
            Some(x) => x,
            None => 0,
        };

        let rn = memmem::find(&received_data[content_count + "Content-Length: ".len()..], b"\r\n").map(|p| p as usize).unwrap();
        let cc =&received_data[content_count + "Content-Length: ".len()..];
        // println!("content count : {:?}", String::from_utf8_lossy(&cc[..]));
        let content_count: usize =  match std::str::from_utf8(&cc[..rn])
                                                .expect("Not a valid UTF-8")
                                                .trim()
                                                .parse() {
                                                    Ok(x) => x,
                                                    Err(e) => {
                                                        println!("There is no content here: {}\n\n", e); 
                                                        9999999999 //or some other arbitrary big fucking number
                                                    }
                                                };
        if content_count == 9999999999 && memmem::find(request.body.as_ref().unwrap(), b"GET ").is_some() {
            send_error_response(&mut stream, 400, &format!("There was a problem uploading your file, please try again later"));   
            return;
        }                              


        request.body.as_mut().unwrap().extend_from_slice(&received_data[end + "\r\n\r\n".len()..]);

        let finished = if is_get {
            true
        } else if is_post {
            memmem::find(&received_data, b"Content-Length").is_some() && 
            request.body.as_ref().unwrap().len() == content_count //i got no clue why i did thi 18.30.25
        } else {
            false
            //check if there is a form for loging in //how tf am i supposed to do this past me?
            //and at the last possible moment return false
        };

        if finished{ //
        // println!("header: {}\n\n\n", String::from_utf8_lossy(&request.header[..]));
        // println!("body: {}\n\n\n", String::from_utf8_lossy(&request.body.as_ref().unwrap().clone()[..]));
        // println!("Finished the request: {:?}", String::from_utf8_lossy(&received_data[..]));


            match log(&*String::from_utf8_lossy(&received_data[..]), 0){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
    
           
            match decode_and_check_path(request.clone()) {
                Ok(x) => {
                    println!("\n\n\n\ndecoded finding: {}", x);
                    ()
                },
                Err(_) => {
                    send_error_response(&mut stream, 404, "The user attempted to do a path traversal trick");
                    return;
                }
            }
            
            if !memmem::find(&request.header, b"../").is_some(){ //first check for path traversal
                if is_get && bytes_read < buffer.len() {
                    get_method(stream, request);
                }
                else if is_post && bytes_read < buffer.len() {
                    post_method(stream, request);
                }
            } else {
                match log(&*String::from("The user tried to do some path traversal"), 3){
                    Ok(x) => x,
                    Err(e) => {
                        println!("error logging: {}", e);
                    } 
                }
                send_error_response(&mut stream, 400, "Did you actually thought you can do this? 1");
            }
            break;
        }
    }
}

fn get_method(mut stream: TcpStream, request: Request) {
    // let buffer = &request.body.as_ref().unwrap()[..];

    let connected = if let Some(_) = memmem::find(&request.header[..], b"Cookie: Auth").map(|p| p as usize) {
        match log(&*String::from("The user is authenticated"), 1){
            Ok(x) => x,
            Err(e) => {
                println!("error logging: {}", e);
            } 
        };
        true
    }   else {
        match log(&*String::from("The user is not authenticated"), 1){
            Ok(x) => x,
            Err(e) => {
                println!("error logging: {}", e);
            } 
        };
        false
    };

    // println!("connected ={:?}", connected);


    if connected == false {
        let status_line =  "HTTP/1.1 200 OK\r\n";
        let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}",status_line, login_signup());

        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
            match log(&format!("Write error: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
            match log(&format!("Error flushing: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }

    } else if request.header[..6] == *b"GET / " && connected == true{
        let status_line = "HTTP/1.1 200 OK\r\n";
        {
            let mut folder = match SHOW_FOLDER.lock() {
                Ok(f) => f,
                Err(e) => {
                    match log(&format!("There was an error opening the folder mutex {}", e), 3) {
                        Ok(x) => x,
                        Err(e) =>{
                            send_error_response(&mut stream, 400, &format!("{}", e));
                            return;
                        }
                    }
                    send_error_response(&mut stream, 400, "Failed to open the folder Variable so i cant see who is conected");
                    return;
                },
            };
            let user = match memmem::find(request.header, b"Cookie: Auth=\"user-").map(|p| p as usize){
                Some(x) => x,
                None => {
                    send_error_response(&mut stream, 400, "I think you broke something in here");
                    return;
                },
            };
            let end = memmem::find(request.header, b"-token").map(|p| p as usize).unwrap();
            let user = &request.header[user + "Cookie: Auth=\"user-".len() ..end];
            *folder = String::from_utf8_lossy(&user[..]).to_string(); //wtffffffff
        }
        // println!("Done with the normal GET request my guy");

        let site = web(request);
        if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
            send_error_response(&mut stream, 400, "There has been an error generating the webpage");
            return;
        }
        let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);

        // println!("{}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
            match log(&format!("Write error: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
            match log(&format!("Error flushing: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }


    } else if memmem::find(&request.header, b"/uploads/").is_some(){ //for previews
        //for images
        web_send_image(stream, request);
    } else if request.header[..19] == *b"GET /open_folder../" { //for back traversal  
        // println!("We got this chat");
        let status_line = "HTTP/1.1 200 OK\r\n";

        {
            let mut folder = match SHOW_FOLDER.lock() {
                Ok(f) => f,
                Err(e) => {
                    println!("failed to open the folder variable {}", e);
                    send_error_response(&mut stream, 400, "Failed to open the folder Variable so i cant see who is conected");
                    return;
                },
            };

            let path = Path::new(&*folder);
            let parent = match path.parent() {
                Some(p) => p,
                None => {
                    send_error_response(&mut stream, 400, "No parent folder found");
                    return;
                }
            };

            // println!("parent = {:?}", parent.display().to_string());
            *folder = parent.display().to_string();
        }

        let site = web(request);
        if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
            send_error_response(&mut stream, 400, "There has been an error generating the webpage");
            return;
        }
        let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);

        // println!("should get a response?");
        // println!("{}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
            match log(&format!("Write error: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
            match log(&format!("Error flushing: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }

        
    } else if request.header[..17] == *b"GET /open_folder/"{

        // println!("buffer = {}", String::from_utf8_lossy(&buffer[..]));
        let status_line = "HTTP/1.1 200 OK\r\n"; 

        let mut end = memmem::find_iter(&request.header[..], b" ").map(|p| p as usize);
        let _ = end.next();
        let inner = &request.header[b"GET /open_folder/".len()..end.next().unwrap()];
        let inner = String::from_utf8_lossy(&inner[..]);
        {    
            let mut folder = match SHOW_FOLDER.lock() {
                Ok(f) => f,
                Err(e) => {
                    println!("There was a problem opening the folder variable {}", e);
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

            // println!("folder that im supposed to show= {}", *folder);
        }

        // println!("Done with the GET request my guy");
    
        let site = web(request);
        if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
            send_error_response(&mut stream, 400, "There has been an error generating the webpage");
            return;
        }
        let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);
        
        // println!("should get a response?");
        // println!("{}", response);
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
            match log(&format!("Write error: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
            match log(&format!("Error flushing: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
        }
    }   
}

fn post_method(stream: TcpStream, buffer: Request) {
    if let Some(action) = memmem::find(&buffer.body.clone().unwrap()[..], b"action=").map(|p| p as usize) {
        post_action(stream, buffer, action);
    } else if let Some(_) = memmem::find(&buffer.body.clone().unwrap()[..], b"account=").map(|p| p as usize){
        auth_user(stream, buffer);
    } else if let Some(_) = memmem::find(&buffer.body.clone().unwrap()[..], b"password=").map(|p| p as usize) {
        auth_pass(stream, buffer);
    } else {
        upload_file(stream, buffer);
    }
}

fn post_action(mut stream: TcpStream, buffer: Request, action: usize) {
        let data = &buffer.body.clone().unwrap()[action + "action=".len()..];
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

        // println!("\n0.5action: {:?}", String::from_utf8_lossy(&action[..]));

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

        // println!("\nfile: {:?}", String::from_utf8_lossy(&filename[..]));

        let filename = percent_decode_str(&*String::from_utf8_lossy(&filename[..]))
            .decode_utf8_lossy()
            .replace("+", " ");

        if path_traversal_check(&filename) {

            match log("The client tried to do a path traversal", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 400, "Did you actually thought you can do this? 2");
            return;    
        }

        if action[..] == *b"DELETE" {
            // println!("Deleted something");
            match log("The user is deleting something", 0){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            delet(stream, filename, buffer);
        } else if action[..] == *b"ADD_FOLDER" {    
            // println!("Added a folder");
            match log("The user is making another folder", 0){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            add_folder(stream, buffer, filename);
        } else if action[..] == *b"DOWNLOAD" {
            // println!("Downloaded a file");
            match log("The user is downloading a file stored on the server", 0){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            download(stream, filename);
        } else if action[..] == *b"RENAME_FOLDER" {

            //idk why this part is here but i'm gonna move it
            //or maybe not
            
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

            // println!("filename after renaming: {:?}", filename);


            // println!("Renaming a folder");
            let new_filename =
                percent_decode_str(&*String::from_utf8_lossy(&data[end2 + "&newFile=".len()..]))
                    .decode_utf8_lossy()
                    .replace("+", " ");
            // println!("new:{}", new_filename);

            if path_traversal_check(&new_filename) {
                send_error_response(&mut stream, 400, "Did you actually thought you can do this? 3");
                return;    
            }

            match log("The user is renaming a folder", 0){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            rename_folder(stream, buffer, filename, new_filename);
        } else if action[..] == *b"DOWNLOAD_FOLDER" {
            // println!("Downloading folder as ZIP");
            match log("The user is zip downloading a folder", 0){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            download_folder(stream, filename);
        } else {
            match log("The user attempted to", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "Action not found to perform");
        }

        // println!("did one of the requests");
    }

// 217193383