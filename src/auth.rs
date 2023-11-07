use crate::blog::{UserID, STORE_PATH};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

// dummy unit tuple so `Auth`s can't be instantiated outside of this file
pub struct Auth(());

async fn read_logins() -> std::io::Result<HashMap<UserID, String>> {
    // TODO: improve the security of storing usernames and passwords
    let file = tokio::fs::File::open(std::path::Path::new(STORE_PATH).join("logins.txt")).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut reader = reader.lines();

    let mut logins = HashMap::new();
    while let Some(line) = reader.next_line().await? {
        let (username, hash) = match line.split_once('\t') {
            Some(pair) => pair,
            None => continue,
        };
        logins.insert(String::from(username), String::from(hash));
    }

    Ok(logins)
}

impl Auth {
    /// `Ok(Some(Auth))` if valid, `Ok(None)` if invalid, `Err` if logins.txt
    /// could not be read/bcrypt verifying failed
    pub async fn validate(
        id: &UserID,
        password: String,
    ) -> Result<Option<Auth>, Box<dyn std::error::Error>> {
        let logins_file = read_logins().await?;

        let Some(hash) = logins_file.get(id).cloned() else {
            return Ok(None);
        };

        let password_is_valid =
            tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
                .await
                .expect("task should not panic")?;

        if password_is_valid {
            Ok(Some(Auth(())))
        } else {
            Ok(None)
        }
    }

    /// `Ok(Some(Auth))` if created, `Ok(None)` if the user id already exists, `Err` if
    /// logins.txt could not be read/bcrypt hashing failed
    pub async fn write_entry(
        id: &UserID,
        password: String,
    ) -> Result<Option<Auth>, Box<dyn std::error::Error>> {
        let logins_file = read_logins().await?;
        if logins_file.get(id).is_some() {
            return Ok(None);
        }

        let hash = tokio::task::spawn_blocking(|| bcrypt::hash(password, bcrypt::DEFAULT_COST))
            .await
            .expect("task should not panic")?;

        tokio::fs::OpenOptions::new()
            .append(true)
            .open(std::path::Path::new(STORE_PATH).join("logins.txt"))
            .await?
            .write_all(format!("\n{id}\t{hash}").as_bytes())
            .await?;

        Ok(Some(Auth(())))
    }
}
