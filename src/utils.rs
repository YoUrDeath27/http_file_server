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
pub fn sort_name() -> Result<Vec<String>, String> {

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


pub fn insert_sort() -> Result<Vec<String>, String> {

    let mut sorting = vec![ String::from("nasfasfd"), String::from("maf"), String::from("basfafa"), String::from("vakkmf")
                            // , String::from("clfma"), String::from("x]ak"), String::from("zojw"), 
                            // String::from("ykanf"), String::from("tklfn"), String::from("rklan"), String::from("eaklmf"), 
                            // String::from("wljkafn"), String::from("qjdaklf"), String::from("lpamv"), String::from("kafjasn"), 
                            // String::from("japfns"), String::from("hpansk"), String::from("ganc"), String::from("faspcn"), 
                            // String::from("dkaas"), String::from("sjnas"), String::from("pasfjnl"), String::from("opojfen"), 
                            // String::from("imvank"), String::from("ulknadl"), String::from("yknlav"), String::from("tlkdmav"), 
                            // String::from("rlnd"), String::from("elkndm"), String::from("wijlkdm"), String::from("qdjaikfl")
                    ];

    let mut sorted: Vec<String> = Vec::new();
    println!("len: {:?}", sorted.len());
    let mut breakpoint = false;
 
    // deci verifica fiecare element din lista ordonata si daca e mai mare o adauga dupa
    // altfel trece la urmatorul pana la final
    // si la inceput daca cuvantul e mai mic decat primul faci un edge case ca sa il adaugi inainte

    //i did it wrong, just start from the end and go backwards as long the index of the words is smaller than the next(backwards mode) one
    //implement in the morning of 14
    for i in 0..sorting.len() {
        let key = sorting[i].as_bytes();
        println!("\nkey: {:?}", sorting[i]);
        println!("len: {:?}", sorted.len());
        
        if sorted.len() == 0 {
            sorted.push(sorting[i].clone());
            println!("inserted first word");
            continue;
        }

        breakpoint = false;

        for j in sorted.len()..0 {
            println!("j: {}", j);

            // if key[0] <
            
            // if j + 1 >= sorted.len() && key[0] < sorted[1].as_bytes()[0] && key[0] > sorted[0].as_bytes()[0] { //sorted = {"a", "c"}     "b"
            //     sorted.push(sorting[i].clone());
            //     break;
            // } 
            
            for index in 0..key.len() {
                println!("index: {}", index);
                if (key.len() < sorted[j].len()) && (key[index] < sorted[j].as_bytes()[index]) {
                    // print!("s  ");
                    sorted.insert(j, sorting[i].clone());
                    println!("stopped because small");
                    breakpoint = true;
                    break;
                }

                if j == 0 && key[index] < sorted[j].as_bytes()[index] { //if begining is smaller but also higher in the hierarchy
                    sorted.insert(j, sorting[i].clone());
                    breakpoint = true;
                    break;
                }

                if key[index] > sorted[j].as_bytes()[index] {
                    sorted.insert(j + 1, sorting[i].clone());
                    break;
                }


                // if index > sorted[j].len() { //cuvantul pe care il verificam e mai mare decat cel care este deja inauntru
                //     sorted.insert(j, sorting[i].clone());
                //     breakpoint = true;
                //     break;
                // }

                // if key[index] > sorted[j].as_bytes()[index] {
                //     sorted.insert(j + 1, sorting[i].clone());
                //     breakpoint = true;
                //     break;
                // }
                // if key[index] < sorted[j].as_bytes()[index]{
                //     sorted.insert(j, sorting[i].clone());
                //     breakpoint = true;
                //     break;
                // }

            }
            //an=t the end after inserting the item

            println!("not sorted: {:?}", sorted); //could do better???

            if breakpoint {
                break;
            }
        }
    }

    println!("sorted: {:?}", sorted); //could do better???
    println!("len sorted: {}", sorted.len());
    println!("len unsorted: {}", sorting.len());
    Ok(sorted)
        
    

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
