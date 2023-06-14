use tokio::io::{self,AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde::{Deserialize, Serialize};

pub mod service;
use service::*;

#[derive(Debug, Serialize)]
pub struct files
{
    name: String,
    size: i32,
    upload_date: String,
}

#[derive(Debug, Serialize)]
pub struct my_files
{
    name: String,
    size: i32,
    upload_date: String,
    members : Option<String>,
}

#[derive(Debug, Serialize)]
pub struct my_folders
{
    name: String,
    items: i32,
    updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request
{
    service:RequestType,
    email:String

}
#[derive(Debug, Serialize, Deserialize)]
pub enum RequestType 
{
    Login { password: String },
    Signup{name:String, password:String},
    UploadFile { filename: String },
    DownloadFile { filename: String, file_path: String},
    DeleteFile {file_name: String},
    MyUploadList,
    GetFileNames {folder_name: String},
    GetFolderNames,
    Share {members: Vec<String>, filename:String},
}
#[derive(Debug, Serialize)]
pub enum Response 
{
    Success,
    Failure(String),
    GetFileNameSuccess { filenames: Vec<files> },
    GetFolderNameSuccess { foldernames: Vec<my_folders>},
    MyUploadListSuccess { list: Vec<my_files>},
    Initiate,
}

pub async fn handle_request(stream: &mut TcpStream) -> io::Result<()> 
{
    println!("New client connected: {:?}", stream.peer_addr().unwrap());

    let mut buffer = [0; 512];
    let bytes_read = stream.read(&mut buffer).await?;
    let request: Request = bincode::deserialize(&buffer[..bytes_read]).unwrap();

    match request.service 
    {
        RequestType::Login {password } => 
        {
            let result = authenticate(&request.email, &password).await;

            let response = match result 
            {
                Ok(()) => Response::Success,
                Err(e) => Response::Failure(e.to_string()),
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::Signup {name,password } => 
        {
            let result = add_user(&request.email, &name, &password).await;

            let response = match result 
            {
                Ok(()) => Response::Success,
                Err(e) => Response::Failure(e.to_string()),
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::UploadFile { filename} => 
        {
            let result = upload_file(stream, &filename, &request.email).await;

            let response = match result 
            {
                Ok(()) => Response::Success,
                Err(e) => Response::Failure(e.to_string()),
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::DownloadFile { filename, file_path } => 
        {
            let result = download_file(stream, &filename, &request.email).await;

            let response = match result 
            {
                Ok(()) => Response::Success,
                Err(e) => Response::Failure(e.to_string()),
            };

            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::DeleteFile { file_name } => 
        {
            let result = delete_file(&file_name, &request.email).await;

            let response = match result 
            {
                Ok(()) => Response::Success,
                Err(e) => Response::Failure(e.to_string()),
            };

            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::GetFileNames { folder_name } => 
        {
            let result = get_file_name(&request.email, &folder_name).await;

            let response = match result 
            {
                Ok(filenames) => Response::GetFileNameSuccess { filenames },
                Err(e) => Response::Failure(e.to_string()),
            };

            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::GetFolderNames => 
        {
            let result = get_folder_name(&request.email).await;

            let response = match result 
            {
                Ok(foldernames) => Response::GetFolderNameSuccess { foldernames },
                Err(e) => Response::Failure(e.to_string()),
            };

            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::MyUploadList => 
        {
            let result = my_upload_list(&request.email).await;

            let response = match result 
            {
                Ok(list) => Response::MyUploadListSuccess { list },
                Err(e) => Response::Failure(e.to_string()),
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }
        RequestType::Share { members, filename } => 
        {
            let result = share(&request.email, members, &filename).await;

            let response = match result 
            {
                Ok(_) => Response::Success,
                Err(e) => Response::Failure(e.to_string()),
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).await?;
        }  
    }
    stream.shutdown().await?;

    Ok(())
}
