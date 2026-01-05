    use super::*;

    pub fn web(stream: &mut TcpStream, buffer: Request) -> String {

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
            .loader-container {
                text-align: center;
                padding: 20px;
            }

            .spinner {
                border: 8px solid rgba(0, 0, 0, 0.1);
                width: 72px;
                height: 72px;
                border-radius: 50%;
                border-left-color: #09f;
                animation: spin 1s ease infinite;
                display: inline-block;
            }

            @keyframes spin {
                0% { transform: rotate(0deg); }
                20% {transform: rotate(50deg); }
                60% {transform: rotate(200deg); }
                100% { transform: rotate(360deg); }
            }

        </style>

        <body>
        <h1>Hello!</h1>
        <p>Welcome to your file server :) &#10003; </p>

        <form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">
            <input type=\"file\" name=\"file\"  required>
            <button type=\"submit\">Upload</button>
        </form> 

        <form action=\"/\" method=\"POST\">
            <input type=\"hidden\" name=\"**action\" value=\"ADD_FOLDER\">
            <input type=\"text\" name=\"filename\" required>
            <button type=\"submit\">Add Folder </button>
        </form> 
        <span id='breadcrumb'>
        </span>
        "
        );
        /*
        //need to make the breadcrum on js too...
        let user = checkAuth(stream, buffer.clone());
        let folder = checkFolder(stream, buffer.clone());
        if user != "" && memmem::find(&folder.as_bytes()[..], b"/").map(|p| p as usize).is_some() {

            println!("breadcrumbing... ");
            let f = memmem::find(&folder.as_bytes()[..], b"/").map(|p| p as usize).unwrap(); //augu/a

            let folder_b = &folder.as_bytes()[f..];
            let user_folder = String::from_utf8_lossy(&folder_b[..]); // /a
            // println!("folder that im currently in= {}", user_folder);
            println!("breadcrumb2: {}", user_folder);

            if &user_folder != "/"  {         //911 joke incoming //it was on line 911 at the time of writing that comment
                let breadcrumb = memmem::rfind(&folder_b[..], b"/").map(|p| p as usize).unwrap(); // *empty*
                let parent_folder = &folder_b[..breadcrumb];

                println!("breadcrumb3: {}", String::from_utf8_lossy(&parent_folder[..]));

                html.push_str(&*format!(
                    "  
                    Location: {}
                    <br>
                    <button onclick=\"window.location.href='/open_folder..{}'\">Go back to home</button>
                    ",
                    String::from_utf8_lossy(&user_folder.as_bytes()[1..]),
                    &user[..f]

                ));
                if !parent_folder.is_empty() {
                    html.push_str(&*format!(
                        "<br>
                        <button onclick=\"window.location.href='/open_folder..{}'\">Go back 1 layer {:?}</button>
                        
                        ",
                        format!("{}/{}", &user[..f], String::from_utf8_lossy(parent_folder)),
                        format!("{}/{}", &user[..f], String::from_utf8_lossy(parent_folder))
                    ));
                }
            }
        } */
        
        html.push_str(r#"
            <h3> File ordering </h3>
            <select id="ordering" onChange="changed(this)">
                <option value="0">A-Z</option>
                <option value="1">New to Old</option>
                <option value="2">Big to Small</option>
                <option value="3">File Type</option>
                <option value="4">Z-A</option>
                <option value="5">Old to New</option>
                <option value="6">small to Big</option>
            </select>
            <h2> Saved Files:</h2>
            <ul id="files-window">
        "#);

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
            }"
        );

        html.push_str(r#"
            const filesWindow = document.getElementById('files-window');
            const bread = document.getElementById('breadcrumb');
            const option = document.getElementById('ordering');

            document.addEventListener('DOMContentLoaded', function() {{
                console.log(option);
                console.log("cookieee " + document.cookie);
                breadcrumbs();
                getFiles("0");
            }})

            function changed(element) {{
                console.log(element);
                console.log(element.value);
                getFiles(element.value);
            
            }} 

            function breadcrumbs() {{
                let str = document.cookie;
                let first = str.indexOf("Folder=\"folder-");
                let end = str.indexOf("-token");

                console.log(str);
                console.log("did it return???");
                let folder = str.slice(first + 15, end);
                console.log("folder" + folder);


                
                if (folder !== "") {{
                    
                    let parent = [...folder.matchAll(/\//g)];
                    let last = parent.at(-1);
                    console.log("last slash at index:", last);

                        bread.innerHTML = `
                            Location: ${folder}
                            <br>
                            <form action="/" method=\"POST\">
                                <input type="hidden" name="**action" value="**home">
                            <input type="hidden" name="filename" value="/">
                            <button type="submit">Go back to home</button>
                        </form> 
                    `
                    if (last.index !== 0) {{

                    bread.innerHTML = `
                        Location: ${folder}
                        <br>
                        
                        <form actionn="/" method=\"POST\">
                            <input type=\"hidden\" name="**action" value="**open_folder..">
                            <input type=\"hidden\" name="filename" value="/">
                            <button type=\"submit\">Go back 1 layer back</button>
                        </form> 
                        <form action="/" method=\"POST\">
                            <input type=\"hidden\" name=\"**action" value=\"**home">
                            <input type=\"hidden\" name="filename" value="/">
                            <button type=\"submit\">Go Home</button>
                        </form> 
                        
                    `
                    }}
                }}  
            }}  

            async function getFiles(value) {{

            filesWindow.innerHTML = `    
                <div class="loader-container">
                    <p>⌛ Loading your files, please wait...</p>
                    <div class="spinner"></div> 
                </div>
            `;

                const params = {
                    order: value,
                };

                const options = {
                    method: 'POST',
                    body: JSON.stringify( params ),
                };  

                console.log(options);
                try {{
                    const response = await fetch('/files_fetch', options);

                    console.log(response);
                    let data = await response.json();
                    console.log("data??");
                    console.log(data);

                    console.log(data);
                    setTimeout(() => {{ //simulate slow server
                        insertFiles(data);
                    }}, 2000);

                }} catch (error) {{
                    console.error('Error loading the files:', error);
                    filesWindow.innerHTML = `<p style="color: red;">❌ Failed to load files: ${error.message}</p>`;
                }}
                
            }}

            function insertFiles(list) {{
                if (list.length == 0) {{
                    filesWindow.innerHTML = `<h2> Let's start uploading some files :3 </h2>`
                }} else {{
                    filesWindow.innerHTML = list.map(file=> {{
                        console.log(file);
                        if (file.is_file == true) {{
                            if (file.file_type == "image") {{
                                return `
                                <li>
                                    <h3>
                                        ${file.realname}
                                    </h3>
                                    <br>
                                    <button class=\"options_file\" id=\"${file.id}\" onclick=\"open_file_options(this)\"> 
                                        <span> &#8942; </span>
                                    </button>
                                    <div id=\"options\" style=\"display:none; z-index: 10\">
                                        <div style=\"margin:0 10px 0 0;\">
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DELETE\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.uploadsname}\">
                                                <button type=\"submit\"> Delete </button>
                                            </form>
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DOWNLOAD\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.uploadsname}\">
                                                <button type=\"submit\"> DOWNLOAD </button>
                                            </form>
                                        </div>
                                        <img src=${file.uploadspath} alt =\"IDFK\" style=\"max-width: 300px;\" >
                                    </div>
                                </li>`
                            }} else if (file.file_type == "video") {{
                                return ` 
                                <li>
                                    <h3>
                                        ${file.realname}
                                    </h3>
                                    <br>
                                    <button class=\"options_file\" id=\"${file.id}\" onclick=\"open_file_options(this)\"> 
                                        <span> &#8942; </span>
                                    </button>
                                    <div id=\"options\" style=\"display:none; z-index: 10\">
                                        <div style=\"margin:0 10px 0 0;\">
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DELETE\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.uploadsname}\">
                                                <button type=\"submit\"> Delete </button>
                                            </form>
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DOWNLOAD\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.uploadsname}\">
                                                <button type=\"submit\"> DOWNLOAD </button>
                                            </form>
                                        </div>
                                        <video width=\"300\" height =\"240\" controls>
                                            <source src=\"${file.uploadspath}\" type=\"${file.f_type}\">
                                            Your browser doesnt support my video :'(
                                        </video>
                                    </div>
                                </li>
                                `
                            }} else if (file.file_type == "text") {{
                                return ` 
                                <li>
                                    <h3>
                                        ${file.realname}
                                    </h3>
                                    <br>
                                    <button class=\"options_file\" id=\"${file.id}\" onclick=\"open_file_options(this)\"> 
                                        <span> &#8942; </span>
                                    </button>
                                    <div id=\"options\" style=\"display:none; z-index: 10\">
                                        <div style=\"margin:0 10px 0 0;\">
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DELETE\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.uploadsname}\">
                                                <button type=\"submit\"> Delete </button>
                                            </form>
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DOWNLOAD\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.uploadsname}\">
                                                <button type=\"submit\"> DOWNLOAD </button>
                                            </form>
                                        </div>
                                    </div>
                                </li>
                                `
                            }}
                        }} else {{
                            return `
                            <li>
                                <h3>
                                    ${file.realname}
                                </h3>
                                <br>
                                <button class="options_folder" id="${file.id}" onclick="open_folder_options(this)"> 
                                    <span> &#8942; </span>
                                </button>
                                <div id="options" style="display: none; z-index: 10">
                                    <div style="display: block;  margin:0 10px 0 0;">
                                        <form action="/" method ="POST">
                                            <input type= "hidden" name= "**action" value= "DELETE">
                                            <input type= "hidden" name= "folder" value= "${file.folder}">
                                            <input type= "hidden" name= "filename" value= "${file.datapath}">
                                            <button type= "submit">Delete</button>
                                        </form>
                                        <form action="/" method ="POST">
                                            <input type= "hidden" name= "**action" value= "RENAME_FOLDER">
                                            <input type= "hidden" name= "filename" value= "${file.folder}">
                                            <input type= "text\" name= "newFile">
                                            <button type= "submit">Rename</button>
                                        </form>
                                        <form action="/" method="POST">
                                            <input type= "hidden" name= "**action" value= "DOWNLOAD_FOLDER">
                                            <input type= "hidden" name= "filename" value= "${file.datapath}">
                                            <button type= "submit">Download as ZIP</button>
                                        </form>
                                    </div>
                                    <form actionn="/" method= "POST">
                                        <input type= "hidden" name= "**action" value= "OPEN_FOLDER">
                                        <input type= "hidden" name= "filename" value= "${file.realname}">
                                        <button type= "submit">Open Folder</button>
                                    </form> 
                                </div>
                            </li>
                            `
                        }}
                    }}).join('');
                }}
            }}
        "#);

        html.push_str("
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
                <input type=\"text\" name=\"**account\">
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
                <input type=\"hidden\" name=\"**user\" value=\"{}\">
                <input type=\"text\" name=\"**password\">
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
