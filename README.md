This is my http_file_server, made by me, debugged by me, researched by me, and improved by me
this is still a work in progress, but if u are interested in helping with making this better or adding a function for it, 
feel free to copy the repository and when u are done push to a branch, im going to review your changes

otherwise the server is quite simple

it just listens to "127.0.0.1:7878" for a client and then it sends a webpage to the client
it is intended to be used on a windows server where it stores the files and displays the stored files on itself
it got quite a simple UI but it needs to be imporoved

(idk what else to say)

in the future im planning to implement:
  - better security (still working on it)
  - input sanitization and validation (e.g. %20 -> " ")

  - file search and filtering
  - session management (so you can be connected on multiple devices)
  - file sharing & public links (or links to directly download)
  - a database config so i dont have to use my current version of storing file data
  - better file metadata
  - background jobs
  - API endpoints (for android ig and other clients)

  - user roles and permisions
  - maybe storage backend (so it does automatical backups on user files either from google photos or directly from phone)