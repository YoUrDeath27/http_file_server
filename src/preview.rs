use super::*; 

pub fn web_send_image(mut stream:TcpStream, buffer: Request){ 

    let user = checkAuth(&mut stream, buffer.clone());
    let file = match memmem::find(&buffer.header[..], b" HTTP/1.1").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("Probably the request got corrupted");
            match log("Error finding the begining of the request", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    // return String::from("");
                } 
            }
            send_error_response(&mut stream, 400, "The request got corrupted");
            return; 
        }
    }; 

    let start = match memmem::find(&buffer.header[..], b"users").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("Probably the request got corrupted bitch");
            match log("Error finding the uploads path to the file", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    // return String::from("");
                } 
            }
            send_error_response(&mut stream, 400, "The request got corrupted");
            return;
        }
    };
    let name = String::from_utf8_lossy(&buffer.header[start..file])
        .trim_start_matches('/')
        .replace("//", "/");
    println!("name: {:?}", name);

    let uploads_prefix = format!("users/{}/uploads/", user);
    let profile_prefix = format!("users/{}/", user);
    let (upload_path, metadata_path, relative_path) =
        if let Some(path) = name.strip_prefix(&uploads_prefix) {
            if path.is_empty() || path.contains("..") {
                send_error_response(&mut stream, 400, "The preview path is invalid");
                return;
            }
            (
                format!("users/{}/uploads/{}", user, path),
                format!("users/{}/data/{}.txt", user, path),
                path.to_string(),
            )
        } else if let Some(path) = name.strip_prefix(&profile_prefix) {
            if !path.starts_with("pfp.") || path.contains("..") {
                send_error_response(&mut stream, 400, "The preview path is invalid");
                return;
            }
            (
                format!("users/{}/{}", user, path),
                format!("users/{}/pfp.txt", user),
                path.to_string(),
            )
        } else {
            send_error_response(&mut stream, 400, "The preview path is invalid");
            return;
        };
    let mut file = match fs::File::open(&upload_path){
        Ok(x) => x,
        Err(e) => {
            println!("File that was attempted to be opened: {}", upload_path);
            println!("There has been an error opening the image\n{}", e);
            match log(&format!("Error opening the file: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    // return String::from("");
                } 
            }

            send_error_response(&mut stream, 400, &format!("There was a problem opening the file: {}", e));   
            return;
        }
    };

    println!("image data: {:?}", metadata_path);
    let mut data = match fs::File::open(&metadata_path) {
        Ok(x) => x,
        Err(e) => {
            println!("The data image file does not exist {}", e);
            match log(&format!("The data image file does not exist: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    // return String::from("");
                } 
            }
            send_error_response(&mut stream, 400, "Unnable to get the data for preview, the image does not exist");
            return;
        }
    };

    let filename = std::path::Path::new(&relative_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let mut file_data = String::new();
    match data.read_to_string(&mut file_data) {
        Ok(x) => x,
        Err(e) => {
            match log(&format!("{}", e), 3) {
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);
                    return;
                }
            }
        return;
        }
    };

    let mut read = Vec::new();
    match file.read_to_end(&mut read){
        Ok(x) => x,
        Err(e) =>{
            println!("uhm, the file cannot be read or no data is inside it\n{:?}", e);
            match log(&format!("The file data cannot be read: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    // return String::from("");
                } 
            }
            send_error_response(&mut stream, 404, "There is a problem reading the data of your file");
            return;
        }
    };  

    /*
    *********************************************************************************************
        This response needs to be re-worked 
    *********************************************************************************************
    */
    let content_type = file_data
        .lines()
        .find_map(|line| line.strip_prefix("Content-Type:"))
        .map(str::trim)
        .unwrap_or("application/octet-stream");
    let file_name = filename.replace('"', "");
    let status_line = "HTTP/1.1 200 OK\r\n";
    let response = format!(
        "{}Content-Type: {}\r\nContent-Length: {}\r\nContent-Disposition: inline; filename=\"{}\"\r\n\r\n",
        status_line,
        content_type.trim_end_matches(';'),
        read.len(),
        file_name,
    );

    println!("\n\n\nresponse: {}", response);

    println!("CWD: {:?}", std::env::current_dir());

    // println!("If this shit doesnt work imma tweak out");
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Write error: {}", e);
        match log(&format!("Write error: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
    }
    if let Err(e) = stream.write_all(&read[..]) {
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
 
