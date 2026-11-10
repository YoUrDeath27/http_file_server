use super::*;
use bcrypt::{DEFAULT_COST, hash, verify};

pub fn auth_user(mut stream: TcpStream, buffer: Request) {
    let name =  match memmem::find(&buffer.body.clone().unwrap()[..], b"**account=").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("the user tried to connect while it did not met the requirements to log in", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "-How did you find me?<br>-GPS tapped on your FUCKING boat");
            println!("Dexter reference activated 1"); 
            return;
        }
    };

    let status_line = "HTTP/1.1 200 OK\r\n";
    let mut response = String::from(""); //dont ask homie

    let user = match memmem::find(&buffer.body.clone().unwrap()[..], b"**account").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The user tried to log in while not meeting the requirements", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                    } 
            }
            send_error_response(&mut stream, 510, "-Holy fucking shit, You are thvere bay harbout butcher<br>-I never liked that nickname");
            println!("Dexter reference activated 2");
            return;
        }
    };
    // println!("{:?}", String::from_utf8_lossy(&buffer.body.clone().unwrap()[..]));
    let user = &buffer.body.clone().unwrap()[user + "**account=".len()..];
    
    let mut text = Vec::new(); //useless now
    {
        let mut file = match fs::File::open("users.txt") {
            Ok(c) => c,
            Err(_) => match fs::File::create_new("users.txt"){
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("Error with creating again the user file: {}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
                    println!("failed to create the ursers \"database\" {}", e);
                    return Default::default();
                },
            },
        };

        match file.read_to_end(&mut text)  {
            Ok(x) => x,
            Err(e) => {
                match log(&format!("{}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);
                        return;
                    }
                }
                send_error_response(&mut stream, 400, "Failed to read the users file");
                return;    
            }
        };
    }
    
    let users1 = match fs::read_dir(format!("users/")) {
        Ok(x) => x,
        Err(e) => {
            println!("Something went wrong with opening the users: \n{}", e);
            match log(&format!("Error reading the users: {}", e), 3){
                Ok(x) => x,
                Err(e) => {  
                    // send_error_response(&mut stream, 400, &e);
                    println!("Failed to log");
                    return;
                } 
            }
            // send_error_response(&mut stream, 400, &format!("{:?}", e));
            return;
        }
    };

    let mut list_users: Vec<String> = vec![];
    for entry in users1 {
        println!("folder: {:#?}", entry);
        let path = &format!("{:?}", entry.unwrap().path());
        let string = &path.as_bytes()["\"files/".len() .. path.len() - 1];
        println!("{}", String::from_utf8_lossy(&string[..]));
        list_users.push(String::from_utf8_lossy(&string[..]).to_string());
    }

    println!("list: {:?}", list_users);

    if response == "" {
        println!("This shit is finger licking good mate");
    }
    for l_user in list_users{
        if  l_user == String::from_utf8_lossy(&user[..]){
            // println!("trying to find the bug in here");
            // println!("{:?}", memmem::find(&text[..], &user[..]));
            let body = password(String::from_utf8_lossy(&user[..]).to_string(), None);
            response = format!("{}Content-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}", status_line, body.len(), body);//dont ask homie
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
    }
    // println!("l_user = {}", l_user);
    // println!("user = {}", String::from_utf8_lossy(&user[..]));

    let body = new_user(String::from_utf8_lossy(&user[..]).to_string());
    response = format!("{}Content-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}", status_line, body.len(), body);//dont ask homie
    

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

pub fn auth_pass_existing(mut stream: TcpStream, buffer: Request) {
    println!("Ur cooked {:?}", String::from_utf8_lossy(&buffer.body.clone().unwrap()[..]));
    println!("Not giving a single fuck for now abt the rest of info");
    
    let body = buffer.body.clone().unwrap();
    let mut user;
    let mut pass;
    // let mut gender;

    let user1 = match memmem::find(&buffer.body.clone().unwrap()[..], b"**account=").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The user tried to log in while not meeting the requirements", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "-Holy fucking shit, You are thvere bay harbout butcher<br>-I never liked that nickname");
            println!("Dexter reference activated 2");
            return;
        }
    };
    let body = &body[user1 + "**account=".len()..];

    let end = match memmem::find(&body[..], b"&").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };
    user = &body[..end];

    let pass1 = match memmem::find(&body[..], b"**password=").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The request probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "You gotta be jorking with me(existig)");
            println!("Bruh");
            return;
        }
    };
    pass = &body[pass1 + "**password=".len()..];
    
    let user = String::from_utf8_lossy(&user[..]);
    let pass = String::from_utf8_lossy(&pass[..]);

    let mut text = Vec::new(); //get the users list (i will have to change this)
    {
        let mut file = match fs::File::open(format!("users/{}/info.txt", user)) {
            Ok(c) => c,
            Err(_) => match fs::File::create_new(format!("users/{}/info.txt", user)){
                Ok(x) => x,
                Err(e) => {
                    match log(&format!("Error with creating again the user file: {}", e), 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
                    println!("failed to create the ursers \"database\" {}", e);
                    return Default::default();
                },
            },
        };

        match file.read_to_end(&mut text)  {
            Ok(x) => x,
            Err(e) => {
                match log(&format!("{}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);
                        return;
                    }
                }
                send_error_response(&mut stream, 400, "Failed to read the users file");
                return;    
            }
        };
    }

    let end = match memmem::find(&text[..], b"***").map(|p| p as usize) {
        Some(x) => x,
        None => {
            println!("The users stored password couldn't been identified");
            send_error_response(&mut stream, 404, "THe user's info probably got corrupted");
            return;
        }
    };





    let username = user; //username
    let stored_pass = String::from_utf8_lossy(&text["pass: ".len() .. end]); //hashed pass
    let unhashed_pass = pass; 

    println!("unhashed: {}", unhashed_pass);
    println!("hashed: {}", stored_pass);

    //if the password doesnt match with the username
    if !(match verify(&*unhashed_pass, &stored_pass){ //make this a smaller function trust me // but idk how
        Ok(x) => x,
        Err(e) => {
            match log(&format!("Error verifying the hash: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            eprintln!("Failed to verify password: {}\n\n", e);
            //instead of telling the user the function didn't worked, we simply tell it's not correct
            false
    }
    }) {
        match log("The user did not get the password right", 1){
        Ok(x) => x,
        Err(e) => {
            send_error_response(&mut stream, 400, &e);   
        } 
    }
        incorrect_pass(&mut stream, &username);
        return;
    } 
    //IF THEY MATCH 
    {
        let mut attempts = match USERS_ATTEMPTS.lock(){
            Ok(x) => x,
            Err(e) => {
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
        let (count, _locked_until) = attempts.entry(username.to_string()).or_insert((0, None));
        *count = 0;
    }

       
    //do some shit
      //add user with pass
         
    //if the user and pass match show the corresponding 




    let status_line = "HTTP/1.1 200 OK\r\n";
    let site = web(&mut stream, buffer);

    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        println!("site: {}", site);
        return;
    }

    let response = format!("{}Set-Cookie: Auth=\"user-{}-token\"; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\nSet-Cookie: Folder=\"folder--token\"; Path=/; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, username, site);

    // println!("\n\n\n\n\nresponse = \n{}", response);
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

pub fn auth_pass_not_existing(mut stream: TcpStream, buffer: Request) {
    //FUCKING CHECK IF THIS IS CORRECT NOT IN THE MOOD RN (01.03.2026)

    // println!("Ur cooked {:?}", String::from_utf8_lossy(&buffer.body.clone().unwrap()[..]));
    // println!("Not giving a single fuck for now abt the rest of info");
    
    let body = buffer.body.clone().unwrap();
    let mut user;
    let mut pass;
    let mut gender;

    let user1 = match memmem::find(&buffer.body.clone().unwrap()[..], b"**account").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The user tried to log in while not meeting the requirements", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "-Holy fucking shit, You are thvere bay harbout butcher<br>-I never liked that nickname");
            println!("Dexter reference activated 2");
            return;
        }
    };
    let body = &body[user1 + "**account\"".len() + "\r\n\r\n".len() ..];

    let end = match memmem::find(&body[..], b"\r\n").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };
    user = &body[..end];

    let pass1 = match memmem::find(&body[..], b"**password\"").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The request probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "You gotta be jorking with me (not existing)");
            println!("Bruh");
            return;
        }
    };
    let body = &body[pass1 + "**password\"".len() + "\r\n\r\n".len() ..];

    let end = match memmem::find(&body[..], b"\r\n").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };
    pass = &body[..end];

    let start = match memmem::find(&body[..], b"gender").map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };
    let body = &body[start + "gender\"\r\n\r\n".len() ..];

    let end = match memmem::find(&body[..], b"\r\n").map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };

    gender = &body[..end];

    let boundary = match get_boundary(&buffer.header.clone()){
        Some(x) => x,
        None => {
            println!("fucking hell");
            send_error_response(&mut stream, 404, "The request got corrupted");
            return;
        }
    };

    let start = match memmem::find(&body[..], &boundary).map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };

    let body = &body[start + boundary.len()..];

    let start = match memmem::find(&body[..], b"Content-Type: ").map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };

    let body = &body[start..];

    let start = match memmem::find(&body[..], b"\r\n\r\n").map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };

    let img_data = &body[..start];  

    let body = &body[start + "\r\n\r\n".len()..];

    let end = match memmem::find(&body[..], &boundary).map(|p| p as usize) {
        Some(x) => x,
        None => {
            match log("The user token probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };
    let img = &body[..end];

    
    let user = String::from_utf8_lossy(&user[..]);
    let pass = String::from_utf8_lossy(&pass[..]);
    let gender = String::from_utf8_lossy(&gender[..]);

    println!("user: {}", user);
    println!("pass: {}", pass);
    println!("gender: {}", gender);

    match fs::create_dir_all(format!("users/{}", user)){
        Ok(x) => x,
        Err(e) => println!("Failed to create the users directory\n{}", e) ,
    }

    // pfp image
    let mut image = match fs::File::create(format!("users/{}/pfp.jpg", user)){
        Ok(x) => x,
        Err(e) => {
            println!("Failes to create the pfp file");
            match log("The pfp was unnable to be created", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            return;
        }
    };

    match image.write_all(img){
        Ok(x) => x,
        Err(e) => {
            println!("Failed to write to pfp file");
            match log("Failed to write to pfp file", 1) {
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);
                }
            }
            return;
        }
    }

    let mut image_data = match fs::File::create(format!("users/{}/pfp.txt", user)){
        Ok(x) => x,
        Err(e) => {
            println!("Failes to create the pfp file");
            match log("The pfp was unnable to be created", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            return;
        }
    };

    match image_data.write_all(img_data){
        Ok(x) => x,
        Err(e) => {
            println!("Failed to write to pfp file");
            match log("Failed to write to pfp file", 1) {
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);
                }
            }
            return;
        }
    }

    let mut file = fs::File::create(format!("users/{}/info.txt", user));
    let mut file = match fs::OpenOptions::new() //idk but it feels better this way
        .write(true)
        .append(true)
        .create(true)
        .open(format!("users/{}/info.txt", user)) 
        .map_err(|e| {
            eprintln!("Failed to open info.txt: {:?}", e);
            match log(&format!("Error mapping the users from the users file: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                }
            }
            send_error_response(&mut stream, 500, "Server configuration error");
            return;
        }){
            Ok(x)=> x,
            Err(e) => {
                eprintln!("Failed to open info.txt: {:?}", e);
                match log(&format!("Failed to open the users file: {:?}", e), 3){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 400, &e);   
                    } 
                }
                send_error_response(&mut stream, 500, "File was unnable to be opened");
                return;
            }
        }; 

    //hashing the password
    let pass = match hash(&*pass, DEFAULT_COST).map_err(|e| {
        eprintln!("Failed to map the password: {}", e);
        match log(&format!("Error mapping the password for hash: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 500, "Failed to create account");
    }){
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed to hash the password somehow: {:?}", e);
            match log(&format!("Failed hashing the password: {:?}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 500, "Failed to create account");
            return;
        }
    };

    println!("pass: {}", pass);

    match writeln!(file, "pass: {}***\r\ngender: {}***\r\nprofile: /pfp.jpg***", pass, gender)
    .map_err(|e| {
        eprintln!("Failed to write to users.txt: {}", e);
        match log(&format!("Error writing to the users file: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(&mut stream, 400, &e);   
            } 
        }
        send_error_response(&mut stream, 500, "Failed to create account");
    }){
        Ok(x) => x,
        Err(e) => {
            println!("There is a severe problem in the usersdatabase {:?}", e);
            match log(&format!("Error writing to the file: {:?}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 500, "Failed to create account, please try again later ");
            return;
        }
    };
    
    match fs::create_dir_all(format!("users/{}/uploads", user)) {
        Ok(x) => x,
        Err(e) => {
            match log(&format!("{}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    println!("{}", e);
                    send_error_response(&mut stream, 400, &format!("Failed to log: {}", e));
                    return;
                }
            }
            send_error_response(&mut stream, 400, "Failed to create the folder");
            return;    
        }
    };    
         
    match fs::create_dir_all(format!("users/{}/data", user)) {
        Ok(x) => x,
        Err(e) => {
            match log(&format!("{}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    println!("{}", e);
                    send_error_response(&mut stream, 400, &format!("Failed to log: {}", e));
                    return;
                }
            }
            send_error_response(&mut stream, 400, "Failed to create the folder");
            return;    
        }
    }; 
    let status_line = "HTTP/1.1 200 OK\r\n";
    let site = web(&mut stream, buffer);

    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        println!("site: {}", site);
        return;
    }

    let response = format!("{}Set-Cookie: Auth=\"user-{}-token\"; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\nSet-Cookie: Folder=\"folder--token\"; Path=/; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, user, site);

    // println!("\n\n\n\n\nresponse = \n{}", response);
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


pub fn logout(mut stream: TcpStream, buffer: Request){

    let body = buffer.header;
    //first check if it is lofgged in
    let start = match memmem::find(&body[..], b"Auth=\"user-").map(|p| p as usize) {
        Some(x) => x,
        None => {
            send_error_response(&mut stream, 404, "Failed to find the Auth key");
            return;
        }
    };
    let inner =&body[start + "Auth=\"user-".len()..];

    let end = match memmem::find(&inner[..], b"-token").map(|p| p as usize) {
        Some(x) => x,
        None => {
            send_error_response(&mut stream, 404, "Failed to fid the Auth key");
            return;
        }
    };

    let token = String::from_utf8_lossy(&inner[..end]);

    if (token == "") {
        println!("You are not logged in so what are you disconnecting from?");
        send_error_response(&mut stream, 400, "You are currently not logged in");
        return;
    }

    //return no cookie and to the login page
    println!("token {}", token);
    //then do whatever u need if it isn't logged in before asking this

    let status_line =  "HTTP/1.1 200 OK\r\n";
    let mut response = format!("{}Set-Cookie: Auth=\"user--token\"; Path=/; HttpOnly; SameSite=Strict; Max-Age=0\r\nLocation: /\r\nSet-Cookie: Folder=\"folder--token\"; Path=/; SameSite=Strict; Max-Age=0\r\nLocation: /\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, login_signup());

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

fn failed_attempt(status_line: &str, user: &str, time: f32) -> String{
    println!("User 2 {} has been temporarily blocked for {} seconds due to too many failed login attempts", user, time );
    format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, password(user.to_string(), Some( &format!("You have been temporarily blocked for {} minutes due to too many failed login attempts for now", time / 60.0))))
    
}

fn incorrect_pass(stream: &mut TcpStream, username: &str) {
    let status_line =  "HTTP/1.1 200 OK\r\n";
    let mut response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, password(username.to_string(), Some("try to remember the password u used when creating this account you fucking bitch")));
    //store here the user and amount of failed attempts

    let mut attempts;
    {
        attempts = match USERS_ATTEMPTS.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the attempts mutex\n{:?}", e);
                send_error_response(stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
    }

    // println!("Username that should exist: {:?}", username);
    // println!("Let's see who tried to connect: {:?}", attempts.iter().collect::<Vec<_>>()); 

    let (count, locked_until) = attempts.entry(username.to_string()).or_insert((0, None));
    /* 
        this shit kinda works 
        keep testing it to make sure
    */

    // println!("The user's attempt: {}", *count);
    let time_remaining = match *count {//it increments later
        0 => 0,
        1 => 0,
        2 => 30,
        3 => 60,
        4 => 120,
        5 => 300,
        6 => 900,
        7 => 1800,
        8 => 3600,
        9 => 7200,
        10 => 14400,
        11 => 28800,
        12 => 57600,
        _ => 18446744073709551615, //good luck recovering you accoutn bozo
    };
    // solve this bullshit tmrw
    
    if *locked_until == None {

        *count += 1;
        match *count {
            1 => {
                println!("hopa, slipped once");
                response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, password(username.to_string(), Some("You have 2 attempts left before you are timed out"))); 
            },
            2 => {
                println!("Hopa, slipped twice");
                response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, password(username.to_string(), Some("You have 1 attempt left before you are timed out"))); 
            },
            _ => {
                println!("Time now {} vs time i should unlock {:?}", time_remaining, Instant::now() + Duration::from_secs(time_remaining));
                if *count >= 12 {
                    println!("RIP FUCKING BOZOOOOOOOOOOOOOOOOOO");
                }
                *locked_until = Some(Instant::now() + Duration::from_secs(time_remaining));
            }
        }
    }

    if let Some(unlock_time) = locked_until {
        if Instant::now() < *unlock_time {
            let remaining = unlock_time.duration_since(Instant::now()).as_secs();
            response = failed_attempt(status_line, &username, (remaining) as f32);
        }
        else {
            *locked_until = None;
            //counter se reseteaza doar atunci cand parola este corecta
        }
    }
    //IT COMPILESSSS

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Write error: {}", e);
        match log(&format!("Write error: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(stream, 400, &e);   
            } 
        }
    }
    if let Err(e) = stream.flush() {
        eprintln!("Error flushing: {}", e);
        match log(&format!("Error flushing: {}", e), 3){
            Ok(x) => x,
            Err(e) => {
                send_error_response(stream, 400, &e);   
            } 
        }
    }
}