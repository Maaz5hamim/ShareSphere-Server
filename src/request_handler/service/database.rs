use std::io::{ErrorKind,Error, stdout};
use tokio::fs;
use sqlx::{mysql::{MySqlPoolOptions, MySqlRow}, Row};
use crate::request_handler::{my_files, my_folders, files};


pub async fn register_user(name: &str, email: &str, password: &str) -> Result<(),sqlx::Error> 
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    //inserting user info in user table
    let result = sqlx::query
    (
        "INSERT INTO user (name, email, password) VALUES (?, ?, ?)"
    )
    .bind(name)
    .bind(email)
    .bind(password)
    .execute(& pool)
    .await?;
    Ok(())
}

pub async fn verify_user(email: &str, password: &str) -> Result<(),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    // Query the password for the given email
    let row = sqlx::query
    (
        "SELECT password FROM user WHERE email = ?"
    )
    .bind(email)
    .fetch_optional(&pool)
    .await?;

    if let Some(row) = row 
    {
        // Compare the stored password with the given password
        let stored_password: String = row.get(0);
        if stored_password == password 
        {
            // Passwords match  
            return Ok(());
        }
    } 
    Err(sqlx::Error::Io(Error::new(ErrorKind::Other, "Invalid email or password")))
}

pub async fn add_file(email: &str, file_name: &str, file_path:&str, size:usize) -> Result<(),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    // Query to check if a file already exists
    let row = sqlx::query
    (
        "SELECT file_id FROM File WHERE name = ? AND owner_id = ?"
    )
    .bind(file_name)
    .bind(email)
    .fetch_optional(&pool)
    .await.unwrap();

    if let Some(row) = row 
    {
        return Ok(());
    } 
    else 
    {
        //inserting file info in file table otherwise
        let result = sqlx::query
        (
            "INSERT INTO file (name, path, owner_id, size) VALUES (?, ?, ?, ?)"
        )
        .bind(file_name)
        .bind(file_path)
        .bind(email)
        .bind(size as u32)
        .execute(& pool)
        .await?;

        Ok(())    
    }


}

pub async fn delete(file_name:&str, email: &str) -> Result<(),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    let path = format!("D://ShareSphere//{email}//{file_name}");

    //removing file from server directory
    fs::remove_file(path).await?;

    //deleting file info in file table
    let result = sqlx::query
    (
        "DELETE FROM File WHERE name = ? AND owner_id = ?"
    )
    .bind(file_name)
    .bind(email)
    .execute(& pool)
    .await?;

    Ok(())
}

pub async fn file_names(email: &str, folder_name:&str) -> Result<(Vec<files>),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    let list = sqlx::query
    (
        "select file.name, file.size, DATE_FORMAT(file.upload_date, '%Y-%m-%d')  AS date from file JOIN shared_with on file.file_id = shared_with.file_id JOIN user on file.owner_id = user.email where user.name = ? AND shared_with.email = ?"
    )
    .bind(folder_name)
    .bind(email)
    .fetch_all(&pool)
    .await?
    .iter()
    .map(|row: &MySqlRow| files 
    {
        name: row.get("name"),
        size: row.get("size"),
        upload_date: row.get("date"),
    })
    .collect::<Vec<files>>();

    Ok(list)
}

pub async fn folder_names(email: &str) -> Result<(Vec<my_folders>),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;
    let list = sqlx::query
    (
        "Select user.name, count(file.upload_date) as Items, DATE_FORMAT(MAX(file.upload_date), '%Y-%m-%d') AS Updated from file JOIN shared_with on file.file_id = shared_with.file_id JOIN user on user.email = file.owner_id where shared_with.email = ? GROUP BY user.name;"
    )
    .bind(email)
    .fetch_all(&pool)
    .await?
    .iter()
    .map(|row: &MySqlRow| my_folders {
        name: row.get("name"),
        items: row.get("Items"),
        updated: row.get("Updated"),
    })
    .collect::<Vec<my_folders>>();

    Ok(list)
}

pub async fn upload_list(email: &str) -> Result<(Vec<my_files>),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    let list = sqlx::query
    (
        "SELECT file.name AS Title, file.size AS Size, DATE_FORMAT(upload_date, '%Y-%m-%d')  AS Date, GROUP_CONCAT(shared_with.email) AS Members
        FROM file LEFT JOIN shared_with  ON  shared_with.file_id = file.file_id where file.owner_id = ? GROUP BY file.file_id"
    )
    .bind(email)
    .fetch_all(&pool)
    .await?
    .iter()
    .map(|row: &MySqlRow| my_files {
        name: row.get("Title"),
        size: row.get("Size"),
        upload_date: row.get("Date"),
        members:row.get("Members"),
    })
    .collect::<Vec<my_files>>();

    Ok(list)

}

pub async fn share_file(email: &str, members: Vec<String>, filename: &str) -> Result<(),sqlx::Error>
{
    //connecting to database
    let pool = MySqlPoolOptions::new()
    .max_connections(2)
    .connect("mysql://root:Sheikh@6229@localhost/sharesphere").await?;

    // Query to check if a file already exists
    let row = sqlx::query
    (
        "SELECT file_id FROM File WHERE name = ? AND owner_id = ?"
    )
    .bind(filename)
    .bind(email)
    .fetch_optional(&pool)
    .await?;

    if let Some(row) = row
    {
        let file_id: i32 = row.get(0);
        for member in members
        {
            //inserting file info in file table otherwise
            let result = sqlx::query
            (
                "INSERT INTO shared_with (file_id, email) VALUES (?, ?)"
            )
            .bind(&file_id)
            .bind(member)
            .execute(& pool)
            .await?;
        }
    }
    Ok(())    


}