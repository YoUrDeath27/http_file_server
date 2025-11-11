use super::*;

lazy_static!{
    pub static ref SHOW_FOLDER: Mutex<String> = Mutex::new(String::from(""));
    pub static ref USERS_ATTEMPTS: Mutex<HashMap<String, (u32, Option<Instant>)>> = Mutex::new(HashMap::new());
} 

pub fn decode_Windows_1255(bytes: &[u8]) -> String{
    // Try UTF-8 first
    if let Ok(utf8_str) = String::from_utf8(bytes.to_vec()) {
        return utf8_str;
    }
    
    // Fall back to Windows-1252
    WINDOWS_1252.decode(bytes, DecoderTrap::Replace).unwrap_or_else(|_| String::from("Invalid encoding"))
}

pub fn get_boundary(buffer: &Vec<u8>) -> Option<Vec<u8>> {
    let buffer = &buffer[..];
    let boundary_b = match memmem::find(buffer, b"boundary=")
        .map(|pos| pos as usize){
            Some(x) => x,
            None => {
                println!("Somehow you managed to do something");
                return None;
            }
        };
    let boundary_b = &buffer[boundary_b + "boundary=".len()..];
    let boundary_right = match memmem::find(boundary_b, b"\r\n")
        .map(|pos| pos as usize){
            Some(x) => x,
            None => {
                println!("YOu are really trying to break me huh?");
                return None;
            }
        };
    let boundary = &boundary_b[..boundary_right];
    let boundary = format!("--{}", String::from_utf8_lossy(&boundary[..])).into_bytes();
    Some(boundary)
}

pub fn parse_file<'a>(
    stream: &mut TcpStream,
    buffer: &'a mut Request,
    boundary: &[u8],
) -> Result<(&'a [u8], &'a str, String), &'static str> {
    let content_boundary = match memmem::find_iter(buffer.body.as_ref().unwrap(), &boundary)
        .map(|p| p as usize)
        .next()
    {
        Some(c) => c,
        None => {
            send_error_response(stream, 400, "Content not found");
            return Err("fuck head, cant find the content");
        }
    };
    let info = &buffer.body.as_mut().unwrap()[content_boundary + boundary.len()..];

    //the content part
    let mut contents_find = memmem::find_iter(info, b"\r\n\r\n").map(|p| p as usize);
    if let Some(_) = memmem::find(&buffer.header[..], b"name=\"folder\"").map(|p| p as usize) {
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
    let mut content_type = memmem::find_iter(&buffer.header[..], b"Content-Type:").map(|p| p as usize);
    let _ = content_type.next();

    if let Some(_) = memmem::find(&buffer.header[..], b"name=\"folder\"").map(|p| p as usize) {
        let content_type = match content_type.next(){
            Some(x) => x,
            None => {
                println!("We might have some trouble boss");
                send_error_response(stream, 500, "Why are you trying to break the server boss?");
                return Ok((&[], "", Default::default()));
            }
        };
        let content_type = &buffer.header[content_type + "Content-Type:\"".len()..];

        // println!("content-type is equal to IDFKK ={}\n\n\n\n", String::from_utf8_lossy(&content_type[..]));

        // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
        let end = match memmem::find(&content_type, b"\r\n\r\n")
            .map(|p| p as usize){
                Some(x) => x,
                None => {
                    println!("This is not ok ");
                    send_error_response(stream, 500, "This file or request is corrupted <br> stop it");
                    return Ok((&[], "", Default::default()));
                }
            };
        let content_type = &content_type[..end];

        //2

        // println!("Content-Type = {}", String::from_utf8_lossy(&content_type[..]));

        //filename part
        let filename = match memmem::find_iter(info, b"filename=")
            .map(|p| p as usize)
            .next(){
                Some(x) => x,
                None => {
                    println!("Why does this file not have a name?");
                    send_error_response(stream, 400, "The file u tried to upload does not contain a name<br>weird");
                    return Ok((&[], "", Default::default()));
                }
            };
        let filename_data = &info[filename + "filename=".len()..];

        let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
        let filename_1 = match filename1.next() {
            Some(x) => x,
            None => {
                println!("Nope");
                send_error_response(stream, 400, "Did you play around before sending this file?");
                return Ok((&[], "", Default::default()));
            }
        };
        let filename_2 = match filename1.next() {
            Some(x) => x,
            None => {
                println!("Nope");
                send_error_response(stream, 400, "Did you play around before sending this file? <br>Are you sure about that?");
                return Ok((&[], "", Default::default()));
            }
        };
        let filename = &filename_data[filename_1 + 1..filename_2];
        //3
        // println!("filename = {:?}", String::from_utf8_lossy(&filename[..]));
        // let file = String::from_utf8_lossy(&filename[..]).to_string();

        // Decode the filename from bytes
        let file = decode_Windows_1255(&filename[..]);

        // Decode HTML entities in the filename
        let file = decode_html(&file).unwrap_or_else(|_| file);

        return Ok((
            content,
            std::str::from_utf8(content_type).unwrap_or("application/octet-stream"),
            file.replace(" ", "_"),
        ));
    }
    
    let content_type = match content_type.next() {
        Some(x) => x,
        None => {
            println!("How did you get past the first check?");
            send_error_response(stream, 400, "I am impressed if you managed to get this error");
            return Ok((&[], "", Default::default()));
        }
    };
    let content_type = &buffer.header[content_type + "Content-Type:\"".len()..];

    // println!("content-type is equal to ={}", String::from_utf8_lossy(&content_type[..]));

    // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let end = match memmem::find(&content_type, b"\r\n\r\n")
        .map(|p| p as usize){
            Some(x) => x,
            None => {
                println!("Looks like someone played around a bit");
                send_error_response(stream, 400, "The file/request has probably been corrupted during transmission");
                return Ok((&[], "", Default::default()));
            }
        };
    let content_type = &content_type[..end];

    //2
    let filename = match memmem::find_iter(info, b"filename=")
        .map(|p| p as usize)
        .next(){
            Some(x) => x,
            None => {
                println!("Why does this file not have a name?");
                send_error_response(stream, 400, "The file u tried to upload does not contain a name<br>weird");
                return Ok((&[], "", Default::default()));
            }
        };
    let filename_data = &info[filename + "filename=".len()..];

    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
    let filename_1 = match filename1.next() {
        Some(x) => x,
        None => {
            println!("Nope");
            send_error_response(stream, 400, "Did you play around before sending this file?");
            return Ok((&[], "", Default::default()));
        }
    };
    let filename_2 = match filename1.next() {
        Some(x) => x,
        None => {
            println!("Nope");
            send_error_response(stream, 400, "Did you play around before sending this file? <br>Are you sure about that?");
            return Ok((&[], "", Default::default()));
        }
    };
    let filename = &filename_data[filename_1 + 1..filename_2];

    //3
    println!("Parse Upload filename = {:?}", String::from_utf8_lossy(&filename_data[..]));
    println!("Parse Upload filename = {:?}", filename_data);
    println!("Parse Upload filename = {:?}", decode_Windows_1255(&filename[..]));

    // upload filename =uploads/"What’s the craziest way you’ve seen someone get humbled_&#129300;.mp4"
    // Content-Type: video/mp4

    // println!("Parse Upload content = {:?}", String::from_utf8_lossy(&content[..]));

    Ok((
        content,
        std::str::from_utf8(content_type).unwrap_or("application/octet-stream"),
        decode_Windows_1255(&filename[..]).replace(" ", "_"), //i think i should repplace this with encode_percent or smth so " " -> %20
    ))
}

pub const MAX_UPLOAD_SIZE: usize = 40 * 1024 * 1024; // 40MB

pub const ALLOWED_MIME_TYPES: &[&str] = &[
    "audio/wav",
    "audio/mp3",
    "application/x-zip-compressed",
    "video/mp4",
    "text/plain",
    "image/jpeg",
    "image/png",
    "application/pdf",
    "application/octet-stream",
]; 

pub const Image_Types: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/gif",
    "image/svg+xml",
    "image/webp",
    "image/tiff",
    "image/bmp",
];

pub const Video_Types: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/x-msvideo",
    "video/mpeg",
    "video/ogg"
];

pub const Audio_Types: &[&str] = &[
    "audio/mpeg",
    "audio/wav",
    "audio/mp4",
    "audio/ogg",
    "audio/opus",
    "audio/aac",
    "audio/webm",
];

pub const Text_Types: &[&str] = &[
    "text/plain",
    "text/html",
    "text/css",
    "text/javascript",
    "text/csv",
    "application/pdf",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/json",
];
