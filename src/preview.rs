use super::*; 

pub fn web_send_image(mut stream:TcpStream, buffer: Vec<u8>){

    let file = match memmem::find(&buffer[..], b" HTTP/1.1").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("Probably the request got corrupted");
            send_error_response(&mut stream, 400, "The request got corrupted");
            return;
        }
    };

    let start = match memmem::find(&buffer, b"uploads").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("Probably the request got corrupted bitch");
            send_error_response(&mut stream, 400, "The request got corrupted");
            return;
        }
    };
    let name = String::from_utf8_lossy(&buffer[start..file]);
    let mut file = match fs::File::open(&*name){
        Ok(x) => x,
        Err(e) => {
            println!("File that was attempted to be opened: {}", name);
            println!("There has been an error opening the image\n{}", e);
            return;
        }
    };

    let mut data = String::from(name.clone());
    data.replace_range(.."uploads".len(), "data");
    data.replace_range(data.len().., ".txt");

    println!("image data: {}", data);
    let mut data = match fs::File::open(data) { //why errorr??????????????????????//
        Ok(x) => x,
        Err(e) => {
            println!("The data image file does not exist");
            send_error_response(&mut stream, 400, "Unnabl
            e to get the data for preview, the image does not exist");
            return;
        }
    };

    let buffer1 = name.as_bytes();
    let filename = match  memmem::rfind(&buffer1, b"/").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("ur seriously cooked if you get this error");
            send_error_response(&mut stream, 400, "Ur cooked chat");
            return;
        }
    };
    let filename = &buffer1[filename..];
    let mut content_type = String::new();
    data.read_to_string(&mut content_type);

    let mut read = Vec::new();
    match file.read_to_end(&mut read){
        Ok(x) => x,
        Err(e) =>{
            println!("uhm, the file cannot be read or no data is inside it\n{:?}", e);
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

    println!("If this shit doesnt work imma tweak out");
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

