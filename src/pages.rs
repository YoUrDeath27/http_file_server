    use super::*;

    pub fn web(buffer: Request) -> String {
        let folder = match SHOW_FOLDER.lock(){
                Ok(x) => x,
                Err(e) => {
                    println!("cant identify the user from the folder mutex\n{:?}", e);
                    match log(&format!("Error identifying the user from Mutex : {}", e), 3){
                        Ok(x) => x,
                        Err(_e) => {
                            // send_error_response(&mut stream, 400, &e);   
                            return String::from("");
                        } 
                    }
                    // send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                    return String::from("");
                }
            };

        let sorting_option = 1; //e.g. alphabetical, uploaded time, type, size 
        let mut sorted = sorting(sorting_option, folder.to_string());
        // match sorting_option {
        //     0 => alfabetical_order(folder.to_string()),
        //     1 => upload_order(folder.to_string()),
        //     _ => alfabetical_order(folder.to_string()),
        // };

        let sorted = match sorted {
            Ok(x) => x,
            Err(e) => {
                println!("There was a problem sorting the files: {}", e);
                match log(&format!("There was a problem sorting the files: {}", e), 3){
                    Ok(x) => x,
                    Err(_e) => {
                        // send_error_response(&mut stream, 400, &e);   
                        return String::from("");
                    } 
                }
                // send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return String::from("");
            }
        };

        println!("\nsorted: {:?}", sorted); //could do better???
        

        let mut html = String::from(
            "    
        <!DOCTYPE html>
        <html lang=\"en\">
        <head>
        <meta charset=\"UTF-8\">
        <title>File Upload</title>
        </head>

        <style>
            li{
                display: flex;
                margin: auto;
                height: 100px;
                padding: 10px;
                justify-content: center;
                align-items: center;
                font-size: 30px;
            }

            li > form {
                margin: 0;
            }

            ul > li > div:nth-child(1){
                margin: 0;
            }

            li > form > button{
                margin: 0 10px;
                font-size: 25px;
            }

            li > button {
                font-size: 25px;
            }

            li > div:nth-child(2) {
                margin:0 0 0 50px;
            }

            .options_file, .options_folder {
                background: none;
                border: none;
                font-size: 10rem;
                position: relative;
            }

            #options {
                background-color: lightgrey;
                border-radius: 10px;
                padding: 20px;
            }

            .options::nth-child(1) {
                display: flex;
            }

            .options_file:hover, .options_folder:hover {
                color: blue;
                cursor: pointer;
            }

        </style>

        <body>
        <h1>Hello!</h1>
        <p>Welcome to your file server :) &#10003; </p>

        <form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">
            <input type=\"file\" name=\"file\"  required>
            <button type=\"submit\">Upload</button>
        </form> 

        <form action\"/\" method=\"POST\">
            <input type=\"hidden\" name=\"action\" value=\"ADD_FOLDER\">
            <input type=\"text\" name=\"filename\" required>
            <button type=\"submit\">Add Folder </button>
        </form> "
        );
        
        if let Some(user) =  memmem::find(&buffer.header[..], b"Cookie: Auth=\"user-").map(|p| p as usize) {
            
            let user_folder = &*folder.as_bytes();
            let user = &buffer.header[user + "Cookie: Auth=\"user-".len() ..];
            let end = match memmem::find(user, b"-token").map(|p| p as usize){
                Some(x) => x,
                None => {
                    println!("Unnable to find the end of Auth token");
                    match log("Error finding the user's Auth token", 2){
                        Ok(x) => x,
                        Err(_e) => {
                            // send_error_response(&mut stream, 400, &e); 
                            return String::from("");  
                        } 
                    }
                    // send_error_response(&mut stream, 404, "We were unnable to locate your auth key<br> u tampered with it right?");
                    return String::from("");
                }
            };
            let user = &user[..end];

            let folder_b = &user_folder[user.len()..];
            let user_folder = String::from_utf8_lossy(&folder_b[..]);
            // println!("folder that im currently in= {}", user_folder);

            if &user_folder != ""  {         //911 joke incoming //it was on line 911 at the time of writing that comment
                let breadcrumb = memmem::rfind(&folder_b[..], b"/").map(|p| p as usize).unwrap();
                let parent_folder = &folder_b[..breadcrumb];
                html.push_str(&*format!(
                    "
                    Location: {}
                    <br>
                    <button onclick=\"window.location.href='/'\">Go back to home</button>
                    ",
                    percent_decode_str(&folder)
                    .decode_utf8_lossy()
                    .replace("+", " ")
                    .to_owned()
                ));
                if !parent_folder.is_empty() {
                    html.push_str(&*format!(
                        "<br>
                        <button onclick=\"window.location.href='/open_folder..{}'\">Go back 1 layer {:?}</button>
                        ",
                        String::from_utf8_lossy(parent_folder),
                        String::from_utf8_lossy(parent_folder)
                    ));
                }
            }
        } 
        
        html.push_str("
            <h2> Saved Files:</h2>
            <ul>
        ");

        let file_folder = &folder;
        for i in 0..sorted.len() {  
            if !sorted[i].is_file {
            
                html.push_str(&*format!(
                    "<li>
                        <h3>
                            {}
                        </h3>
                        <br>
                        <button class=\"options_folder\" id=\"{}\"  onclick=\"open_folder_options(this)\"> 
                            <span> &#8942; </span>
                        </button>
                        <div id=\"options\" style=\"display: none; z-index: 10\">
                            <div style=\"display: block;  margin:0 10px 0 0;\">
                                <form action=\"/\" method =\"POST\">
                                    <input type=\"hidden\" name=\"action\" value=\"DELETE\">
                                    <input type=\"hidden\" name=\"folder\" value=\"{}\">
                                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                                    <button type=\"submit\">Delete</button>
                                </form>
                                <form action=\"/\" method =\"POST\">
                                    <input type=\"hidden\" name=\"action\" value=\"RENAME_FOLDER\">
                                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                                    <input type=\"text\" name=\"newFile\">
                                    <button type=\"submit\">Rename</button>
                                </form>
                                <form action=\"/\" method=\"POST\">
                                    <input type=\"hidden\" name=\"action\" value=\"DOWNLOAD_FOLDER\">
                                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                                    <button type=\"submit\">Download as ZIP</button>
                                </form>
                            </div>
                            <button onclick=\"window.location.href='/open_folder/{}'\">Open folder</button>
                        </div>
                    </li>",
                    sorted[i].realname,
                    i,
                    sorted[i].diskname.replace("\\", "/"),
                    sorted[i].diskname.replace("\\", "/"),
                    sorted[i].diskname.replace("\\", "/"),
                    sorted[i].diskname.replace("\\", "/"),
                    sorted[i].diskname.replace("\\", "/")
                ));
            } else {
                // println!("file: {:?}", file_names[i]);
                // println!("filejs: {}", files[i].display());

                println!("File: {}", folder);

                html.push_str(&*format!(
                    "<li> 
                        <h3>
                            {}
                        </h3>
                        <br>
                        <button class=\"options_file\" onclick=\"open_file_options(this)\" id=\"{}\"> 
                            <span> &#8942; </span>
                        </button>
                        <div id=\"options\" style=\"display:none; z-index: 10\">
                            <div style=\"margin:0 10px 0 0;\">
                                <form action=\"/\" method =\"POST\">
                                    <input type=\"hidden\" name=\"action\" value=\"DELETE\">
                                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                                    <button type=\"submit\"> Delete </button>
                                </form>
                                <form action=\"/\" method =\"POST\">
                                    <input type=\"hidden\" name=\"action\" value=\"DOWNLOAD\">
                                    <input type=\"hidden\" name=\"filename\" value=\"{}\">
                                    <button type=\"submit\"> DOWNLOAD </button>
                                </form>
                            </div>
                        ",
                    sorted[i].realname, 
                    i, 
                    sorted[i].uploads, 
                    sorted[i].uploads
                ));
                let mut content_type_file = match fs::File::open(sorted[i].diskname.clone()){
                    Ok(x) => x,
                    Err(e) => {
                        println!("The user's uploads folder cannot be read\n{:?}", e);
                        match log(&format!("Error reading the uploads folder: {}", e), 3){
                            Ok(x) => x,
                            Err(_e) => {
                                // send_error_response(&mut stream, 400, &e);   
                                return String::from("");
                            } 
                        }
                        return String::from("");
                    }
                };

                let mut content_type = String::new();
                match content_type_file.read_to_string(&mut content_type){
                    Ok(x) => x, 
                    Err(e) => {
                        println!("error: {}", e);
                        match log(&format!("{}", e), 3){
                            Ok(x) => x,
                            Err(e) => {
                                println!("{}", e);
                                return String::from("");
                            }
                        }
                        return String::from("");
                    }
                };
                let end = match memmem::find(&content_type.as_bytes()[..], b";").map(|p| p as usize) {
                    Some(x) => x,
                    None => {
                        println!("SIgma sigma on the wall, why did u mess with the files again?");
                        match log("There is problem reading the content type", 3){
                            Ok(x) => x,
                            Err(_e) => {
                                // send_error_response(&mut stream, 400, &e);  
                                return String::from("");
                            } 
                        }
                        return String::from("");
                    }
                };
                let c_type = String::from_utf8_lossy(&content_type.as_bytes()["Content-Type:".len()..end]);
                println!("FIle data type is: {}", c_type);

                if IMAGE_TYPES.contains(&&*c_type) {
                    html.push_str(&format!("
                        <img src={} alt =\"IDFK\" style=\"max-width: 300px; \" >",
                        sorted[i].uploads
                    ));
                    // println!("image showing");
                } //then check for videos, text and all the other
                else if VIDEO_TYPES.contains(&&*c_type) { //why tf is this not working?
                    html.push_str(&format!("
                        <video width=\"300\" height =\"240\" controls>
                            <source src=\"{}\" type=\"{}\">
                            Your browser doesnt support my video :'(
                        </video>
                    ",  sorted[i].uploads,
                        c_type
                    ));
                    // println!("Video showing");
                }
                html.push_str("</div>
                                </li>\n
                            ");
            }
        }

        html.push_str(
            "
            </ul>
            </body>
            <script>

            function open_file_options(file){
                console.log(file);
                console.log(file.id);
                if (document.getElementById(file.id).parentElement.children[3].style.display  == \"none\") {
                    let button = document.getElementById(file.id);
                    button.parentElement.children[3].style.display = 'flex';
                    //button.parentElement.children[3].children[0].children[2].style.display = 'none';
                    //button.parentElement.children[3].children[1].children[2].style.display = 'none';
                    //button.parentElement.children[3].children[2].style.display = 'none';
                } else {
                    let button = document.getElementById(file.id);
                    button.parentElement.children[3].style.display = 'none';

                }
            }
            function open_folder_options(folder){
                console.log(folder);
                let button = document.getElementById(folder.id);
                if(button.parentElement.children[3].style.display == 'none') {
                    button.parentElement.children[3].style.display = 'flex';
                } else {
                    button.parentElement.children[3].style.display = 'none';
                }
            }
            </script>
            </html>"
            );

        return html;
    }

    pub fn send_error_response(stream: &mut TcpStream, code: u16, message: &str) {
        let status_line = match code {
            400 => "HTTP/1.1 400 Bad Request",
            403 => "HTTP/1.1 403 Forbidden",
            404 => "HTTP/1.1 404 Not Found",
            // 413 => "HTTP/1.1 413 Payload Too Large",
            420 => "HTTP/1.1 420 I know you are the bay harbour butcher",
            500 => "HTTP/1.1 500 Internal Server Error",
            _ => "HTTP/1.1 500 Internal Server Error",
        };

        let response = format!("{}\r\n\r\n{}", status_line, error_web(message));
        // println!("reponse =\n{}", response);
        
        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Write error: {}", e);
            match log(&format!("Write error: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    // send_error_response(&mut stream, 400, &e);  
                    println!("YOu are literally cooked {}", e); 
                } 
            }
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing: {}", e);
            match log(&format!("Error flushing: {}", e), 3){
                Ok(x) => x,
                Err(e) => {
                    // send_error_response(&mut stream, 400, &e);   
                    println!("YOu are literally cooked {:?}", e); 
                } 
            }
        }
    }

    pub fn error_web(message: &str) -> String {
        let mut html = String::from(
            "
        <!DOCTYPE html>
        <html>
        <head>
        <title> Error processing your request </title>
        </head>
        <style>

            button {
                display: flex;
                margin: auto;
                height: 100px;
                width: 343px;
                font-size: 32px;
                padding: 0;
            }
            
        </style>
        <body>
        ",
        );
        html.push_str(&*format!("<h1> {} </h1>", message));
        html.push_str(
            "
            <button onclick=\"window.location.href='/'\"> Go back to the main page </button>
        ",
        );

        html.push_str(" </body> </html>");

        html
    }

    pub fn login_signup() -> String {
        let html = String::from("
            <!DOCTYPE html>
            <html>
            <head>
            <title> Login / Signup bro </title>
            </head>
            <body>
                <h1> Welcome to your File Manager Server </h1>

                <h3> It seems you are not currently connected to an account <br> Please Signup or Login to use this platform<h3>
            <h4> Username: </h4>
            <form action=\"/\" method=\"POST\">
                <input type=\"text\" name=\"account\">
                <button type=\"submit\"> Continue </button>
            </form>
            </body>
            </html>
        ");
        html
    }

    pub fn password(name: String, extra_info: Option<&str>) -> String { 
        
        let mut html = String::from(format!("
            <!DOCTYPE html>
            <html>
            <head>
            <title> Login / Signup bro </title>
            </head>
            <body>
                <h1> Welcome to your File Manager Server </h1>

                <h3> Enter the password for your account<h3>

                <h4> Password: </h4>
            <form action=\"/\" method=\"POST\">
                <input type=\"hidden\" name=\"user\" value=\"{}\">
                <input type=\"text\" name=\"password\">
                <button type=\"submit\"> Login/Signup </button>
            </form>
            <br><br>

            
        ",
        name
        ));

        if let Some(info) = extra_info{
            html.push_str(info);
        }
        html.push_str("
            </body>
            </html>
        ");
        html
    }
