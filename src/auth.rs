use super::*;
use bcrypt::{DEFAULT_COST, hash, verify};

pub fn auth_user(mut stream: TcpStream, buffer: Request) {
    let name =  match memmem::find(&buffer.body.clone().unwrap()[..], b"account=").map(|p| p as usize){
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
    let name = &buffer.body.clone().unwrap()[name + "account=".len()..];
    let name = String::from_utf8_lossy(&name[..]);

    let status_line = "HTTP/1.1 200 OK\r\n";
    let response = format!("{}{}", status_line, password(name.to_string(), None));//dont ask homie

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

pub fn auth_pass(mut stream: TcpStream, buffer: Request) {
    let user = match memmem::find(&buffer.body.clone().unwrap()[..], b"user=").map(|p| p as usize){
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
    let user = &buffer.body.clone().unwrap()[user + "user=".len()..];
    let end = match memmem::find(&user[..], b"&").map(|p| p as usize){
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
    let user = &user[..end];

    let user = String::from_utf8_lossy(&user[..]);

    let pass = match memmem::find(&buffer.body.clone().unwrap()[..], b"password=").map(|p| p as usize){
        Some(x) => x,
        None => {
            match log("The request probably got corrupted", 2){
                Ok(x) => x,
                Err(e) => {
                    send_error_response(&mut stream, 400, &e);   
                } 
            }
            send_error_response(&mut stream, 510, "You gotta be jorking with me");
            println!("Bruh");
            return;
        }
    };
    let pass = &buffer.body.clone().unwrap()[pass + "password=".len()..];
    let pass = String::from_utf8_lossy(&pass[..]);

    let mut text = Vec::new();
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

    // let hashed_pass = match hash(&*pass, DEFAULT_COST).map_err(|e| {
    //     eprintln!("Failed to hash password: {}", e);
    //     send_error_response(&mut stream, 500, "Failed to log in with this password");
    // }){
    //     Ok(x) => x,
    //     Err(e) => {
    //         eprintln!("Failed to hash the password somehow: {:?}", e);
    //         send_error_response(&mut stream, 500, "Failed to find account");
    //         return;
    //     }
    // };
    // let search = format!("{}: {} ",user, hashed_pass);
    // let search = search.as_bytes();

    // println!("text in da file ={:?}", text);
    // println!("text in string form = {}", String::from_utf8_lossy(&text[..]));
    // println!("user = {:?}", user);
    // println!("pass = {:?}", pass);
    // println!("search = {:?}", String::from_utf8_lossy(&search[..]));
    // println!("\n\n\n\n\n");

    match memmem::find(&text[..], user.as_bytes()).map(|p| p as usize){
        Some(user) => {
            //search for the username
            let search_boundary = &text[user..];
            let mut end = memmem::find_iter(&search_boundary, " ").map(|p| p as usize);
            end.next(); 
            let search_boundary = &search_boundary[..end.next().unwrap() + 1];
            // println!("search_boundary = {:?}", String::from_utf8_lossy(&search_boundary[..])); 
            // println!("search = {:?}", String::from_utf8_lossy(&search[..])); 

            //aici verifica daca hashed pass = unhashed pass 
            let stored_pass = match memmem::find(&search_boundary[..], b": ").map(|p| p as usize){
                Some(x) => x,
                None => {
                    send_error_response(&mut stream, 500, "There is a problem with the users database, please try again later");
                    println!("There is a problem with the users database, please try again later");
                    return;
                }
            }; //hashed pass

            let username = String::from_utf8_lossy(&search_boundary[..stored_pass]); //username
            let stored_pass = String::from_utf8_lossy(&search_boundary[stored_pass + 2.. search_boundary.len() - 1]); //hashed pass
            let unhashed_pass = pass; 

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

                //else if it matches do nothing
        },
        None => {
            //if the user doesnt exist create it
            let _ = { 
                let metadata = match fs::metadata(Path::new("users.txt")){
                    Ok(x) => x,
                    Err(e) => {
                        println!("Error: {}", e);
                        send_error_response(&mut stream, 500, "There is a problem starting the users database, please try again later");
                        return;
                    }
                };
                if metadata.permissions().readonly() { //idk why i'm checking this but it's a good thing ig?
                    match log("The user file cannot be write to, something bad happened", 3){
                        Ok(x) => x,
                        Err(e) => {
                            send_error_response(&mut stream, 400, &e);   
                        } 
                    }
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
                            eprintln!("Failed to open users.txt: {:?}", e);
                            match log(&format!("Failed to open the users file: {:?}", e), 3){
                                Ok(x) => x,
                                Err(e) => {
                                    send_error_response(&mut stream, 400, &e);   
                                } 
                            }
                            send_error_response(&mut stream, 500, "Server configuration error");
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

                match writeln!(file, "{}: {} ", user, pass)
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
                return;
            };
        },
    } //do some shit
      //add user with pass
         
    //if the user and pass match show the corresponding 


    {
        let mut folder = match SHOW_FOLDER.lock(){
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
        *folder = (&user).to_string(); 
        // println!("folder ={}", *folder);
        // fs::read_dir(format!("uploads/{}", *folder)).unwrap();
        match fs::read_dir(format!("uploads/{}", *folder)) {
            Err(_) => {
                match fs::create_dir_all(format!("uploads/{}", folder)) {
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
                match fs::create_dir_all(format!("data/{}", folder)) {
                    Ok(x) => x,
                    Err(e) => {
                        match log(&format!("{}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                println!("{}", e);
                                send_error_response(&mut stream, 400, "Failed to log");
                                return;
                            }
                        }

                        send_error_response(&mut stream, 400, "Failed to create the folder");
                        return;    
                    }
                };
            }
            _ => println!("everything is allright"),
        }
    }

    let status_line = "HTTP/1.1 200 OK\r\n";
    let site = web(buffer);

    if !memmem::find(site.as_bytes(), b"<!DOCTYPE html>").map(|p| p as usize).is_some() {
        send_error_response(&mut stream, 400, "There has been an error generating the webpage");
        println!("site: {}", site);
        return;
    }

    let response = format!("{}Set-Cookie: Auth=\"user-{}-token\"; Path=/; HttpOnly; SameSite=Strict; Max-Age=3600\r\nLocation: /\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, user, site);

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

fn failed_attempt(status_line: &str, user: &str, time: f32) -> String{
    println!("User 2 {} has been temporarily blocked for {} seconds due to too many failed login attempts", user, time );
    format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}", status_line, password(user.to_string(), Some( &format!("You have been temporarily blocked for {} minutes due to too many failed login attempts for now", time / 60.0))))
    
}

fn incorrect_pass(stream: &mut TcpStream, username: &str) {
    let status_line =  "HTTP/1.1 200 OK\r\n";
    let mut response = format!("{}Content-Type: text/html; charset=UTF-8\r\n\r\n{}",status_line, password(username.to_string(), Some("try to remember the password u used when creating this account you fucking bitch")));
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