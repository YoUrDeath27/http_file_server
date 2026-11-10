    use super::*;

    pub fn web(stream: &mut TcpStream, buffer: Request) -> String {

        let user = checkAuth(stream, buffer.clone());

        let mut html = String::from(format!(
            r#"    
        <!DOCTYPE html>
        <html lang=\"en\">
        <head>
        <meta charset=\"UTF-8\">
        <title>File Upload</title>
        </head>

        <style>
            :root {{
                color-scheme: dark;
                --page: #111518;
                --panel: #1b2125;
                --panel-soft: #222a2f;
                --line: #303b41;
                --text: #e8edf0;
                --muted: #9caab1;
                --accent: #62c6d9;
                --accent-strong: #39a9c1;
                --danger: #e87979;
            }}
            * {{ box-sizing: border-box; }}
            html {{ background: var(--page); }}
            body {{
                color: var(--text);
                background: radial-gradient(circle at 15% 0%, #1d3035 0, var(--page) 38rem);
                font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                margin: 0;
                min-height: 100vh;
                padding: 24px clamp(16px, 4vw, 56px) 48px;
            }}
            button, input, select {{ font: inherit; }}
            button {{
                background: var(--accent);
                border: 0;
                border-radius: 8px;
                color: #0c2025;
                cursor: pointer;
                font-weight: 700;
                padding: 10px 15px;
                transition: transform .18s ease, background .18s ease, box-shadow .18s ease;
            }}
            button:hover {{ background: #86d9e7; box-shadow: 0 5px 18px #3caabd33; transform: translateY(-1px); }}
            button:active {{ transform: translateY(0); }}
            input[type="text"], input[type="file"], select {{
                background: #12181b;
                border: 1px solid var(--line);
                border-radius: 8px;
                color: var(--text);
                padding: 10px 12px;
            }}
            input:focus, select:focus, button:focus-visible {{ border-color: var(--accent); outline: 2px solid #62c6d955; outline-offset: 2px; }}
            #banner {{
                align-items: center;
                background: #1c2529dd;
                border: 1px solid var(--line);
                border-radius: 16px;
                box-shadow: 0 18px 45px #00000028;
                display: flex;
                gap: 16px;
                margin: 0 auto 28px;
                max-width: 1180px;
                min-height: 76px;
                padding: 12px 18px;
                position: sticky;
                top: 14px;
                z-index: 20;
            }}
            #banner h1 {{ font-size: clamp(1.05rem, 2vw, 1.35rem); margin: 0; }}
            #banner p {{ color: var(--muted); margin: 3px auto 0 0; }}
            #banner img {{ border: 2px solid var(--accent); border-radius: 50%; height: 46px; margin-left: auto; object-fit: cover; width: 46px; }}
            #LogOut {{ margin: 0; }}
            #logout {{ background: transparent; border: 1px solid var(--line); color: var(--muted); }}
            #logout:hover {{ background: var(--panel-soft); color: var(--text); }}
            .workspace {{ margin: auto; max-width: 1180px; }}
            .hero {{ margin-bottom: 22px; }}
            .hero h2 {{ font-size: clamp(1.8rem, 4vw, 2.8rem); letter-spacing: -.04em; margin: 0 0 6px; }}
            .hero p {{ color: var(--muted); margin: 0; }}
            .actions {{ display: grid; gap: 14px; grid-template-columns: minmax(0, 1.5fr) minmax(240px, 1fr); margin-bottom: 24px; }}
            .action-card, .files-card {{ background: #1a2125dd; border: 1px solid var(--line); border-radius: 14px; padding: 18px; }}
            .action-card h3 {{ font-size: .95rem; margin: 0 0 12px; }}
            .inline-form {{ align-items: center; display: flex; flex-wrap: wrap; gap: 10px; }}
            .upload-zone {{ border: 1px dashed #52737a; border-radius: 10px; padding: 12px; transition: background .2s ease, border-color .2s ease; }}
            .upload-zone:hover, .upload-zone.dragging {{ background: #62c6d912; border-color: var(--accent); }}
            .upload-zone input[type="file"] {{ border: 0; flex: 1; padding: 4px; }}
            #breadcrumb {{ color: var(--muted); display: block; margin: 0 0 18px; }}
            #breadcrumb form {{ display: inline-block; margin: 10px 8px 0 0; }}
            #breadcrumb button {{ background: transparent; border: 1px solid var(--line); color: var(--text); }}
            .files-toolbar {{ align-items: center; display: flex; flex-wrap: wrap; gap: 12px; justify-content: space-between; }}
            .files-toolbar h2 {{ margin: 0; }}
            .search {{ min-width: min(100%, 240px); }}
            #files-window {{ margin: 18px 0 0; }}
            .file-section {{ border-top: 1px solid var(--line); margin-top: 28px; padding-top: 20px; }}
            .file-section:first-child {{ border-top: 0; margin-top: 0; padding-top: 0; }}
            .file-section h3 {{ color: var(--muted); font-size: .78rem; letter-spacing: .12em; margin: 0 0 12px; text-transform: uppercase; }}
            .file-grid {{ display: grid; gap: 12px; grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); }}
            .file-grid li {{ align-items: flex-start; background: var(--panel); border: 1px solid transparent; border-radius: 13px; display: flex; flex-direction: column; gap: 12px; justify-content: space-between; min-height: 150px; padding: 18px; position: relative; transition: border-color .2s ease, background .2s ease, transform .2s ease; }}
            .file-grid li:hover {{ background: var(--panel-soft); border-color: #3b555d; transform: translateY(-3px); }}
            .file-grid li h3 {{ font-size: 1rem; margin: auto 0 0; overflow: hidden; text-overflow: ellipsis; width: 100%; white-space: nowrap; }}
            .card-top {{ align-items: center; display: flex; justify-content: space-between; width: 100%; }}
            .file-icon {{ align-items: center; background: #62c6d91c; border-radius: 10px; color: var(--accent); display: flex; font-size: 1.7rem; height: 48px; justify-content: center; width: 48px; }}
            .folder-icon {{ background: #d3a75322; border-radius: 4px 6px 6px 6px; display: block; height: 30px; position: relative; transform-origin: bottom left; transition: transform .25s ease; width: 40px; }}
            .folder-icon::before {{ background: #d3a753; border-radius: 4px 5px 0 0; content: ""; height: 7px; left: 3px; position: absolute; top: -5px; width: 17px; }}
            .folder-icon::after {{ background: #e0b967; border-radius: 4px 6px 6px 6px; content: ""; inset: 0; position: absolute; transform-origin: bottom left; transition: transform .25s ease; }}
            .folder-card:hover .folder-icon {{ transform: translateY(-2px); }}
            .folder-card:hover .folder-icon::after {{ transform: perspective(80px) rotateX(-35deg); }}
            .options_file, .options_folder {{ background: transparent; color: var(--muted); font-size: 1.7rem; line-height: 1; padding: 8px 12px; }}
            .options_file:hover, .options_folder:hover {{ background: #62c6d91c; color: var(--accent); }}
            .options {{ align-items: center; background: #111719f5; border: 1px solid var(--line); border-radius: 10px; box-shadow: 0 15px 35px #00000055; display: flex; gap: 14px; padding: 12px; }}
            .options form {{ margin: 0 0 8px; }}
            .options form:last-child {{ margin-bottom: 0; }}
            .options button {{ background: var(--panel-soft); color: var(--text); font-size: .82rem; padding: 8px 10px; width: 100%; }}
            .options form:first-child button {{ color: var(--danger); }}
            .options img, .options video {{ border-radius: 7px; max-height: 180px; max-width: 220px; object-fit: contain; }}
            .loader-container {{ color: var(--muted); padding: 42px 20px; text-align: center; }}
            .spinner {{ border: 5px solid #ffffff12; border-left-color: var(--accent); border-radius: 50%; display: inline-block; height: 42px; width: 42px; animation: spin 1s linear infinite; }}
            .empty-state {{ color: var(--muted); padding: 34px 12px; text-align: center; }}
            @keyframes spin {{ to {{ transform: rotate(360deg); }} }}
            @media (max-width: 700px) {{
                body {{ padding: 12px 12px 32px; }}
                #banner {{ top: 8px; }}
                #banner p {{ display: none; }}
                .actions {{ grid-template-columns: 1fr; }}
                .options {{ align-items: stretch; flex-direction: column; }}
                .options img, .options video {{ max-width: 100%; }}
            }}
        </style>

        <body>
        <div id = "banner">
            <form id = "LogOut" action="/" method="POST">
                <input type="hidden" name="**logout" value="_">
                <button id="logout" type="submit"> LogOut </button>
            </form>
            <h1>Hello!</h1>
            <p>Welcome to your file server :) &#10003; </p>

            <img src="users/{}/pfp.jpg" alt="profile picture`">
        </div>
        

        <main class="workspace">
            <section class="hero">
                <h2>Your files, in one place.</h2>
                <p>Upload, organize, and access your files from anywhere.</p>
            </section>
            <section class="actions">
                <div class="action-card">
                    <h3>Upload a file</h3>
                    <form class="inline-form upload-zone" id="upload-form" action="/" method="POST" enctype="multipart/form-data">
                        <input type="file" name="file" required>
                        <button type="submit">Upload file</button>
                    </form>
                </div>
                <div class="action-card">
                    <h3>Create a folder</h3>
                    <form class="inline-form" action="/" method="POST">
                        <input type="hidden" name="**action" value="ADD_FOLDER">
                        <input type="text" name="filename" placeholder="Folder name" required>
                        <button type="submit">Add folder</button>
                    </form>
                </div>
            </section>
            <span id='breadcrumb'></span>
            <section class="files-card">
                <div class="files-toolbar">
                    <h2>Saved files</h2>
                    <input class="search" id="file-search" type="search" placeholder="Search files..." aria-label="Search files">
                    <label>Sort:
                        <select id="ordering" onChange="changed(this)">
                            <option value="0">A-Z</option>
                            <option value="1">New to Old</option>
                            <option value="2">Big to Small</option>
                            <option value="3">File Type</option>
                            <option value="4">Z-A</option>
                            <option value="5">Old to New</option>
                            <option value="6">Small to Big</option>
                        </select>
                    </label>
                </div>
                <div id="files-window">
        "#
        , user) );
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

        html.push_str(
            "
            </div>
            </section>
            </main>
            </body>
            <script>

            function open_file_options(file){
                const options = file.nextElementSibling;
                const isOpen = options.style.display === 'flex';
                document.querySelectorAll('.options').forEach(item => item.style.display = 'none');
                options.style.display = isOpen ? 'none' : 'flex';
            }
            function open_folder_options(folder){
                const options = folder.nextElementSibling;
                const isOpen = options.style.display === 'flex';
                document.querySelectorAll('.options').forEach(item => item.style.display = 'none');
                options.style.display = isOpen ? 'none' : 'flex';
            }"
        );

        html.push_str(r#"
            const filesWindow = document.getElementById('files-window');
            const bread = document.getElementById('breadcrumb');
            const option = document.getElementById('ordering');
            const search = document.getElementById('file-search');

            document.addEventListener('DOMContentLoaded', function() {
                breadcrumbs();
                getFiles("0");
            })

            function changed(element) {
                getFiles(element.value);
            }

            search.addEventListener('input', function() {
                const query = search.value.toLowerCase().trim();
                filesWindow.querySelectorAll('li').forEach(file => {
                    file.hidden = !file.querySelector('h3').textContent.toLowerCase().includes(query);
                });
            });

            const uploadZone = document.getElementById('upload-form');
            ['dragenter', 'dragover'].forEach(eventName => {
                uploadZone.addEventListener(eventName, function(event) {
                    event.preventDefault();
                    uploadZone.classList.add('dragging');
                });
            });
            ['dragleave', 'drop'].forEach(eventName => {
                uploadZone.addEventListener(eventName, function() {
                    uploadZone.classList.remove('dragging');
                });
            });
            uploadZone.addEventListener('drop', function(event) {
                event.preventDefault();
                if (event.dataTransfer.files.length > 0) {
                    uploadZone.querySelector('input[type="file"]').files = event.dataTransfer.files;
                }
            });

            function breadcrumbs() {
                let str = document.cookie;
                let first = str.indexOf("Folder=\"folder-");
                let end = str.indexOf("-token");

                let folder = str.slice(first + 15, end);
                
                if (folder !== "") {
                    
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
                    if (last.index !== 0) {

                    bread.innerHTML = `
                        Location: ${folder}
                        <br>
                        
                        <form action="/" method=\"POST\">
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
                    }
                }
            }

            async function getFiles(value) {

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
                try {
                    const response = await fetch('/files_fetch', options);

                    console.log(response);
                    let data = await response.json();
                    console.log("data??");
                    console.log(data);

                    insertFiles(data);
                    /*
                    setTimeout(() => { //simulate slow server
                        insertFiles(data);
                    }, 2000);
                    */

                } catch (error) {
                    console.error('Error loading the files:', error);
                    filesWindow.innerHTML = `<p style="color: red;">❌ Failed to load files: ${error.message}</p>`;
                }
                
            }

            function insertFiles(list) {
                if (list.length == 0) {
                    filesWindow.innerHTML = `<div class="empty-state"><h2>Let's start uploading some files :3</h2><p>Your uploaded files will appear here.</p></div>`
                } else {
                    const files = list.filter(file => file.is_file);
                    const folders = list.filter(file => !file.is_file);
                    const renderItems = items => items.map(file => {
                        console.log("file:");
                        console.log(file);
                        if (file.is_file == true) {
                            if (file.file_type == "image") {
                                return `
                                <li class="file-card">
                                    <div class="card-top">
                                        <span class="file-icon">▧</span>
                                        <button class=\"options_file\" id=\"${file.id}\" onclick=\"open_file_options(this)\">
                                            <span> &#8942; </span>
                                        </button>
                                    </div>
                                    <h3>${file.realname}</h3>
                                    <div class=\"options\" style=\"display:none; z-index: 10\">
                                        <div style=\"margin:0 10px 0 0;\">
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DELETE\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.name}\">
                                                <button type=\"submit\"> Delete </button>
                                            </form>
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DOWNLOAD\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.name}\">
                                                <button type=\"submit\"> DOWNLOAD </button>
                                            </form>
                                        </div>
                                        <img src=${file.u_path} alt =\"IDFK\" style=\"max-width: 300px;\" >
                                    </div>
                                </li>`
                            } else if (file.file_type == "video") {
                                return ` 
                                <li class="file-card">
                                    <div class="card-top">
                                        <span class="file-icon">▶</span>
                                        <button class=\"options_file\" id=\"${file.id}\" onclick=\"open_file_options(this)\">
                                            <span> &#8942; </span>
                                        </button>
                                    </div>
                                    <h3>${file.realname}</h3>
                                    <div class=\"options\" style=\"display:none; z-index: 10\">
                                        <div style=\"margin:0 10px 0 0;\">
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DELETE\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.name}\">
                                                <button type=\"submit\"> Delete </button>
                                            </form>
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DOWNLOAD\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.name}\">
                                                <button type=\"submit\"> DOWNLOAD </button>
                                            </form>
                                        </div>
                                        <video width=\"300\" height =\"240\" controls>
                                            <source src=\"${file.u_path}\" type=\"${file.f_type}\">
                                            Your browser doesnt support my video :'(
                                        </video>
                                    </div>
                                </li>
                                `
                            } else if (file.file_type == "text") {
                                return ` 
                                <li class="file-card">
                                    <div class="card-top">
                                        <span class="file-icon">≡</span>
                                        <button class=\"options_file\" id=\"${file.id}\" onclick=\"open_file_options(this)\">
                                            <span> &#8942; </span>
                                        </button>
                                    </div>
                                    <h3>${file.realname}</h3>
                                    <div class=\"options\" style=\"display:none; z-index: 10\">
                                        <div style=\"margin:0 10px 0 0;\">
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DELETE\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.name}\">
                                                <button type=\"submit\"> Delete </button>
                                            </form>
                                            <form action=\"/\" method =\"POST\">
                                                <input type=\"hidden\" name=\"**action\" value=\"DOWNLOAD\">
                                                <input type=\"hidden\" name=\"filename\" value=\"${file.name}\">
                                                <button type=\"submit\"> DOWNLOAD </button>
                                            </form>
                                        </div>
                                    </div>
                                </li>
                                `
                            }
                        } else {
                            return `
                            <li class="file-card folder-card">
                                <div class="card-top">
                                    <span class="file-icon"><span class="folder-icon" aria-hidden="true"></span></span>
                                    <button class="options_folder" id="${file.id}" onclick="open_folder_options(this)">
                                        <span> &#8942; </span>
                                    </button>
                                </div>
                                <h3>${file.realname}</h3>
                                <div class="options" style="display: none; z-index: 10">
                                    <div style="display: block;  margin:0 10px 0 0;">
                                        <form action="/" method ="POST">
                                            <input type= "hidden" name= "**action" value= "DELETE">
                                            <input type= "hidden" name= "filename" value= "${file.name}">
                                            <button type= "submit">Delete</button>
                                        </form>
                                        <form action="/" method ="POST">
                                            <input type= "hidden" name= "**action" value= "RENAME_FOLDER">
                                            <input type= "hidden" name= "filename" value= "${file.name}">
                                            <input type= "text\" name= "newFile">
                                            <button type= "submit">Rename</button>
                                        </form>
                                        <form action="/" method="POST">
                                            <input type= "hidden" name= "**action" value= "DOWNLOAD_FOLDER">
                                            <input type= "hidden" name= "filename" value= "${file.name}">
                                            <button type= "submit">Download as ZIP</button>
                                        </form>
                                    </div>
                                    <form action="/" method= "POST">
                                        <input type= "hidden" name= "**action" value= "OPEN_FOLDER">
                                        <input type= "hidden" name= "filename" value= "${file.name}">
                                        <button type= "submit">Open Folder</button>
                                    </form> 
                                </div>
                            </li>
                            `
                        }
                    }).join('');
                    filesWindow.innerHTML = `
                        ${folders.length ? `<section class="file-section"><h3>Folders</h3><div class="file-grid">${renderItems(folders)}</div></section>` : ''}
                        ${files.length ? `<section class="file-section"><h3>Files</h3><div class="file-grid">${renderItems(files)}</div></section>` : ''}
                    `;
                }
            }
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

            html {
                color-scheme: dark;
            }
            body{
                color : #d1cfccfc;
            }
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

            <style>
                html {
                    color-scheme: dark;
                }
                body{
                    color : #d1cfccfc;
                }
            </style>
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
            <style>
                html {{
                    color-scheme: dark;
                }}
                body{{
                    color : #d1cfccfc;
                }}
            </style>
            <body>
                <h1> Welcome to your File Manager Server </h1>

                <h3> Enter the password for your account<h3>

                <h4> Password: </h4>
            <form action=\"/\" method=\"POST\">
                <input type=\"hidden\" name=\"**account\" value=\"{}\">
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

    pub fn new_user(name: String ) -> String{

        let mut html = String::from(format!("
            <!DOCTYPE html>
            <html>
            <head>
            <title> Set up ur account</title>
            </head>
            <style>
                html {{
                    color-scheme: dark;
                }}
                body{{
                    color : #d1cfccfc;
                }}
            </style>
            <body>
                <h1> Welcome to your File Manager Server </h1>

                <h3> Enter some info for your account <h3>

                <form action=\"/\" method=\"POST\" enctype=\"multipart/form-data\">

                    <input type=\"hidden\" name=\"**account\" value=\"{}\">

                    <h4> Password: </h4>
                    <input type=\"text\" name=\"**password\" required>
                    
                    <p> Choose your gender </p> 
                    <select id=\"gender\" name=\"gender\">
                        <option value=\"SMTH\">Walmart Bag</option>
                        <option value=\"M\">Male</option>
                        <option value=\"F\">Female</option>
                    </select>

                    <p> Profile picture (it will be cropped in a 1:1 aspect ratio in the middle so choose wisely)
                    <input type=\"file\" name=\"file\" required>
                    
                    
                    <button type=\"submit\"> Finish the Sign up </button>
                </form>
            <br><br>

            
        ",
        name
        ));

        html.push_str("
            </body>
            </html>
        ");
        html
    }