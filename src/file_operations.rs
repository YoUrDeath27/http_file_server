use super::*;

pub fn upload_file(mut stream: TcpStream, buffer: Request) {

    let boundary = match get_boundary(&buffer.header[..].to_vec()) {
        Some(b) => b,
        None => {
            send_error_response(&mut stream, 400, "Invalid request format");
            return;
        }
    };

    // println!("boundary={}", String::from_utf8_lossy(&boundary[..]));


    let mut buffer1 = buffer.clone();
    let (content, content_type, filename) = match parse_file(&mut stream, &mut buffer1, &boundary) {
        Ok(data) => data,
        Err(e) => {
            send_error_response(&mut stream, 400, &format!("Failed to parse request, {}", e));
            return;
        }
    };

    println!("filename uploaded: {}", filename);
    if !ALLOWED_MIME_TYPES.contains(&&*content_type) {
        println!("sontent_type ={}", content_type);
        match log("The user tried to upload a file that is not supported", 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 400, "Unsuported file type");
        return;
    }

    if let Some(_) = memmem::find(&buffer.body.clone().unwrap()[..], b"name=\"folder\"").map(|p| p as usize) {
        add_file_in_folder(stream, buffer, &content, &content_type, filename);
        return;
    }

    add_file(stream, buffer, &content, &content_type, filename);
    return;
}

pub fn download_folder(mut stream: TcpStream, folder_name: String) {
    let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                match log(&format!("Error identifying the: {}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);   
                    } 
                }
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        }; 
    let zip_path = format!("{}.zip", folder_name);
    let folder_path;
    
    if *folder != "" {
        folder_path = format!("uploads/{}/{}", folder, folder_name);
    } else {
        folder_path = format!("uploads/{}", folder_name);
    }
    
    // Create temporary ZIP
    if let Err(e) = zip_folder(Path::new(&folder_path), Path::new(&zip_path)) {
        match log(&format!("Error generating the ZIP file: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 500, &format!("ZIP creation failed: {}", e));
        return;
    }
    // Send ZIP to client
    let mut file = match fs::File::open(&zip_path) {
        Ok(f) => f,
        Err(e) => {
            match log(&format!("Error : {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 500, &format!("Failed to open ZIP: {}", e));
            return;
        }
    };

    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        match log(&format!("Error reading the file's contents: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 500, &format!("Read error: {}", e));
        return;
    }

    // Clean up temporary ZIP
    if let Err(e) = fs::remove_file(&zip_path) {
        match log(&format!("Error cleaning the ZIP file: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
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

fn zip_folder(folder_path: &Path, zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);

    let options= SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let base_path = folder_path.parent().unwrap_or_else(|| Path::new(""));

    for entry in WalkDir::new(folder_path) {
        /*
            Here get every file and get the file location
            search for the file data
            and replace the file name with the one stored in the data fileq
        */
        let entry = entry?;
        let path = entry.path();
        let name = match path.strip_prefix(base_path)?.to_str(){
            Some(x) => x,
            None => {
                println!("Yep, ur cooked");
                // send_error_response(&mut stream, 500, "There was a problem getting your folder, there is a chance it got corrupted :'(");
                return Err(Box::new(Error::new(std::io::ErrorKind::Other, "There was a problem getting your folder, there is a chance it got corrupted :'(")));
            }
        };

        if path.is_file() {
            let name2 = path.as_os_str().to_str().unwrap().as_bytes();
            let begining = match memmem::find(&name2[..], b"uploads").map(|p| p as usize){
                Some(x) => x,
                None => {
                    println!("oh well, it seems that the server got corrupted");
                    match log("Error identifying the file data location", 3){
                        Ok(x) => x,
                        Err(e) => {
                            // send_error_response(&mut stream, 400, &e);   
                            return Err(Box::new(Error::new(std::io::ErrorKind::Other, &*e)));
                        } 
                    }
                    return Err(Box::new(Error::new(std::io::ErrorKind::Other, "There was a problem with the server, please try again later")));
                }
            };

            let trunk = &name2[begining + "uploads".len()..];
            let path2 = format!("data{}.txt", String::from_utf8_lossy(&trunk[..]).replace("\\", "/"));

            let mut file2 = fs::File::open(path2)?;
            let mut read = Vec::new();
            match file2.read_to_end(&mut read)  {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("Error reading the file's contents: {}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            
                            // send_error_response(&mut stream, 400, "Failed to log");
                            return Err(Box::new(Error::new(std::io::ErrorKind::Other, "Failed to log in zip")));
                        }
                    };
                    return Err(Box::new(Error::new(std::io::ErrorKind::Other, "Failed to read the file for zip")));    
                }
            };

            let name3 = match memmem::find(&read[..], b"file_name:\"").map(|p| p as usize){
                Some(x) => x,
                None => {
                    println!("oh well oh well, you'r cooked");
                    match log("There has been an error generating the ZIP file", 3){
                        Ok(x) => x,
                        Err(e) => {
                            return Err(Box::new(Error::new(std::io::ErrorKind::Other, &*e)));
                        } 
                    }
                    return Err(Box::new(Error::new(std::io::ErrorKind::Other, "welp, look who's file just got corrupted :3")));
                }
            };
            let name_file = &read[name3 + "file_name:\"".len().. read.len() - 1];
            let name2 = name.replace("\\", "/");
            let slash = match memmem::rfind(&name2.as_bytes()[..], b"/").map(|p| p as usize) {
                Some(x) => x,
                None => {
                    println!("how tf mate");
                    match log("There has been an error generating the zip file", 3){
                        Ok(x) => x,
                        Err(e) => {
                            return Err(Box::new(Error::new(std::io::ErrorKind::Other, &*e)));
                        } 
                    }
                    return Err(Box::new(Error::new(std::io::ErrorKind::Other, "HOW TF MATE")));
                }
            };

            let parent = &name.as_bytes()[..slash];

            let name = &format!("{}/{}", String::from_utf8_lossy(&parent[..]), String::from_utf8_lossy(&name_file[..]));
            
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

pub fn rename_folder(mut stream: TcpStream, buffer: Request, old_folder: String, new_folder: String) {
    {
        let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                match log(&format!("Error identifing the user from the Mutex: {}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);   
                    } 
                }
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        if *folder != "" {
            // println!("before uploads/{}/{}", folder, old_folder);
            // println!("after uploads/{}/{}", folder, new_folder);
            match fs::rename(format!("uploads/{}/{}", folder, old_folder), format!("uploads/{}/{}", folder, new_folder))  {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                    send_error_response(&mut stream, 400, "Failed to rename the file");
                    return;    
                }
            };
            match fs::rename(format!("data/{}/{}", folder, old_folder), format!("data/{}/{}", folder, new_folder)) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                    send_error_response(&mut stream, 400, "Failed to rename the file");
                    return;    
                }
            };
        } else {
            // println!("uploads/{}", old_folder);
            // println!("uploads/{}", new_folder);
            match fs::rename(format!("uploads/{}", old_folder), format!("uploads/{}", new_folder)) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                    send_error_response(&mut stream, 400, "Failed to rename the file");
                    return;    
                }
            };
            match fs::rename(format!("data/{}", old_folder), format!("data/{}", new_folder)) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                    send_error_response(&mut stream, 400, "Failed to rename the file");
                    return;    
                }
            };
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("\n\nDone with the POST RENAME_FOLDER action request my guy");

    let site = web(buffer);
    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        match log("There was an error generating the web site", 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }

    let response = format!("{}{}", status_line, site);
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

pub fn delet(mut stream: TcpStream, filename: String, buffer: Request) {
        
    { 
        let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                match log(&format!("Error identifying the user from the Mutex: {}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);   
                    } 
                }
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        let folder1 = percent_decode_str(&*folder)
                        .decode_utf8_lossy()
                        .replace("+", " ")
                        .to_owned();
        if &*folder1 != "" {

            // println!("\n\nbuffer={}\n\n\n{}", String::from_utf8_lossy(&buffer.header[..]), String::from_utf8_lossy(&buffer.body.clone().unwrap()[..]));
            if let Some(_) = memmem::find(&buffer.body.clone().unwrap()[..], b"folder=") {
                let file = match memmem::find(&buffer.body.clone().unwrap()[..], b"filename=")
                    .map(|p| p as usize) {
                        Some(x) => x, 
                        None => {
                            println!("unnable to delete this ");
                            match log("There has been an error finding the file/folder the user wants deleted", 3){
                                Ok(x) => x,
                                Err(e) => {
                                    send_error_response(&mut stream, 400, &e);   
                                } 
                            }
                            send_error_response(&mut stream, 400, "The deletion cannot be completed 💔 ");
                            return;
                        }
                    };
                let file = &buffer.body.clone().unwrap()[file + "filename=".len()..];
                let filename = String::from_utf8_lossy(&file[..]);

                let filename = match percent_decode_str(&filename)
                    .decode_utf8(){
                        Ok(x) => x,
                        Err(e) => {
                            println!("It has been unnable to decode\n{:?}", e);
                            match log(&format!("Error decoding the filename: {}", e), 3){
                                Ok(x) => x,
                                Err(e) => {
                                    send_error_response(&mut stream, 400, &e);   
                                } 
                            }
                            send_error_response(&mut stream, 400, "The deletion cannot be completed since it contains weird characters");
                            return;
                        }
                    };

                let filename = match decode_html(&filename){
                    Ok(x) => x,
                    Err(e) => {
                        println!("Unnable to decode html \n{:?}", e);
                        match log(&format!("Error decoding the filename: {:?}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                send_error_response(&mut stream, 400, &e);   
                            } 
                        }
                        send_error_response(&mut stream, 400, "It contains non UTF-8 characters");
                        return;
                    }
                }.replace("+", " ");

                println!("filename suppoised to get deleted= uploads/{}/{}", folder1, filename);

                match fs::remove_file(&*format!("./uploads/{}/{}", folder1, filename)) {
                    Ok(x) => x,
                    Err(e) => {
                        match log(&format!("{}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                println!("{}", e);
                                send_error_response(&mut stream, 400, &e);
                                return;
                            }
                        }

                    send_error_response(&mut stream, 400, "Failed to delete the file");
                    return;    
                    }
                }; //dont u dare change this shi
                match fs::remove_file(&*format!("./data/{}/{}.txt", folder1, filename)) {
                    Ok(x) => x,
                    Err(e) => {
                        match log(&format!("{}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                println!("{}", e);
                                send_error_response(&mut stream, 400, &e);
                                return;
                            }
                        }

                    send_error_response(&mut stream, 400, "Failed to delete the file");
                    return;    
                    }
                };
            } else {
                
                // println!("deleting file={}/{}",folder1, filename);
                match fs::remove_file(&*format!("./uploads/{}/{}", folder1, filename)) {
                    Ok(x) => x,
                    Err(e) => {
                        match log(&format!("{}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                println!("{}", e);
                                send_error_response(&mut stream, 400, &e);
                                return;
                            }
                        }

                    send_error_response(&mut stream, 400, &format!("Failed to delete the file uploads ./uploads/{}/{}",folder1, filename));
                    return;    
                    }
                }; //dont u dare change this shi
                match fs::remove_file(&*format!("./data/{}/{}.txt", folder1, filename)) {
                    Ok(x) => x,
                    Err(e) => {
                        match log(&format!("{}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                println!("{}", e);
                                send_error_response(&mut stream, 400, &e);
                                return;
                            }
                        }
                    
                    send_error_response(&mut stream, 400, "Failed to delete the file");
                    return;    
                    }
                };
            }
        } else {
            match log("The user is not logged it and cannot delete the file/folder", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 403, "Somehow you are not logged in");
            return;
        } 
    };

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("\n\nDone with the POST delete action request my guy");
    let site = web(buffer);
    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        match log("Error generating the web site", 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }
    let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);
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

pub fn download(mut stream: TcpStream, filename: String) {
    let folder;
    {
        folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                match log(&format!("Error identifying the folder from the Mutex: {}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);   
                    } 
                }
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
    }

    // let user = memmem::find()

    // println!("Filename ig ={}/{}",folder, filename);
    let mut file = match fs::File::open(format!("uploads/{}/{}", folder, filename)){
        Ok(x) => x,
        Err(e) => {
            println!("The user's uploads folder cannot be read\n{:?}", e);
            match log(&format!("Error : {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "We are unnable to locate your file, please try again later");
            return;
        }
    };
    let mut data = match fs::File::open(format!("data/{}/{}.txt", folder, filename)){
        Ok(x) => x,
        Err(e) => {
            println!("The user's data folder cannot be read\n{:?}", e);
            match log(&format!("Error finding the file: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "We are unnable to locate your file's data, please try again later");
            return;
        }
    };

    // println!("{}", format!("download uploads/{}/{}", folder, filename));

    let mut read = Vec::new();
    match file.read_to_end(&mut read){
        Ok(x) => x,
        Err(e) =>{
            println!("uhm, the file cannot be read or no data is inside it\n{:?}", e);
            match log(&format!("Error reading from the data file: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "There is a problem reading the data of your file");
            return;
        }
    };  

    let mut file_data = String::new();
    match data.read_to_string(&mut file_data){
        Ok(x) => x,
        Err(e) => {
            match log(&format!("Error reading from file data: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    println!("{}", e);
                    send_error_response(&mut stream, 400, &e);
                    return;
                }
            }

        send_error_response(&mut stream, 400, "Failed to read data");
        return;    
        }
    };
    let file_data = file_data.into_bytes(); // here is the file type and name

    let content_type = match memmem::find(&file_data[..], b";").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("There was a problem getting the file data");
            match log("There was problem getting the file data", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "The file data proobably got corrupted\n RIP BOZO 1");
            return;
        }
    };

    let file_name = match memmem::find(&file_data[..], b"file_name:\"").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("There was a problem getting the file data");
            match log("There was a problem getting the file data", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "The file data probably got corrupted\n RIP BOZO 2");
            return;
        }
    };

    let filename = &file_data[file_name + "file_name:\"".len()..];
    let end = match memmem::find(&filename[..], b"\"").map(|p| p as usize) {
        Some(x) => x,
        None => {
            println!("There was a problem getting the file data");
            match log("Error getting the file data", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "The file data probably got corrupted\n RIP BOZO 3");
            return;
        }
    };

    let filename = &filename[..end];

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("filename={}", decode_html(&filename).unwrap());
    if filename.contains(&b"/"[0]){
        let start = match memmem::find(filename, &*b"/")
                                .map(|p| p as usize){
                                    Some(x) => x,
                                    None => {
                                        println!("");
                                        return;
                                    }
                                };

        let filename = String::from_utf8_lossy(&filename[start + 1..]);

        let response = format!(
            "{}{}\r\nContent-Disposition: W; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",  
            status_line,
            content_type,
            match decode_html(&filename){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to decode the filename {:?}", e);
                    match log(&format!("Error decoding the filename: {:?}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
                    send_error_response(&mut stream, 400, "Filename is unnable to be decoded sir");
                    return;
                }
            },
            read.len()
        );

        // println!("Done with the POST download action my guy");
        match stream.write(response.as_bytes()){
            Ok(x) => {println!("The authentification worked well"); x},
            Err(e) => {
                println!("Failed to respond ig??? {}", e);
                send_error_response(&mut stream, 400, "There was a problem responding");
                return;
            }
        };
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
        return;
    }


    let filename = String::from_utf8_lossy(&filename[..]);
    let response = format!(
        "{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_type,
        match decode_html(&filename){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to decode the filename {:?}", e);
                    match log(&format!("Error decoding the filename: {:?}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
                    send_error_response(&mut stream, 400, "Filename is unnable to be decoded sir");
                    return;
                }
            },
        read.len()
    );

    // println!("Done with the POST download action my guy");
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Write error: {}", e);
        match log(&format!("Write error: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        };
        return;
    }
    if let Err(e) = stream.write_all(&read[..]) {
        eprintln!("Write error: {}", e);
        match log(&format!("Write error: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        };
        return;
    }
    if let Err(e) = stream.flush() {
        eprintln!("Error flushing: {}", e);
        match log(&format!("Error flushing: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        };
        return;
    }
    // println!("filename={}", filename);
    
} 

pub fn add_folder(mut stream: TcpStream, buffer: Request, filename: String) {
    if filename.contains("../") {
        println!("Caught u red handed");
        println!("filename={}", filename);
        match log("The user tried to go out of bounds", 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 404, "Dont try to go out of bounds, mister");
        return;
    }

    // saves to- do%20me 
    // instead to- do me
    // println!("ADD_FOLDER\n  folder to add ={}", filename);

    {
        let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                match log(&format!("Error identifying the user from Mutex: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        let folder = percent_decode_str(&*folder)
                        .decode_utf8_lossy();
        if *folder != *"" {
            if Path::new(&format!("uploads/{}/{}", folder, filename)).exists() {
                send_error_response(&mut stream, 403, "Folder already exists");
                println!("uploads/{}/{}", folder, filename);
                return;
            }
            match fs::create_dir_all(format!("uploads/{}/{}", folder, filename)){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to create the folder {}", e);
                    match log(&format!("Error creating the folder: {}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
                    send_error_response(&mut stream, 500, "We were unnable to create you folder, please try again later");
                    return;
                }
            }; // handle gracefully
            match fs::create_dir_all(format!("data/{}/{}", folder, filename)){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to create the folder for data {}", e);
                    match log(&format!("Error generating the data folder: {}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
                    send_error_response(&mut stream, 500, "We were unnable to create you folder for data, please try again later");
                    return;
                }
            };
        
            // println!("uploads/{}/{:?}\n\n",folder, filename);
            
        } else {
            match log("The folder empty so i cannot add it", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 403, "The folder name is empty");
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";
    let site = web(buffer);
    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        match log("There was an error generating the site", 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }
    let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);
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

fn add_file_in_folder(
    mut stream: TcpStream,
    buffer: Request,
    content: &[u8],
    content_type: &str,
    filename: String,
) {
    let folder = match memmem::find(&buffer.body.clone().unwrap()[..], b"name=\"folder\"").map(|p| p as usize) {
        Some(f) => f,
        None => {
            match log("The folder that was supposed to be put the file in did not get identified", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 404, "Folder not found");
            return;
        }
    };

    // println!("should add a file in da folder");
    let folder = &buffer.body.clone().unwrap()[folder + "name=\"folder\"".len() + "\r\n\r\n".len()..];

    // println!("folder? = {}", String::from_utf8_lossy(&folder[..]));
    let end = match memmem::find(folder, b"\r\n").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("this shit probly got corrupted");
            match log("The request probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 400, "YOur request oribably got corrupted");
            return;
            
        }
    };
    let folder = &folder[..end];

    // println!("filename before change = {}", filename);

    let filename = format!("{}/{}", String::from_utf8_lossy(&folder[..]), filename);

    // println!("filename after change = {}", filename);

    add_file(stream, buffer, content, content_type, filename);
}

fn add_file(
    mut stream: TcpStream,
    buffer: Request,
    content: &[u8],
    content_type: &str,
    filename: String,
) {
    // do some shady shit

    let user_filename = filename.clone();
    let disk_filename = Uuid::new_v4();

    let mut disk_filename = String::from(disk_filename);

    let extension = match memmem::rfind(&user_filename.as_bytes()[..], b".").map(|p| p as usize){
        Some(x) => x,
        None => 9999999999,
    };

    if extension== 9999999999 {
        disk_filename.push_str(".txt");
    } else {
        let extension = &filename.as_bytes()[extension..];
        disk_filename.push_str(&String::from_utf8(extension.to_vec()).unwrap());
    }

    {
        let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                match log(&format!("Error identifying the user from Mutex: {:?}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);   
                    } 
                }
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        // println!("folder im supposed to save the file={:?}", folder);
        if *folder != "" {
            let filename_upload = format!("uploads/{}/{}", 
                percent_decode_str(&folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned(),
                disk_filename);
            // println!("upload filename ={}\n\n", filename_upload);

            let mut file = fs::File::create(&filename_upload).unwrap();
            match file.write_all(content) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                send_error_response(&mut stream, 400, "Failed to write to file");
                return;    
                }
            }

            let filename_data = format!("data/{}/{}.txt", 
                percent_decode_str(&folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned(),
                disk_filename);
            // println!("filename_data = {}", filename_data);
            // println!("filename_data = {}", filename_data);

            let date = match get_date() {
                Ok(x) => x,
                Err(e) => {
                    println!("Error getting the date: {}", e);
                    panic!("YOu are fucked because chronos doesnt work (date)");
                }   
            };

            let time = match get_time() {
                Ok(x) => x,
                Err(e) => {
                    println!("Error getting the date: {}", e);
                    panic!("YOu are fucked because chronos doesnt work (time)");
                }   
            };
            let mut file2 = fs::File::create(&filename_data).unwrap();
            match file2.write_all(&format!("Content-Type:{};\r\nfile_name:\"{}\";\r\ndate:\"{}\";\r\ntime:\"{}\";", content_type, user_filename, date, time).into_bytes()[..]) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                send_error_response(&mut stream, 400, "Failed to write to data file");
                return;    
                }
            }//idk how this works
            //till here we saved the file on the server (hopefully)

        } else {
            let filename_upload = format!("uploads/{}", disk_filename);
            // println!("upload filename ={}", filename_upload);

            let mut file = fs::File::create(&filename_upload).unwrap();
            match file.write_all(content) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                send_error_response(&mut stream, 400, "Failed to write to file");
                return;    
                }
            }

            let filename_data = format!("data/{}.txt", disk_filename);
            // println!("filename_data = {}", filename_data);
            // println!("filename_data = {}", filename_data);
            let mut file2 = fs::File::create(&filename_data).unwrap();

            match file2.write_all(&format!("Content-Type:{};\r\nfile_name:\"{}\"", content_type, user_filename).into_bytes()[..]) {
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("{}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            println!("{}", e);
                            send_error_response(&mut stream, 400, &e);
                            return;
                        }
                    }

                send_error_response(&mut stream, 400, "Failed to write to data file");
                return;    
                }
            } //idk how this works
            //till here we saved the file on the server (hopefully)
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("\n\nDone with the POST add_file request my guy");
    let site = web(buffer);
    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        match log("There was an error generating the web server", 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }
    let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);

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

 
