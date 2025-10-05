use super::*;

pub fn web(buffer: &[u8]) -> String {
    let folder = match SHOW_FOLDER.lock(){
            Ok(x) => x,
            Err(e) => {
                println!("cant identify the user from the folder mutex\n{:?}", e);
                // send_error_response(&mut stream, 500, "There is a problem that we dont know how u got here");
                return String::from("");
            }
        };
    // transform this uploads/figet/smashbros/dump%20me 
    // in this uploads/figet/smashbros/dump me 
    
    println!("WEB\n\n\nDefinetly able to enter this folder: uploads/{}", 
            percent_decode_str(&*folder)
                .decode_utf8_lossy()
                .replace("+", " ")
                .to_owned()
            );
    let folder2 = folder.clone();
    let folder3 = folder.clone();

    let binding = decode_Windows_1255(&folder3.into_bytes()[..]);
    let folder3 = percent_decode_str(
        &*binding
    ).decode_utf8_lossy().to_string().into_bytes();
    let folder3 = decode_Windows_1255(&folder3[..]);

    // println!("uploads/{}", 
    //     folder3
    // );

    let data_entries = match fs::read_dir(format!("data/{}", folder3)){
        Ok(x) => x,
        Err(e) => {
            println!("Unnable to read the folder\n{}", e);
            // send_error_response(&mut stream, 404, "Unnable to locate your folder with files, try logging in again");
            return String::from("");
        }
    };

    let entries = match fs::read_dir(format!("uploads/{}", folder3)){
        Ok(x) => x,
        Err(e) => {
            println!("Unnable to read the folder\n{}", e);
            // send_error_response(&mut stream, 404, "Unnable to locate your folder with files, try logging in again");
            return String::from("");
        }
    };

    println!("folder checking: {}", folder);
    
    let mut file_names = Vec::new();

    let mut files = Vec::new();

    for entry in entries {
        let entry = match entry{
            Ok(x) => x,
            Err(e) => {
                println!("No users uploads found\n{:?}", e);
                // send_error_response(&mut stream, 404, "There is a problem accessing your uploads, try again later");
                return String::from("");
            }
        };
        files.push(entry.path());
        let file_name = match entry.file_name().into_string(){
            Ok(x) => x,
            Err(e) => {
                println!("The user's username is unnable to be converted to string\n{:?}", e);
                // send_error_response(&mut stream, 404, "The user contains illegitimate characters");
                return String::from("");
            }
        };
        println!("entry in bytes= {:?}", &file_name.clone().into_bytes()[..]);
        println!("entry in string: {}", String::from_utf8_lossy(&file_name.clone().into_bytes()[..]));
        file_names.push(file_name);
    }

    let mut data_file_names = Vec::new();
    let mut data_files = Vec::new();

    for entry in data_entries {
        let entry = match entry{
            Ok(x) => x,
            Err(e) => {
                println!("No users uploads found\n{:?}", e);
                // send_error_response(&mut stream, 404, "There is a problem accessing your uploads, try again later");
                return String::from("");
            }
        };

        data_files.push(entry.path());
        let file_name = match entry.file_name().into_string(){
            Ok(x) => x,
            Err(e) => {
                println!("The user's username is unnable to be converted to string\n{:?}", e);
                // send_error_response(&mut stream, 404, "The user contains illegitimate characters");
                return String::from("");
            }
        };
        data_file_names.push(file_name);
    }

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
            width: 300px;
            height: 50px;
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
    </style>

    <body>
    <h1>Hello!</h1>
    <p>Welcome to your file server :)</p>

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
    
    if let Some(user) =  memmem::find(buffer, b"Cookie: Auth=\"user-").map(|p| p as usize) {
        
        let folder = &*folder.as_bytes();
        let user = &buffer[user + "Cookie: Auth=\"user-".len() ..];
        let end = match memmem::find(user, b"-token").map(|p| p as usize){
            Some(x) => x,
            None => {
                println!("Unnable to find the end of Auth token");
                // send_error_response(&mut stream, 404, "We were unnable to locate your auth key<br> u tampered with it right?");
                return String::from("");
            }
        };
        let user = &user[..end];

        let folder = &folder[user.len()..];
        let folder = String::from_utf8_lossy(&folder[..]);

        if &folder != ""  {         //911 joke incoming
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
        }
    } //else make the user log in
    

    html.push_str("
        <h2> Saved Files:</h2>
        <ul>
    ");

    for i in 0..file_names.len() {

        println!("file idfk: {}", files[0].display());

        if !files[i].is_file() {
            html.push_str(&*format!(
                "<li>
                    <h3>
                        {}
                    </h3>
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
                    <button onclick=\"window.location.href='/open_folder/{}'\">Open folder</button>
                </li>",
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i],
                file_names[i]
            ));
        } else {
            println!("file: {:?}", file_names[i]);
            println!("filejs: {}", files[i].display());

            html.push_str(&*format!(
                "<li> 
                    <h3>
                        {}
                    </h3>
                    <br>
                    <form action=\"/\" method =\"POST\">
                        <input type=\"hidden\" name=\"action\" value=\"DELETE\">
                        <input type=\"hidden\" name=\"filename\" value=\"{}\">
                        <button type=\"submit\">Delete</button>
                    </form>
                    <form action=\"/\" method =\"POST\">
                        <input type=\"hidden\" name=\"action\" value=\"DOWNLOAD\">
                        <input type=\"hidden\" name=\"filename\" value=\"{}\">
                        <button type=\"submit\">DOWNLOAD</button>
                    </form>",
                file_names[i], file_names[i], file_names[i]
            ));
            let mut content_type_file = match fs::File::open(data_files[i].clone()){
                Ok(x) => x,
                Err(e) => {
                    println!("The user's uploads folder cannot be read\n{:?}", e);
                    return String::from("");
                }
            };

            let mut content_type = String::new();
            content_type_file.read_to_string(&mut content_type);
            let c_type = String::from_utf8_lossy(&content_type.as_bytes()["Content-Type:".len()..]);
            println!("FIle data type is: {}", c_type);

            if Image_Types.contains(&&*c_type) {
                html.push_str(&format!("
                    <img src={} alt =\"IDFK\" style=\"max-width: 300px\">",
                    files[i].display()
                ))
            } //then check for videos, text and all the other
            if Video_Types.contains(&&*c_type) {
                html.push(&format!("
                    <video width=\"300\" height =\"240\" controls>
                        <source src=\"{}\" type=\"{}\">
                        Your browser doesnt support my video :'(
                    </video>
                ",  files[i].display(),
                    c_type
                ))
            }
            html.push_str("<li>\n")
        }
    }

    html.push_str(
        "
        </ul>
        </body>
        </html>",
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
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
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
