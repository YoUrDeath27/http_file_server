use super::*;

lazy_static!{
    pub static ref SHOW_FOLDER: Mutex<String> = Mutex::new(String::from(""));
    pub static ref USERS_ATTEMPTS: Mutex<HashMap<String, (u32, Option<Instant>)>> = Mutex::new(HashMap::new());
    pub static ref LOGFILE: Mutex<fs::File> = Mutex::new(
        match fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(format!("logs/{}.txt", 
            match get_date() {
                Ok(x) => x,
                Err(e) => {
                    println!("Error getting the date: {}", e);
                    panic!("YOu are fucked because chronos doesnt work (date)");
                }   
            })) {
            Ok(mut x) => {
                match x.write(format!("[{}]*******************************NEW LOG CREATED*******************************", 
                    match get_time() {
                        Ok(x) => x,
                        Err(e) => {
                            println!("Error getting the time: {}", e);
                            panic!("YOu are fucked because chronos doesnt work (time)");
                        } 
                    }).as_bytes()){
                    Ok(x) => x,
                    Err(e) => {
                        println!("Error writing the starting {}", e);
                        panic!("There is a problem writing the starting line on the log file");
                        
                    }
                };
                x
            },
            Err(e) => {
                println!("Probably because the file doesnt exist {}", e);
                match create_log() {
                    Ok(x) => x,
                    Err(e) => {
                        println!("You are fucked because the program cannot create the log file\n{}", e);
                        panic!("The program was unnable to create the log file");
                    }
                }
            }
        }
    );

    pub static ref LOG_LOCATION: Mutex<String> = {
        Mutex::new({
            // println!("Hope it gets here"); //it does lil guy
            let file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(format!("logs/{}.txt", 
                match get_date() {
                    Ok(x) => x,
                    Err(e) => {
                        println!("Error getting the date: {}", e);
                        panic!("YOu are fucked because chronos doesnt work (date)");
                    }
                }
            ));
            let file_s = format!("{:?}", file.unwrap()).replace("\\\\", "/");
            // println!("file ig = {:?};", file_s);
            // println!("file in byes ig = {:?}", fileS.as_bytes());
            let path = match memmem::find(&file_s.as_bytes()[..], b"/?/").map(|p| p as usize) {
                Some(x) => x,
                None => {
                    println!("Oh well, would u look at that, my way doesnt work");
                    panic!("The File cannot be transfered the way i thought");
                }
        };

            let path = &file_s.as_bytes()[path + "/?/".len()..];
            let end = match memmem::rfind(&path[..], b"\"").map(|p| p as usize) {
                Some(x) => x,
                None => {
                    println!("If the previous one didnt fail then im surprised");
                    panic!("This shit is so fucked up mate");
                }
            };
            let path = &path[..end];
            // println!("path hopefully: {}", String::from_utf8_lossy(&path[..]));
            
            String::from_utf8_lossy(&path[..]).to_string() 
        })
    };

} 

pub fn check_log_location(path: &str) -> Result<(), String> {
    let origin = match LOG_LOCATION.lock(){
        Ok(x) => x,
        Err(e) => {
            println!("You are cooked my guy\n {}", e);
            panic!("YOu are fucked (log location mutex error, proobly poisoning)");
        }
    };

    // println!("log_location = {}", origin);
    // println!("to check     = {}", path);

    if *origin != path {
        return Err(String::from("The file log has been deleted/moved"));
    }
    Ok(())

}

pub fn decode_windows_1255(bytes: &[u8]) -> String{
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

pub fn log(message: &str, variant: i8) -> Result<(), String>{ 

    match logging(message, variant){
        Ok(x) => x,
        Err(e) => {
            println!("error : {}", e);
            let file = match create_log() {
                Ok(x) => x,
                Err(e) => {
                    println!("there has certainly been an error {}", e);
                    panic!("There has been an error creating the log file"); //keep building the logic mate
                }
            };
            let mut previous;
            {
                previous = match LOGFILE.lock() {
                    Ok(x) => x,
                    Err(e) => {
                        println!("error with logfile {}", e);
                        panic!("It was a mistake getting the logfile mutex");
                    }
                };
            }
            *previous = file;

            return Err(format!("{}", e));
        } 
    }
    Ok(())
}

pub fn logging(message: &str, variant: i8) -> Result<(), String> { //i dont wnat it to return anything and solve on it's own the problem
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

    /* 
        variant
        0 - normal log
        1 - warning
        2 - client error
        3 - server error 
        4 - fatal error
     */

    let phrase = match variant {
        1 => format!("\n\nWARNING[{} {}] {}\n", 
                date, 
                time, 
                message),
        2 => format!("\n*******CLIENT SIDE ERROR*******\n[{} {}] {}\n", 
                date, 
                time, 
                message),
        3 => format!("\n*******SERVER SIDE ERROR*******\n[{} {}] {}\n", 
                date, 
                time, 
                message),
        4 => format!("\n*******FATAL ERROR*******\n[{} {}] {}\n", 
                date, 
                time, 
                message),
        // _ => format!("\n[{} {}] {}", 
        //         date, 
        //         time, 
        //         message),
        _ => format!("\n {}", 
                    message),
    };


    let _file = match LOGFILE.lock(){
        Ok(mut x) => {
            //check log location here
            let log_path = format!("{:?}", x).replace("\\\\", "/");
            // println!("{:?}", log_path);
            let p = match memmem::find(&log_path.as_bytes()[..], b"//?/").map(|p| p as usize) {
                Some(x) => x,
                None => {
                    println!("oh well you are cooked");
                    // return Err(String::from("sigma is not doign great"));
                    panic!("There was a problem with the file path");
                }
            };
            let log_path = &log_path.as_bytes()[p + "//?/".len()..];
            let end = match memmem::rfind(&log_path[..], b"\"").map(|p| p as usize) {
                Some(x) => x,
                None => {
                    println!("oh well you are cooked");
                    // return Err(String::from("sigma is not doign great pt 2"));
                    panic!("There was a problem with the file path");
                }
            };

            let log_path = String::from_utf8_lossy(&log_path[..end]);
            match check_log_location(&log_path) {
                Ok(x) => x,
                Err(e) => {
                    return Err(String::from(format!("{}", e)));   
                }
            }

            
            match x.write(phrase.as_bytes()){
                Ok(t) => t,
                Err(e) => {
                    println!("it happenes again\n {}", e);
                    // panic!("error encountered: {}", e);
                    return Err(String::from("You are cooked"));
                }
            };
            x
        },
        Err(e) => {
            println!("Can't open the logfile from themutex\n{:?}", e);
            // send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
            return Err(String::from("You are cooked more"));
        }
    };
    // println!("whatever: {:?}", logFile);
    Ok(())
}

pub fn create_log() -> Result<fs::File, String> {
    match fs::File::create(format!("logs/{}.txt", 
        match get_date() {
            Ok(x) => x,
            Err(e) => {
                println!("Error getting the date: {}", e);
                return Err(format!("{}", e).to_string());

            }   
        } )){
        Ok(x) => {
            println!("Successfully creating the log file");
            return Ok(x)
        },
        Err(e) => {
            println!("Error in creating the log file\n{}", e);
            // panic!("Error in creating the log file: {}", e);
            return Err(format!("{}", e).to_string());
        }
    }
}

pub fn get_date() -> Result<String, String> {
    let datetime: DateTime<Local> = Local::now();

    let date = match memmem::find(&format!("{}", datetime).as_bytes()[..], &[0x20]/* (space) */ ).map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("failed to find the space, probably got corrupted", 3){
                Ok(x) => x,
                Err(e) => {
                    return Err(String::from(&format!("failed to log {}", e)));
                }
            };
            return Err(String::from("There was a problem getting your date"))
            // panic!("The date is fucked up");
        }
    };
    let placeholder = format!("{}", datetime);
    let date = String::from_utf8_lossy(&placeholder.as_bytes()[..date]);
    // println!("date: {:?}", date);
    Ok(date.to_string())
}

pub fn get_time() -> Result<String, String> {
    let datetime: DateTime<Local> = Local::now();
    let placeholder = format!("{}", datetime).into_bytes();

    let date = match memmem::find(&placeholder[..], &[0x20] ).map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("failed to find the space, probably got corrupted", 3){
                Ok(x) => x,
                Err(e) => {
                    return Err(String::from(&format!("failed to log {}", e)));
                }
            };
            return Err(String::from("There was an error getting your time"));
            // panic!("The time is fucked up");
        }
    };
    let end = match memmem::rfind(&placeholder[..], &[0x2E]).map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("failed to find the space, probably got corrupted", 3){
                Ok(x) => x,
                Err(e) => {
                    return Err(String::from(&format!("failed to log {}", e)));
                }
            };
            return Err(String::from("There was an error getting your time"));
            // panic!("The time is fucked up");
        }
    };
    let time = String::from_utf8_lossy(&placeholder[date + 1.. end]);
    // println!("time: {:?}", time);
    Ok(time.to_string())
}

pub fn parse_file<'a>(
    stream: &mut TcpStream,
    buffer: &'a mut Request,
    boundary: &[u8],
) -> Result<(Vec<u8>, String, String), &'static str> {
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
    
    let info = &buffer.body.clone().unwrap()[content_boundary + boundary.len()..];

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
    let content_type = match memmem::find(&buffer.body.clone().unwrap()[..], b"Content-Type:").map(|p| p as usize){
        Some(x) => x,
        None => {
            println!("We might have some trouble boss");
            send_error_response(stream, 500, "Why are you trying to break the server boss?");
            return Ok(((&[]).to_vec(), String::from(""), Default::default()));
        }
    };

    if let Some(_) = memmem::find(&buffer.header[..], b"name=\"folder\"").map(|p| p as usize) {
        let content_type = &buffer.body.clone().unwrap()[content_type + "Content-Type:\"".len()..];

        // println!("content-type is equal to IDFKK ={}\n\n\n\n", String::from_utf8_lossy(&content_type[..]));

        // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
        let end = match memmem::find(&content_type, b"\r\n\r\n")
            .map(|p| p as usize){
                Some(x) => x,
                None => {
                    println!("This is not ok ");
                    send_error_response(stream, 500, "This file or request is corrupted <br> stop it");
                    return Ok(((&[]).to_vec(), String::from(""), Default::default()));
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
                    return Ok(((&[]).to_vec(), String::from(""), Default::default()));
                }
            };
        let filename_data = &info[filename + "filename=".len()..];

        let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
        let filename_1 = match filename1.next() {
            Some(x) => x,
            None => {
                println!("Nope");
                send_error_response(stream, 400, "Did you play around before sending this file?");
                return Ok(((&[]).to_vec(), String::from(""), Default::default()));
            }
        };
        let filename_2 = match filename1.next() {
            Some(x) => x,
            None => {
                println!("Nope");
                send_error_response(stream, 400, "Did you play around before sending this file? <br>Are you sure about that?");
                return Ok(((&[]).to_vec(), String::from(""), Default::default()));
            }
        };
        let filename = &filename_data[filename_1 + 1..filename_2];
        //3
        // println!("filename = {:?}", String::from_utf8_lossy(&filename[..]));
        // let file = String::from_utf8_lossy(&filename[..]).to_string();

        // Decode the filename from bytes
        let file = decode_windows_1255(&filename[..]);

        // Decode HTML entities in the filename
        let file = decode_html(&file).unwrap_or_else(|_| file);

        return Ok((
            content.to_vec(),
            String::from_utf8(content_type.to_vec()).unwrap_or(String::from("application/octet-stream")),
            file.replace(" ", "_"),
        ));
    }
    let content_type = &buffer.body.clone().unwrap()[content_type + "Content-Type:\"".len()..];

    // println!("content-type is equal to ={}", String::from_utf8_lossy(&content_type[..]));

    // let end = memmem::find(&content_type, b"\r\n\r\n").map(|p| p as usize).unwrap();
    let end = match memmem::find(&content_type, b"\r\n\r\n")
        .map(|p| p as usize){
            Some(x) => x,
            None => {
                println!("Looks like someone played around a bit");
                send_error_response(stream, 400, "The file/request has probably been corrupted during transmission");
                return Ok(((&[]).to_vec(), String::from(""), Default::default()));
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
                return Ok(((&[]).to_vec(), String::from(""), Default::default()));
            }
        };
    let filename_data = &info[filename + "filename=".len()..];

    let mut filename1 = memmem::find_iter(filename_data, "\"").map(|p| p as usize);
    let filename_1 = match filename1.next() {
        Some(x) => x,
        None => {
            println!("Nope");
            send_error_response(stream, 400, "Did you play around before sending this file?");
            return Ok(((&[]).to_vec(), String::from(""), Default::default()));
        }
    };
    let filename_2 = match filename1.next() {
        Some(x) => x,
        None => {
            println!("Nope");
            send_error_response(stream, 400, "Did you play around before sending this file? <br>Are you sure about that?");
            return Ok(((&[]).to_vec(), String::from(""), Default::default()));
        }
    };
    let filename = &filename_data[filename_1 + 1..filename_2];

    //3
    // println!("Parse Upload filename = {:?}", String::from_utf8_lossy(&filename_data[..]));
    // println!("Parse Upload filename = {:?}", filename_data);
    // println!("Parse Upload filename = {:?}", decode_Windows_1255(&filename[..]));

    // upload filename =uploads/"What’s the craziest way you’ve seen someone get humbled_&#129300;.mp4"
    // Content-Type: video/mp4

    // println!("Parse Upload content = {:?}", String::from_utf8_lossy(&content[..]));

    Ok((
        content.to_vec(),
        String::from_utf8(content_type.to_vec()).unwrap_or(String::from("application/octet-stream")),
        decode_windows_1255(&filename[..]).replace(" ", "_"), //i think i should repplace this with encode_percent or smth so " " -> %20
    ))
}


//sorting functions
pub fn bubble_sort() -> Result<Vec<String>, String> {

    let user = match SHOW_FOLDER.lock(){
        Ok(x) => x.clone(),
        Err(e) => {
            println!("You are cooked my guy\n {}", e);
            // panic!("YOu are fucked (show folder mutex error, proobly poisoning)");
            match log(&format!("Error getting the user from Mutex: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    return Err(String::from(format!("failed to log {}", e)));
                }
            };
            return Err(String::from("You are cooked my guy, failed to get the user from Mutex"));
        }
    };

    let uploads = match fs::read_dir(format!("uploads/{}", user)){
        Ok(x) => x,
        Err(e) => {
            println!("Unnable to read the folder\n{}", e);
            match log(&format!("Error reading the uploads folder: {}", e), 3){
                Ok(x) => x,
                Err(_e) => {  
                    return Err(String::from("Error reading logging"));
                } 
            }
            return Err(String::from("Error reading the uploads folder"));
        }
    };

    let data = match fs::read_dir(format!("data/{}", user)){
        Ok(x) => x,
        Err(e) => {
            println!("Unnable to read the folder\n{}", e);
            match log(&format!("Error reading the data folder: {}", e), 3){
                Ok(x) => x,
                Err(_e) => {
                    return Err(String::from("Error reading logging"));
                } 
            }
            return Err(String::from("Error reading the data folder"));
        }
    };

    let mut sorting = vec![ String::from("maf"), String::from("nasfasfd"), String::from("basfafa"), 
                            String::from("vakkmf"), String::from("clfma"), String::from("x]ak"), String::from("zojw"), 
                            String::from("ykanf"), String::from("tklfn"), String::from("rklan"), String::from("eaklmf"), 
                            String::from("wljkafn"), String::from("qjdaklf"), String::from("lpamv"), String::from("kafjasn"), 
                            String::from("japfns"), String::from("hpansk"), String::from("ganc"), String::from("faspcn"), 
                            String::from("dkaas"), String::from("sjnas"), String::from("pasfjnl"), String::from("opojfen"), 
                            String::from("imvank"), String::from("ulknadl"), String::from("yknlav"), String::from("tlkdmav"), 
                            String::from("rlnd"), String::from("elkndm"), String::from("wijlkdm"), String::from("qdjaikfl")];

    let mut sorted: Vec<String> = Vec::new();
    
    loop{
        for i in 0..sorting.len() { //is this bubble sort?
            //start sorting logic here ig
            println!("sorting item: {:?}", sorting[i]);
            let item = sorting[i].clone().into_bytes();
            let mut breakpoint = false;
            for j in 0..item.len(){
                println!("byte: {}", item[j]);
                if i + 1 >= sorting.len(){
                    breakpoint = true;
                    break;
                };
                if j + 1 >= sorting[i + 1].len(){
                    breakpoint = true;
                    break;
                };
                let next_word = sorting[i + 1].as_bytes()[j];
                if item[j] > next_word{
                    if j >= item.len() - 1 && item.len() > 1{ //what if the name is one digit long????
                        println!("first {} is before the second {}", sorting[i], sorting[i + 1]);
                        match log(&format!("first {} is before the second {}", sorting[i], sorting[i + 1]), 0){
                            Ok(x) => x,
                            Err(e) => {
                                return Err(String::from(format!("failed to log {}", e)));
                            }
                        };
                        break;
                    } else {
                        println!("first {} is after the second {}", sorting[i], sorting[i + 1]);
                        match log(&format!("first {} is after the second {}", sorting[i], sorting[i + 1]), 0){
                            Ok(x) => x,
                            Err(e) => {
                                return Err(String::from(format!("failed to log {}", e)));
                            }
                        };
                        let temp1 = sorting[i].clone(); 
                        let temp2 = sorting[i + 1].clone();
                        sorting[i] = temp2;
                        sorting[i + 1] = temp1;
                    }
                    break;
                } else {
                    break;
                }
            }
        }
        for i in 0..sorting.len(){
            println!("checking");
            match log("checking", 0){
                Ok(x) => x,
                Err(e) => {
                    return Err(String::from(format!("failed to log {}", e)));
                }
            };
            if sorting[i] > sorting[i + 1]{
                break;
            }
            if i == sorting.len() - 2{
                println!("sorted successfully");
                match log("sorted successfully", 0){
                    Ok(x) => x,
                    Err(e) => {
                        return Err(String::from(format!("failed to log {}", e)));
                    }
                };
                sorted = sorting.clone();
                println!("sorted: {:?}", sorting); //could do better???
                match log(&format!("sorted: {:?}", sorting), 0){
                    Ok(x) => x,
                    Err(e) => {
                        return Err(String::from(format!("failed to log {}", e)));
                    }
                };
                return Ok(sorting)
            }
        }
        println!("sorted: {:?}", sorting); //could do better???
    }
    Ok(Vec::new())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Files {
    list: Vec<FileNames>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileNames{
    pub datapath: String,   //augu/sigma/1003-19-129.txt.txt
    pub dataname: String,   // 1003-19-129.txt.txt
    pub uploadspath: String,    //augu/sigma/1003-19-129.txt
    pub uploadsname: String,    //1003-19-129.txt
    pub realname: String,   // text.txt
    pub file_type: String,
    pub f_type: String,
    pub folder: String,
    pub date: String,
    pub time: String,
    pub is_file: bool,

    //more to come like date time, file type sort and size
}

impl FileNames {

    pub fn new() -> Self {
        FileNames{
            datapath: String::from(""),
            dataname: String::from(""),
            uploadspath: String::from(""),
            uploadsname: String::from(""),
            realname: String::from(""),
            folder: String::from(""),
            file_type: String::from(""),
            f_type: String::from(""),
            date: String::from(""),
            time: String::from(""),
            is_file: false,
        }
    }
    /* 
    pub fn ins_disk(&mut self, data: String) -> Self {
        FileNames{
            diskname: data,
            realname: self.realname.clone(),
            date: self.date.clone(),
            time: self.time.clone(),
            is_file: self.is_file.clone(),
        }
    }
    pub fn ins_real(&mut self, data: String) -> Self {
        FileNames{
            diskname: self.diskname.clone(),
            realname: data,
            date: self.date.clone(),
            time: self.time.clone(),
            is_file: self.is_file.clone(),
        }
    }
    pub fn ins_date(&mut self, data: String) -> Self {
        FileNames{
            diskname: self.diskname.clone(),
            realname: self.realname.clone(),
            date: data,
            time: self.time.clone(),
            is_file: self.is_file.clone(),
        }
    }
    pub fn ins_time(&mut self, data: String) -> Self {
        FileNames {
            diskname: self.diskname.clone(),
            realname: self.realname.clone(),
            date: self.date.clone(),
            time: data,
            is_file: self.is_file.clone(),

        }
    }
    pub fn ins_is_file(&mut self, data: bool) -> Self {
        FileNames {
            diskname: self.diskname.clone(),
            realname: self.realname.clone(),
            date: self.date.clone(),
            time: self.time.clone(),
            is_file: data,
        }
    }
    */
}

pub fn sorting(how: u8, user: String) -> Result<Vec<FileNames>, String> {
    let data = match fs::read_dir(format!("data/{}", user)){
        Ok(x) => x,
        Err(e) => {
            println!("Unnable to read the folder\n{}", e);
            match log(&format!("Error reading the data folder: {}\n While sorting duh", e), 3){
                Ok(x) => x,
                Err(_e) => {
                    return Err(String::from("Error reading logging"));
                } 
            }
            return Err(String::from("Error reading the data folder"));
        }
    };

    let uploads = match fs::read_dir(format!("uploads/{}", user)){
        Ok(x) => x,
        Err(e) => {
            println!("Unnable to read the folder\n{}", e);
            match log(&format!("Error reading the uploads folder: {}\n While sorting duh", e), 3){
                Ok(x) => x,
                Err(_e) => {
                    return Err(String::from("Error reading logging"));
                } 
            }
            return Err(String::from("Error reading the uploads folder"));
        }
    };

    let mut folders = Vec::new();
    let mut names = Vec::new();

    let mut i = 0;
    for entry in data {
        let entry = match entry{
            
            Ok(x) => x,
            Err(e) => {
                println!("No users data found\n{:?}", e);
                match log(&format!("Error in finding data files: {}", e), 1){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from(""));
                    } 
                }
                return Err(String::from(""));
            }
        };

        println!("Entry: {:?}", entry);
        
        let file_name = match entry.file_name().into_string(){
            Ok(x) => x,
            Err(e) => {
                println!("The user's username is unnable to be converted to string\n{:?}", e);
                match log(&format!("Error converting the filename in a UTF-8 format: {:?}", e), 1){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from(""));
                    } 
                }
                return Err(String::from(""));
            }
        };

        println!("name: {:?}", file_name);

        //if able to open file then search to get the file name

        let mut upld_path = String::from(&format!("uploads/{}/{}", user, file_name)); //for folders
        let data = match fs::read(format!("data/{}/{}", user, file_name)){
            Ok(x) => x,
            Err(e) => {
                println!("The user's data file cannot be read\n{:?}", e);
                match log(&format!("The user's data file cannot be read\n{:?}", e), 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }

                let fold = String::from_utf8_lossy(&upld_path.as_bytes()[..upld_path.len() - (file_name.len() + 1)]);
                folders.push(FileNames{
                    datapath: entry.path().display().to_string().replace("\\", "/"),
                    dataname: file_name.clone(),
                    uploadspath: upld_path.to_string(),
                    uploadsname: file_name.to_string(),
                    realname: file_name.clone(),
                    folder: fold.to_string(),
                    file_type: String::from(""),
                    f_type: String::from(""),
                    date: String::from(""),
                    time: String::from(""),
                    is_file: false,
                });


                continue;
                // vec![0u8; 0]
            }
        };

        
        let end = match memmem::rfind(&file_name.as_bytes()[..], b".txt").map(|p| p as usize){
            Some(x) => x,
            None => {
                println!("oke this shit is impossible");
                match log("There was a mixup with where the files are supposed to go ig", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                return Err(String::from("The wrong file format has been found"));
            }
        };
        
        let upld_name = String::from_utf8_lossy(&file_name.as_bytes()[..end]);
        upld_path = String::from(&format!("uploads/{}/{}", user, upld_name));
        println!("attempted upload path: {}", upld_path);


        let (name, f_type, date, time) = match get_data_info(data){
            Ok(x) => x,
            Err(e) => {
                println!("{:?}", e);
                match log(&format!("{:?}", e), 1){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from(""));
                    } 
                }
                return Err(String::from(""));
            }
        }; 

        let mut file_type = String::from("");
        if IMAGE_TYPES.contains(&&*f_type) {
            file_type = String::from("image");
        } else if VIDEO_TYPES.contains(&&*f_type) {
            file_type = String::from("video");
        } else if TEXT_TYPES.contains(&&*f_type) {
            file_type = String::from("text");
        }





        let fold = String::from_utf8_lossy(&upld_path.as_bytes()[..upld_path.len() - (file_name.len() + 1)]);
        names.push(FileNames{
            datapath: entry.path().display().to_string().replace("\\", "/"),
            dataname: file_name.clone(),
            uploadspath: upld_path.to_string(),
            uploadsname: upld_name.to_string(),
            realname: name.to_string(),
            folder: fold.to_string(),
            file_type : file_type.to_string(),
            f_type : f_type.to_string(),
            date: date.to_string(),
            time: time.to_string(),
            is_file: true,
        })

    }

    let sorted = match how {
        0 => { //alphabetical order
            let sorted_folders = sort_fn(folders.clone());
            let sorted_files = sort_fn(names.clone());

            let mut sorted1 = Vec::new();
            
            for i in sorted_folders{
                sorted1.push(i);
            } 
            for i in sorted_files{
                sorted1.push(i);
            }
            sorted1
        },
        1 => { //reverse upload time order 

            let sorted_folders = sort_fn(folders.clone());
            let sorted_files = sort_fn_date(names.clone());

            println!("\n\nnames: {:?}", names);
            let mut sorted1 = Vec::new();
            
            for i in (0..sorted_files.len()).rev(){
                sorted1.push(sorted_files[i].clone());
            }
            for i in sorted_folders{
                sorted1.push(i);
            } 
            println!("\n\n\nsorted ig...{:#?}", sorted1);

            sorted1
        },
        4 => {
            let sorted_folders = sort_fn(folders.clone());
            let sorted_files = sort_fn(names.clone());

            let mut sorted1 = Vec::new();
            
            for i in (0..sorted_folders.len()).rev(){
                sorted1.push(sorted_folders[i].clone());
            } 
            for i in sorted_files{
                sorted1.push(i);
            }
            sorted1
        },
        5 => { //reverse upload time order 

            let sorted_folders = sort_fn(folders.clone());
            let sorted_files = sort_fn_date(names.clone());

            println!("\n\nnames: {:?}", names);
            let mut sorted1 = Vec::new();
            
            for i in sorted_files{
                sorted1.push(i);
            }
            for i in sorted_folders{
                sorted1.push(i);
            } 
            println!("\n\n\nsorted ig...{:#?}", sorted1);

            sorted1
        },
        _=> {
            //the default is in alphabetical order
            let sorted_folders = sort_fn(folders.clone());
            let sorted_files = sort_fn(names.clone());

            let mut sorted1 = Vec::new();
            
            for i in (0..sorted_files.len()).rev(){
                sorted1.push(sorted_files[i].clone());
            }
            for i in sorted_files{
                sorted1.push(i);
            }
            sorted1

        }   
    };
    

    // println!("\n\nsorted upload time: {:?}", sorted); //could do better???

    Ok(sorted)
}

fn sort_fn(list: Vec<FileNames>) -> Vec<FileNames> {

    let mut lower_list = Vec::new();
    let mut lower_sorted = Vec::new();
    let mut sorted = Vec::new();

    for i in 0..list.len() {
        lower_list.push(FileNames{
            datapath: list[i].datapath.clone(),
            dataname: list[i].dataname.clone(),
            uploadspath: list[i].uploadspath.clone(),
            uploadsname: list[i].uploadsname.clone(),
            realname: list[i].realname.to_lowercase(),
            folder: list[i].folder.clone(),
            file_type: list[i].file_type.clone(),
            f_type: list[i].f_type.clone(),
            date: list[i].date.clone(),
            time: list[i].time.clone(),
            is_file: list[i].is_file.clone()
        })
    }

    let mut breakpoint = false;

    for i in 0..list.len() {
        let key = list[i].realname.as_bytes();

        println!("\nkey: {:?}", list[i]);
        // println!("len: {:?}", sorted.len());
        
        if sorted.len() == 0 {
            lower_sorted.push(lower_list[i].clone()); 
            sorted.push(list[i].clone());
            println!("key index first word byte: {:?}", key[0]);
            println!("inserted first word");
            continue;
        }

        breakpoint = false;
        println!("checking ");

        for j in (0..sorted.len()).rev(){
            // println!("j: {}", j);

            // println!("How is {:?} compared to {:?}", key[0], sorted[j].as_bytes()[0]);
            // println!("How is {:?} compared to {:?}",String::from_utf8_lossy(&[key[0]]), String::from_utf8_lossy(&[sorted[j].as_bytes()[0]]));

            if key == lower_sorted[j].realname.as_bytes() {
                lower_sorted.insert(j, lower_list[i].clone());
                sorted.insert(j, list[i].clone());
                break;
            }

            if key[0] >= lower_sorted[j].realname.as_bytes()[0] {

                for index in 0..key.len(){

                    println!("key index byte: {:?}", key[index]);
                    if index >= lower_sorted[j].realname.len() {
                        lower_sorted.insert(j, lower_list[i].clone());
                        sorted.insert(j, list[i].clone());
                        breakpoint = true;
                        break;
                    }

                    if key[index] < lower_sorted[j].realname.as_bytes()[index]{
                        lower_sorted.insert(j, lower_list[i].clone());
                        sorted.insert(j, list[i].clone());
                        breakpoint = true;
                        break;
                    }

                    if key[index] > lower_sorted[j].realname.as_bytes()[index]{
                        lower_sorted.insert(j + 1, lower_list[i].clone());
                        sorted.insert(j + 1, list[i].clone());
                        breakpoint = true;
                        break;
                    }

                }
            } else if j == 0 && key[0] < lower_sorted[j].realname.as_bytes()[0]{
                lower_sorted.insert(j, lower_list[i].clone());
                sorted.insert(j, list[i].clone());
                println!("key index byte: {:?}", list[i].realname.as_bytes()[0]);
                break;
            } 

            // println!("not sorted: {:?}", sorted); //could do better???

            if breakpoint {
                break;
            }
        }
    }

    sorted
}

fn sort_fn_date(list: Vec<FileNames>) -> Vec<FileNames> {
    // println!("SORTING AFTER UPLOAD TIME");
    let mut sorted = Vec::new();

    let mut breakpoint = false;

    for i in 0..list.len() {
        let date = list[i].date.as_bytes();

        // println!("\nkey: {:?}", list[i]);
        // println!("len: {:?}", sorted.len());
        
        if sorted.len() == 0 {
            sorted.push(list[i].clone());
            // println!("key index first word byte: {:?}", date[0]);
            // println!("inserted first word");
            continue;
        }

        breakpoint = false;
        // println!("checking ");

        for j in (0..sorted.len()).rev(){
            // println!("checking again");
            // println!("j: {}", j);

            // println!("How is {:?} compared to {:?}", key[0], sorted[j].as_bytes()[0]);
            // println!("How is {:?} compared to {:?}",String::from_utf8_lossy(&[key[0]]), String::from_utf8_lossy(&[sorted[j].as_bytes()[0]]));

            // if date == sorted[j].date.as_bytes() {
            //     sorted.insert(j, list[i].clone());
            //     break;
            // }

            if date >= sorted[j].date.as_bytes() {

                for index in 0..date.len(){
                    let mut breakpoint2 = false;

                    // println!("date index byte: {:?}", date[index]);

                    // debug this shit
                    //doesnt sort as it should and idk why
                    if date == sorted[j].date.as_bytes() {
                        /* if list[i].time == sorted[j].time { //it's impossible this so imma leave it commented for now(i dont want another for loop in here)

                        } */

                        for l in 0..list[i].time.len(){
                            if list[i].time.as_bytes()[l] < sorted[j].time.as_bytes()[l]{
                                // println!("{} is smaller than {}", list[i].time, sorted[j].time);
                                // println!("the digit that made the difference between {} and {} is {} < {}", list[i].time, sorted[j].time, list[i].time.as_bytes()[l], sorted[j].time.as_bytes()[l]);
                                sorted.insert(j, list[i].clone());
                                breakpoint2 = true;
                                breakpoint = true;
                                break;
                            }

                            if list[i].time.as_bytes()[l] > sorted[j].time.as_bytes()[l]{
                                // println!("{} is bigger than {}", list[i].time, sorted[j].time);
                                // println!("the digit that made the difference between {} and {} is {} > {}", list[i].time, sorted[j].time, list[i].time.as_bytes()[l], sorted[j].time.as_bytes()[l]);
                                sorted.insert(j + 1, list[i].clone());
                                breakpoint2 = true;
                                breakpoint = true;
                                break;
                            }
                        }
                    }

                    if date[index] < sorted[j].date.as_bytes()[index]{
                        // println!("\n\nIs the date smaller than the stored date");
                        sorted.insert(j, list[i].clone());
                        breakpoint = true;
                        break;
                    }

                    if date[index] > sorted[j].date.as_bytes()[index]{
                        // println!("\n\nIs the date bigger than the stored date");
                        sorted.insert(j + 1, list[i].clone());
                        breakpoint = true;
                        break;
                    }

                    if breakpoint2 == true {
                        break;
                    }

                }
            } else if j == 0 && date < sorted[j].date.as_bytes(){
                // println!("\n\nIs j = 0 and sorted bigger than the uploaded date, HUH?");
                sorted.insert(j, list[i].clone());
                // println!("key index byte: {:?}", list[i].date.as_bytes()[0]);
                break;
            } 

            // println!("not sorted: {:?}", sorted); //could do better???

            if breakpoint {
                break;
            }
        }
    }

    sorted
}

fn get_data_info(data:Vec<u8>) -> Result<(String, String, String, String), String> {
        let end = match memmem::find(&data[..], b";").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };

        let file_type = &data["Content-Type:".len()..end];
    
        let start = match memmem::find(&data[..], b"file_name:\"").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };

        let data = &data[start + "file_name:\"".len()..];
        let end = match memmem::find(&data[..], b"\"").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };
        let name = &data[.. end ];

        let start = match memmem::find(&data[..], b"date:\"").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };

        let data = &data[start + "date:\"".len()..];
        let end = match memmem::find(&data[..], b"\"").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };
        let date = &data[.. end];

        let start = match memmem::find(&data[..], b"time:\"").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };

        let data = &data[start + "time:\"".len()..];
        let end = match memmem::find(&data[..], b"\"").map(|p| p as usize) {
            Some(x) => x,
            None => {
                println!("The user's data file probably got corrupted");
                match log("The user's data file probably got corrupted", 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return Err(String::from("Error logging"));
                    } 
                }
                // return Err(String::from("Error reading the data file"));
                return Err(String::from("The user's data file probably got corrupted"));
            }
        };
        let time = &data[.. end];

        return Ok((
            String::from_utf8_lossy(&name[..]).to_string(),
            String::from_utf8_lossy(&file_type[..]).to_string(),
            String::from_utf8_lossy(&date[..]).to_string(),
            String::from_utf8_lossy(&time[..]).to_string(),
        ))

}

pub fn get_path_fron_direntry(dir: PathBuf) -> Result<PathBuf, String> {
    //diskname: "DirEntry(\"data/augu/77e05d75-cbd0-4787-8a6a-abecb3fbca12.png\")"

    //<button onclick="window.location.href='/open_folder/DirEntry(" data="" augu="" 1cbe97fc-74fb-4cb2-987b-e018f636be41.png")'"="">Open folder</button>

    let Str = format!("{:?}", dir);
    println!("dir in string: {}", Str); //dir in string: "DirEntry(\"data/augu/53bbf5da-1a21-40ba-a343-9903fdafe0b5.png\")"
    let start = match memmem::find(&Str.as_bytes()[..], b"(\\\"").map(|p| p as usize) {
        Some(x) => x,
        None => {
            println!("oh well ayaye, deal with it");
            match log("There was an error getting the start of the file path", 3){
                Ok(x) => x,
                Err(_e) => {
                    // send_error_response(&mut stream, 400, &e);   
                    return Err(String::from(""));
                } 
            }
            // panic!("I'm tired of dealing with random error")
            return Err(String::from("couldn't find the start of the path"));
        }
    };

    let end = match memmem::find(&Str.as_bytes()[..], b"\\\")").map(|p| p as usize) {
        Some(x) => x,
        None => {
            println!("oh well ayaye, deal with it");
            match log("There was an error getting the end of the file path", 3){
                Ok(x) => x,
                Err(_e) => {
                    // send_error_response(&mut stream, 400, &e);   
                    return Err(String::from(""));
                } 
            }
            // panic!("I'm tired of dealing with random error")
            return Err(String::from("couldn't find the end of the path"));
        }
    };

    let path = &Str[start + "(\\\"".len()..end];

    Ok(PathBuf::from(path))
}

pub fn give_files(mut stream: TcpStream, buffer: Request){
    //{"order":"0"}

    let order = match memmem::find(&buffer.body.clone().unwrap(), b"{\"order\":\"").map(|p| p as usize) {
        Some(x) => x,
        None => {
            println!("impossible to not find");
            send_error_response(&mut stream, 400, "Wasnt able to find the order to sort the files");
            return;
        }
    };

    let order = &buffer.body.clone().unwrap()[order + "{\"order\":\"".len()..buffer.body.clone().unwrap().len() - 2];
    println!("order :{}", String::from_utf8_lossy(&order[..]));

    let order = String::from_utf8_lossy(&order[..]).parse::<u8>().unwrap();
    println!("order :{}", order);


    let user = checkAuth(&mut stream, buffer.clone());
    let folder = checkFolder(&mut stream, buffer.clone());

    if user == ""{
        println!("user: {}", user);
        println!("there was a problem getting the auth key");
        send_error_response(&mut stream, 404, "There was a problem getting the auth key");
        return;
    }
    let path = format!("{}/{}", user, folder);  

    let sorted =match sorting(order, path){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &format!("There was an error sorting the files {}", e));
                return;
            }
    };

    println!("sorted: {:#?}", sorted); // didtn return shit that's why
    // let s = &format!("{:?}", sorted);
    let json = serde_json::to_string(&sorted).unwrap();

    println!("json = {}", json);

    let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                json.len(),
                json
            );

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

pub fn checkAuth(stream: &mut TcpStream, buffer: Request) -> String {
    // Cookie: Auth="user-augu-token"; Folder="folder--token"
    let body = buffer.header;
    let start = match memmem::find(&body[..], b"Auth=\"user-").map(|p| p as usize){
        Some(x) => x,
        None => {
            // send_error_response(stream, 404, "Unnable to get auth key");
            return String::from("");
        }
    };
    let body = &body[start + "Auth=\"user-".len()..];
    let end = match memmem::find(&body[..], b"-token").map(|p| p as usize){
        Some(x) => x,
        None => {
            // send_error_response(stream, 404, "Unnable to get auth key");
            return String::from("");
        }
    };

    return String::from_utf8_lossy(&body[..end]).to_string();
}

pub fn checkFolder(stream: &mut TcpStream, buffer: Request) -> String {
let body = buffer.header;
    let start = match memmem::find(&body[..], b"Folder=\"folder-").map(|p| p as usize){
        Some(x) => x,
        None => {
            // send_error_response(stream, 404, "Unnable to get auth key");
            return String::from("");
        }
    };
    let body = &body[start + "Folder=\"folder-".len()..];
    let end = match memmem::find(&body[..], b"-token").map(|p| p as usize){
        Some(x) => x,
        None => {
            // send_error_response(stream, 404, "Unnable to get auth key");
            return String::from("");
        }
    };

    return String::from_utf8_lossy(&body[..end]).to_string();
}
//more to come
pub const MAX_UPLOAD_SIZE: usize = 100 * 1024 * 1024; // 100MB

pub const ALLOWED_MIME_TYPES: &[&str] = &[
    "audio/wav",
    "audio/mp3",
    "application/x-zip-compressed",
    "video/mp4",
    "text/plain",
    "image/jpeg",
    "image/png",
    "image/gif",
    "application/pdf",
    "application/octet-stream",
    "multipart/form-data",
]; 

pub const IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/gif",
    "image/svg+xml",
    "image/webp",
    "image/tiff",
    "image/bmp",
];

pub const VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/x-msvideo",
    "video/mpeg",
    "video/ogg"
];

pub const AUDIO_TYPES: &[&str] = &[
    "audio/mpeg",
    "audio/wav",
    "audio/mp4",
    "audio/ogg",
    "audio/opus",
    "audio/aac",
    "audio/webm",
];

pub const TEXT_TYPES: &[&str] = &[
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
