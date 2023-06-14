use std::io::{ErrorKind,Error};

use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::fs::{self,File};

use crate::request_handler::{Response,my_files, my_folders, files};
use database::*;

mod database;

pub async fn authenticate(email: &str,password: &str) -> io::Result<()> 
{
    match verify_user(email, password).await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

pub async fn add_user(email: &str, name: &str, password: &str) -> io::Result<()> 
{
    match register_user(name, email, password).await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

pub async fn delete_file(file_name:&str, email: &str) -> io::Result<()> 
{
    match delete(file_name, email).await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

pub async fn upload_file(stream: &mut TcpStream, filename: &str, email:&str) -> io::Result<()> 
{
    let confirmation = Response::Initiate;
    let request_bytes = bincode::serialize(&confirmation).unwrap();
    stream.write_all(&request_bytes).await?;

    //create a new folder in server directory for user if doesnt exist
    fs::create_dir(format!("D://ShareSphere//{email}")).await;
    //create a new file in user's folder
    let filepath = format!("D://ShareSphere//{email}//{filename}");
    let mut file = File::create(&filepath).await?;
    // Read the file contents from the client
    let mut size = 0;
    let mut buffer = [0; 10000];
    loop 
    {
        let bytes_read = stream.read(&mut buffer).await?;
        size += bytes_read;
        if bytes_read == 0 
        {
            // all contents read
            break;
        }
        // Save the file contents to file created
        file.write_all(&mut buffer[..bytes_read]).await?;
    }

    println!("File received and saved to D://ShareSphere//{email}//{filename}");


    match add_file(email,filename, &filepath, size).await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }

}

pub async fn download_file(stream:&mut TcpStream, filename: &str, email: &str) -> io::Result<()> 
{
    //sending confirmation to client of acceptance of request
    let confirmation = Response::Initiate;
    let request_bytes = bincode::serialize(&confirmation).unwrap();
    stream.write_all(&request_bytes).await?;

    let filepath = format!("D://ShareSphere//{email}//{filename}");

    let mut file = File::open(filepath).await?;

    // Read the file in chunks and send it over the network
    let mut buffer = [0; 10000];
    loop 
    {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0
        {
            // End of file reached
           break;
        }
        stream.write_all(&buffer[..bytes_read]).await?;
    }
    
    Ok(())
}

pub async fn get_file_name(email: &str, folder_name:&str) -> io::Result<Vec<files>> 
{
    match file_names(email, folder_name).await
    {
        Ok(list) => Ok(list),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

pub async fn get_folder_name(email: &str) -> io::Result<Vec<my_folders>> 
{

    match folder_names(email).await
    {
        Ok(list) => Ok(list),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

pub async fn my_upload_list(email: &str) -> io::Result<Vec<my_files>>  
{
    match upload_list(email).await
    {
        Ok(list) => Ok(list),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

pub async fn share(email: &str, members: Vec<String>, filename: &str) -> io::Result<()>  
{
    match share_file(email, members, filename).await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}
