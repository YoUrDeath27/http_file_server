imma be honest, idk what .github/workflow does

anyway, this is my http_file_server, made by me, debugged by me, researched by me, and improved by me
this is still a wrok in progress, but if u are interested in helping with making this better or adding a function for it, 
feel free to copy the repository and when u are done to push to a branch, im going to review your changes, and i will decide wether its worth it or not

otherwise the server is quite simple

it just listens to "127.0.0.1:7878" for a client and then it sends a webpage to him
(idk what else to say)

in the future im planning to implement:
  - multi-threading
  - rename file function
  - better UI (styling)
  - folders (kinda done)   -but nested folders (folder> folder> file) dont work atm, gotta work on that
  - make so that when u do a GET/{folder} to make u enter that folder and view its contents, and somewhere up to get u back at root (/)
    
