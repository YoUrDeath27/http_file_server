use super::*; 

pub fn web_send_image(mut stream:TcpStream, buffer: Request){

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

    let start = match memmem::find(&buffer.header[..], b"uploads").map(|p| p as usize){
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
    let name = String::from_utf8_lossy(&buffer.header[start..file]);
    let mut file = match fs::File::open(&*name){
        Ok(x) => x,
        Err(e) => {
            println!("File that was attempted to be opened: {}", name);
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

    let mut data = String::from(name.clone());
    data.replace_range(.."uploads".len(), "data");
    data.replace_range(data.len().., ".txt"); //add at the end .txt

    // println!("image data: {}", data);
    let mut data = match fs::File::open(data) { //why errorr??????????????????????//
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

    let buffer1 = name.as_bytes();
    let filename = match  memmem::rfind(&buffer1, b"/").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("ur seriously cooked if you get this error");
            match log("The image filename cannot be identified", 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    // return String::from("");
                } 
            }
            send_error_response(&mut stream, 400, "Ur cooked chat");
            return;
        }
    };
    let filename = &buffer1[filename..];
    let mut content_type = String::new();
    match data.read_to_string(&mut content_type) {
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

    let status_line = "HTTP/1.1 200 OK\r\n";
    let response = format!(
        "{}{}\r\nContent-DIsposition: W; filename = \"{}\"\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_type,
        String::from_utf8_lossy(&filename[..]),
        read.len()
    );

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
 
