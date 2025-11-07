use super::*;

pub fn upload_file(mut stream: TcpStream, buffer: Vec<u8>) {
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

pub fn download_folder(mut stream: TcpStream, folder_name: String) {
    let mut folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
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
        let name = match path.strip_prefix(base_path)?.to_str(){
            Some(x) => x,
            None => {
                println!("Yep, ur cooked");
                // send_error_response(&mut stream, 500, "There was a problem getting your folder, there is a chance it got corrupted :'(");
                return Err(Box::new(Error::new(std::io::ErrorKind::Other, "There was a problem getting your folder, there is a chance it got corrupted :'(")));
            }
        };

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

pub fn rename_folder(mut stream: TcpStream, buffer: Vec<u8>, old_folder: String, new_folder: String) {
    {
        let mut folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
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

    let site = web(&buffer[..]);
    if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }

    let response = format!("{}{}", status_line, site);
    match stream.write(response.as_bytes()){
        Ok(x) => {println!("The authentification worked well"); x},
        Err(e) => {
            send_error_response(&mut stream, 400, "There was a problem responding");
            println!("Failed to respond ig???");
            return;
        }
    };
    match stream.flush(){
        Ok(x) => x,
        Err(x) => {
            send_error_response(&mut stream, 400, "How tf did this fail");
            println!("Failed to respond ig???");
            return;
        }
    };
}

pub fn delet(mut stream: TcpStream, filename: String, buffer: Vec<u8>) {
        
        { 
        let mut folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        let folder1 = percent_decode_str(&*folder)
                        .decode_utf8_lossy()
                        .replace("+", " ")
                        .to_owned();
        if &*folder1 != "" {

            println!("\n\nbuffer={}", String::from_utf8_lossy(&buffer[..]));
            if let Some(folder) = memmem::find(&buffer[..], b"folder=") {
                let file = match memmem::find(&buffer[..], b"filename=")
                    .map(|p| p as usize) {
                        Some(x) => x, 
                        None => {
                            println!("unnable to delete this ");
                            send_error_response(&mut stream, 400, "The deletion cannot be completed 💔 ");
                            return;
                        }
                    };
                let file = &buffer[file + "filename=".len()..];
                let filename = String::from_utf8_lossy(&file[..]);

                let filename = match percent_decode_str(&filename)
                                    .decode_utf8(){
                                        Ok(x) => x,
                                        Err(e) => {
                                            println!("It has been unnable to decode\n{:?}", e);
                                            send_error_response(&mut stream, 400, "The deletion cannot be completed since it contains weird characters");
                                            return;
                                        }
                                    };

                let filename = match decode_html(&filename){
                    Ok(x) => x,
                    Err(e) => {
                        println!("Unnable to decode html \n{:?}", e);
                        send_error_response(&mut stream, 400, "It contains non UTF-8 characters");
                        return;
                    }
                }.replace("+", " ");

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
    let site = web(&buffer[..]);
    if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }
    let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);
    match stream.write(response.as_bytes()){
        Ok(x) => {println!("The authentification worked well"); x},
        Err(e) => {
            send_error_response(&mut stream, 400, "There was a problem responding");
            println!("Failed to respond ig???");
            return;
        }
    };
    match stream.flush(){
        Ok(x) => x,
        Err(x) => {
            send_error_response(&mut stream, 400, "How tf did this fail");
            println!("Failed to respond ig???");
            return;
        }
    };
}

pub fn download(mut stream: TcpStream, filename: String, buffer: Vec<u8>) {
    let entries = match fs::read_dir("uploads"){
        Ok(x) => x,
        Err(e) => {
            println!("I was unnable to read the directory \"uploads\"\n{:?}", e);
            send_error_response(&mut stream, 404, "There is a problem accessing you files, please try again later");
            return;

        }
    };
    let mut file_names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = match entry{
            Ok(x) => x,
            Err(e) => {
                println!("No users uploads found\n{:?}", e);
                send_error_response(&mut stream, 404, "There is a problem accessing your uploads, try again later");
                return;
            }
        };
        let file_name = match entry.file_name().into_string(){
            Ok(x) => x,
            Err(e) => {
                println!("The user's username is unnable to be converted to string\n{:?}", e);
                send_error_response(&mut stream, 404, "The user contains illegitimate characters");
                return;
            }
        };
        file_names.push(file_name);
    }

    let mut folder;

    {
        folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
    }

    // let user = memmem::find()

    println!("Filename ig ={}/{}",folder, filename);
    let mut file = match fs::File::open(format!("uploads/{}/{}", folder, filename)){
        Ok(x) => x,
        Err(e) => {
            println!("The user's uploads folder cannot be read\n{:?}", e);
            send_error_response(&mut stream, 404, "We are unnable to locate your file, please try again later");
            return;
        }
    };
    let mut data = match fs::File::open(format!("data/{}/{}.txt", folder, filename)){
        Ok(x) => x,
        Err(e) => {
            println!("The user's data folder cannot be read\n{:?}", e);
            send_error_response(&mut stream, 404, "We are unnable to locate your file's data, please try again later");
            return;
        }
    };

    println!("{}", format!("download uploads/{}/{}", folder, filename));

    let mut read = Vec::new();
    match file.read_to_end(&mut read){
        Ok(x) => x,
        Err(e) =>{
            println!("uhm, the file cannot be read or no data is inside it\n{:?}", e);
            send_error_response(&mut stream, 404, "There is a problem reading the data of your file");
            return;
        }
    };  

    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    let status_line = "HTTP/1.1 200 OK\r\n";

    // println!("filename={}", decode_html(&filename).unwrap());
    if filename.contains("/"){
        let start = match memmem::find(filename.as_bytes(), b"/")
                                .map(|p| p as usize){
                                    Some(x) => x,
                                    None => {
                                        println!("");
                                        return;
                                    }
                                };

        let filename = String::from_utf8_lossy(&filename.as_bytes()[start + 1..]);

        let response = format!(
            "{}{}\r\nContent-Disposition: W; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
            status_line,
            content_type,
            match decode_html(&filename){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to decode the filename {:?}", e);
                    send_error_response(&mut stream, 400, "Filename is unnable to be decoded sir");
                    return;
                }
            },
            read.len()
        );

        println!("Done with the POST download action my guy");
        match stream.write(response.as_bytes()){
            Ok(x) => {println!("The authentification worked well"); x},
            Err(e) => {
                send_error_response(&mut stream, 400, "There was a problem responding");
                println!("Failed to respond ig???");
                return;
            }
        };
        match stream.write(&read[..]){
            Ok(x) => {println!("The authentification worked well"); x},
            Err(e) => {
                send_error_response(&mut stream, 400, "There was a problem responding");
                println!("Failed to respond ig???");
                return;
            }
        };
        match stream.flush(){
            Ok(x) => x,
            Err(x) => {
                send_error_response(&mut stream, 400, "How tf did this fail");
                println!("Failed to respond ig???");
                return;
            }
        };
        return;
    }


    let response = format!(
        "{}{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_type,
        match decode_html(&filename){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to decode the filename {:?}", e);
                    send_error_response(&mut stream, 400, "Filename is unnable to be decoded sir");
                    return;
                }
            },
        read.len()
    );

    println!("Done with the POST download action my guy");
    match stream.write(response.as_bytes()){
        Ok(x) => {println!("The write worked well"); x},
        Err(e) => {
            send_error_response(&mut stream, 400, "There was a problem responding");
            println!("Failed to respond ig???");
            return;
        }
    };
    match stream.write(&read[..]){
        Ok(x) => {println!("The authentification worked well"); x},
        Err(e) => {
            send_error_response(&mut stream, 400, "There was a problem responding");
            println!("Failed to respond ig???");
            return;
        }
    };
    match stream.flush(){
        Ok(x) => x,
        Err(x) => {
            send_error_response(&mut stream, 400, "How tf did this fail");
            println!("Failed to respond ig???");
            return;
        }
    };
    // println!("filename={}", filename);
    
}

pub fn add_folder(mut stream: TcpStream, buffer: &[u8], filename: String) {
    if filename.contains("../") {
        println!("Caught u red handed");
        println!("filename={}", filename);

        send_error_response(&mut stream, 404, "Dont try to go out of bounds, mister");
        return;
    }

    // saves to- do%20me 
    // instead to- do me
    println!("ADD_FOLDER\n  folder to add ={}", filename);

    {
        let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        let folder = percent_decode_str(&*folder)
                        .decode_utf8_lossy();
        if *folder != *"" {
            if Path::new(&format!("uploads/{}/{}", folder, filename)).exists() {
                send_error_response(&mut stream, 403, "Folder already exists");
                return;
            }
            match fs::create_dir_all(format!("uploads/{}/{}", folder, filename)){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to create the folder");
                    send_error_response(&mut stream, 500, "We were unnable to create you folder, please try again later");
                    return;
                }
            }; // handle gracefully
            match fs::create_dir_all(format!("data/{}/{}", folder, filename)){
                Ok(x) => x,
                Err(e) => {
                    println!("Unnable to create the folder for data");
                    send_error_response(&mut stream, 500, "We were unnable to create you folder for data, please try again later");
                    return;
                }
            };
        
            println!("uploads/{}/{:?}\n\n",folder, filename);
            
        } else {
            send_error_response(&mut stream, 403, "Somehow you are not connected");
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";
    let site = web(buffer);
    if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }
    let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);
    match stream.write(response.as_bytes()){
        Ok(x) => {println!("The write worked well"); x},
        Err(e) => {
            send_error_response(&mut stream, 400, "There was a problem responding");
            println!("Failed to respond ig???");
            return;
        }
    };
    match stream.flush(){
        Ok(x) => x,
        Err(x) => {
            send_error_response(&mut stream, 400, "How tf did this fail");
            println!("Failed to respond ig???");
            return;
        }
    };
    
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
    let end = match memmem::find(folder, b"\r\n").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("this shit probly got corrupted");
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
    buffer: &[u8],
    content: &[u8],
    content_type: &str,
    filename: String,
) {
    // do some shady shit
    {
        let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
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
    let site = web(buffer);
    if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }
    let response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, site);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
