use super::*;
use bcrypt::{DEFAULT_COST, hash, verify};

pub fn auth_user(mut stream: TcpStream, buffer: Vec<u8>) {
    let name =  match memmem::find(&buffer[..], b"account=").map(|p| p as usize){
        Some(x) => x,
        None => {
            send_error_response(&mut stream, 510, "-How did you find me?<br>-GPS tapped on your FUCKING boat");
            println!("Dexter reference activated 1");
            return;
        }
    };
    let name = &buffer[name + "account=".len()..];
    let name = String::from_utf8_lossy(&name[..]);

    let status_line = "HTTP/1.1 200 OK\r\n";
    let response = format!("{}{}", status_line, password(name.to_string(), None));//dont ask homie

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

pub fn auth_pass(mut stream: TcpStream, buffer: Vec<u8>) {
    // println!("{}", String::from_utf8_lossy(&buffer[..]));
    let user = match memmem::find(&buffer[..], b"user=").map(|p| p as usize){
        Some(x) => x,
        None => {
            send_error_response(&mut stream, 510, "-Holy fucking shit, You are thvere bay harbout butcher<br>-I never liked that nickname");
            println!("Dexter reference activated 2");
            return;
        }
    };
    let user = &buffer[user + "user=".len()..];
    let end = match memmem::find(&user[..], b"&").map(|p| p as usize){
        Some(x) => x,
        None => {
            send_error_response(&mut stream, 510, "How tf did u get here");
            println!("Just... how?");
            return;
        }
    };
    let user = &user[..end];

    let user = String::from_utf8_lossy(&user[..]);

    let pass = match memmem::find(&buffer[..], b"password=").map(|p| p as usize){
        Some(x) => x,
        None => {
            send_error_response(&mut stream, 510, "You gotta be jorking with me");
            println!("Bruh");
            return;
        }
    };
    let pass = &buffer[pass + "password=".len()..];
    let pass = String::from_utf8_lossy(&pass[..]);

    let mut text = Vec::new();
    {
        let mut file = match fs::File::open("users.txt") {
            Ok(c) => c,
            Err(_) => match fs::File::create_new("users.txt"){
                Ok(x) => x,
                Err(e) => {
                    println!("failed to create the ursers \"database\"");
                    return Default::default();
                },
            },
        };

        file.read_to_end(&mut text);
    }

    let hashed_pass = match hash(&*pass, DEFAULT_COST).map_err(|e| {
                    eprintln!("Failed to hash password: {}", e);
                    send_error_response(&mut stream, 500, "Failed to log in with this password");
                }){
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("Failed to hash the password somehow: {:?}", e);
                        send_error_response(&mut stream, 500, "Failed to find account");
                        return;
                    }
                };
    let search = format!("{}: {} ",user, hashed_pass);
    let search = search.as_bytes();


    println!("text in da file ={:?}", text);
    println!("text in string form = {}", String::from_utf8_lossy(&text[..]));
    println!("user = {:?}", user);
    println!("pass = {:?}", pass);
    println!("search = {:?}", String::from_utf8_lossy(&search[..]));
    println!("\n\n\n\n\n");
    

    // check if the person is in the file 
    // else add user and pass
    // but if user is but pass isnt
    // make the user retry pass

    //
    match memmem::find(&text[..], user.as_bytes()).map(|p| p as usize){
        Some(user) => {
            //search for the username
            let search_boundary = &text[user..];
            let mut end = memmem::find_iter(&search_boundary, " ").map(|p| p as usize);
            end.next(); 
            let search_boundary = &search_boundary[..end.next().unwrap() + 1];
            println!("search_boundary = {:?}", String::from_utf8_lossy(&search_boundary[..])); 
            println!("search = {:?}", String::from_utf8_lossy(&search[..])); 

            //if the password doesnt match with the username
            if !(search_boundary == search) {
                let status_line =  "HTTP/1.1 200 OK\r\n";
                let response = format!("{}{}",status_line, password(user.to_string(), Some("try to remember the password u used when creating this account")));
        
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
                return;
            } 
            //else if it matches do nothing
            ()
        },
        None => {
            //if the user doesnt exist create it
            let _ = { 
                let metadata = match fs::metadata(Path::new("users.txt")){
                    Ok(x) => x,
                    Err(e) => {
                        send_error_response(&mut stream, 500, "There is a problem starting the users database, please try again later");
                        return;
                    }
                };
                if metadata.permissions().readonly() {
                    println!("Write permission denied for {:?}", metadata);
                } else {
                    println!("Write permission accepted? for {:?}", metadata);
                }

                let mut file = match fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open("users.txt")
                    .map_err(|e| {
                        eprintln!("Failed to open users.txt: {:?}", e);
                        send_error_response(&mut stream, 500, "Server configuration error");
                        return;
                    }){
                        Ok(x)=> x,
                        Err(e) => {
                            eprintln!("Failed to open users.txt: {:?}", e);
                            send_error_response(&mut stream, 500, "Server configuration error");
                            return;
                        }
                    }; 

                //hashing the password
                let pass = match hash(&*pass, DEFAULT_COST).map_err(|e| {
                    eprintln!("Failed to hash password: {}", e);
                    send_error_response(&mut stream, 500, "Failed to create account");
                }){
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("Failed to hash the password somehow: {:?}", e);
                        send_error_response(&mut stream, 500, "Failed to create account");
                        return;
                    }
                };

                match writeln!(file, "{}: {} ", user, pass)
                .map_err(|e| {
                    eprintln!("Failed to write to users.txt: {}", e);
                    send_error_response(&mut stream, 500, "Failed to create account");
                }){
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("There is a severe problem in the usersdatabase");
                        send_error_response(&mut stream, 500, "Failed to create account, please try again later ");
                        return;
                    }
                };
            };
            ()
        }, //do some shit
        //add user with pass
    }     
    //if the user and pass match show the corresponding 


    {
        let mut folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return;
            }
        };
        *folder = (&user).to_string(); 
        println!("folder ={}", *folder);
        // fs::read_dir(format!("uploads/{}", *folder)).unwrap();
        match fs::read_dir(format!("uploads/{}", *folder)) {
            Err(_) => {
                fs::create_dir_all(format!("uploads/{}", folder));
                fs::create_dir_all(format!("data/{}", folder));
            }
            _ => println!("everything is allright"),
        }
    }

    

    let status_line = "HTTP/1.1 200 OK\r\n";
    let site = web(&buffer[..]);
    println!("site thats giving me problems:\n{}", site);

    if(!memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some()){
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        return;
    }

    let response = format!("{}Set-Cookie: Auth=\"user-{}-token\"; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, user, site);

    // println!("\n\n\n\n\nresponse = \n{}", response);
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
