# RustDrive
RustDrive is a client-server file storage system written in Rust.

The project provides file upload and download, user authentication, session management, metadata management, file search, access control, AI-based file categorization, and download/upload progress bars.

---

## Features

- User registration and login
- Session management
- File upload
- File download
- File deletion
- File renaming
- File listing
- File search
- Public / Private file access
- Download access control
- File metadata management
- AI-based file categorization
- Upload progress bar
- Download progress bar
- Automatic unique filenames when downloading duplicate files
- TCP client-server communication
- Asynchronous I/O with Tokio

---

## Architecture

RustDrive is divided into three Rust crates:

```text
RustDrive/
│
├── client/
│   └── Client application
│
├── server/
│   └── Server application
│
└── common/
    └── Shared models and communication protocol