fn chat(chat: String, user: &str) -> String {
    println!("chat = {:?}", chat);

    let mut html = String::from(
        r#"<!DOCTYPE html>
    <html>
    <head>
        <title>Chat</title>
        <meta charset="utf-8">
    </head>
    <body>
        <h2>Welcome to the chat!</h2>
        <img src="img=1-1" id="1" alt="testing" width="200" height="200">

        <form action="/" method="POST">
            <input type="hidden" name="exit_chat">
            <button type="submit">Go back to browse the chat rooms</button>
        </form>

        <ul id="chat-window"></ul>

        <form id="chatForm" method="POST">
            <input type="text" placeholder="Enter a message to send in chat" 
                   name="input_message" id="inputMessage">
            <button type="submit">Send message</button>
        </form>

        <form id="file_upload" method="POST" enctype="multipart/form-data">
            <input type="file" name="file" id="file">
            <button type="submit">Upload file</button>
        </form>
    "#
    );

    // Add the JavaScript with SSE implementation
    html.push_str(&format!(r#"
    <script>
    const chatWindow = document.getElementById('chat-window');
    const currentUser = '{}';

    // Initialize SSE connection when DOM is loaded
    document.addEventListener('DOMContentLoaded', function() {{
        setupSSE();
        loadInitialMessages();
    }});

    // Set up Server-Sent Events connection
    function setupSSE() {{
        const eventSource = new EventSource('/messages');
        
        eventSource.onmessage = function(e) {{
            const messages = JSON.parse(e.data);
            updateChatWindow(messages);
        }};
        
        eventSource.onerror = function(e) {{
            console.error('SSE Error:', e);
            eventSource.close();
            // Reconnect after 1 second
            setTimeout(setupSSE, 1000);
        }};
    }}

    // Load initial messages when page loads
    async function loadInitialMessages() {{
        try {{
            const response = await fetch('/messages');
            if (!response.ok) throw new Error('Network response was not ok');
            const messages = await response.json();
            updateChatWindow(messages);
        }} catch (error) {{
            console.error('Error loading initial messages:', error);
        }}
    }}

    // Update the chat window with new messages
    function updateChatWindow(messages) {{
        chatWindow.innerHTML = messages.map(msg => {{
            if (msg.is_deleted) {{
                return `
                <li>
                    <p style="color: ${{msg.color}}"> 
                        ${{msg.name}}
                    </p>
                    <h4>Message has been deleted</h4>
                </li>`;
            }}
            else if (msg.attachments[0].file_path == "") {{
                return `
                <li>
                    <p style="color: ${{msg.color}}"> 
                        ${{msg.name}}
                    </p>
                    <h4>${{msg.message}}</h4>
                    ${{msg.name === currentUser ? 
                    `<form action="/" method="POST">
                        <input type="hidden" name="remove_message" value="${{msg.id}}">
                        <button type="submit">Delete message</button>
                    </form>` : ''}}
                </li>`;
            }}
            else if (msg.message == "" && msg.attachments[0].file_path.startsWith("video")) {{
                return format!(r#"
                <li>
                    <p style="color: ${{msg.color}}"> 
                        ${{msg.name}}
                    </p>
                    <video width="320" height="240" controls>
                        <source src="img=${{msg.attachments[0].message_id}}-${{msg.attachments[0].id}}" 
                                type="video/mp4" alt="${{msg.attachments[0].file_name}}">
                        Your browser does not support the video tag.
                    </video>
                    ${{msg.name === currentUser ? 
                    `<form action="/" method="POST">
                        <input type="hidden" name="remove_message" value="${{msg.id}}">
                        <button type="submit">Delete message</button>
                    </form>` : ''}}
                </li>"#)   
                //the user ig
            )
            else {{
                return `
                <li>
                    <p style="color: ${{msg.color}}"> 
                        ${{msg.name}}
                    </p>
                    <img src="img=${{msg.attachments[0].message_id}}-${{msg.attachments[0].id}}" 
                         alt="${{msg.attachments[0].file_name}}" width="320" height="240">
                    ${{msg.name === currentUser ? 
                    `<form action="/" method="POST">
                        <input type="hidden" name="remove_message" value="${{msg.id}}">
                        <button type="submit">Delete message</button>
                    </form>` : ''}}
                </li>`;
            }}
        }}).join('');
        scrollToBottom();
    }}

    function scrollToBottom() {{
        chatWindow.scrollTop = chatWindow.scrollHeight;
    }}

    // Message sending functions remain the same
    document.getElementById("chatForm").addEventListener("submit", function(event) {{
        event.preventDefault();
        const input = document.getElementById("inputMessage");
        const inputMessage = input.value;
        input.value = "";
        send_message(inputMessage);
    }});
        
    async function send_message(message) {{
        const data = {{ input_message: message }};
        
        try {{
            const response = await fetch('/enter_message', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify(data)
            }});
            if (!response.ok) throw new Error('Network response was not ok');
        }} catch (error) {{
            console.error('Error:', error);
        }}
    }}

    document.getElementById("file_upload").addEventListener("submit", function(event) {{
        event.preventDefault();
        const button = event.target.querySelector("button[type=submit]");
        button.disabled = true;
        const file = document.getElementById("file");
        
        if (file.files.length > 0) {{
            send_file_message(file.files[0]).finally(() => {{
                button.disabled = false;
                file.value = "";
            }});
        }} else {{
            console.log("No file selected");
            file.value = "";
        }}
    }});
        
    async function send_file_message(file) {{
        const data = new FormData();
        data.append('file', file);
        
        try {{
            const response = await fetch('/enter_file_message', {{
                method: 'POST',
                body: data
            }});
            if (!response.ok) throw new Error('Network response was not ok');
        }} catch (error) {{
            console.error('Error:', error);
        }}
    }}
    </script>
    </body>
    </html>
    "#, user));

    let cookie = format!(
        "Set-Cookie: Chat_room=\"chats/{}.db\"; Path=/; HttpOnly; SameSite=Strict\r\n\r\n",
        chat
    );

    format!("{}{}", cookie, html)
}