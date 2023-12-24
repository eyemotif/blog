use crate::blog::STORE_PATH;
use argon2::{PasswordHasher, PasswordVerifier};
use rand::SeedableRng;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

// dummy unit tuple so `Auth`s can't be instantiated outside of this file
pub struct Auth(());

async fn read_logins() -> std::io::Result<HashMap<String, String>> {
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

fn hash_password(password: &str) -> argon2::password_hash::Result<String> {
    let salt =
        argon2::password_hash::SaltString::generate(&mut rand_chacha::ChaCha20Rng::from_entropy());

    Ok(argon2::Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}
fn verify_password(password: &str, hash: &str) -> argon2::password_hash::Result<bool> {
    let hash = argon2::password_hash::PasswordHash::new(hash)?;

    match argon2::Argon2::default().verify_password(password.as_bytes(), &hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(err) => Err(err),
    }
}

impl Auth {
    /// `Ok(Some(Auth))` if valid, `Ok(None)` if invalid, `Err` if logins.txt
    /// could not be read/argon2 verifying failed
    pub async fn validate(
        username: &str,
        password: String,
    ) -> Result<Option<Auth>, Box<dyn std::error::Error>> {
        let logins_file = read_logins().await?;

        let Some(hash) = logins_file.get(username).cloned() else {
            return Ok(None);
        };

        let password_is_valid =
            tokio::task::spawn_blocking(move || verify_password(&password, &hash))
                .await
                .expect("task should not panic")?;

        if password_is_valid {
            Ok(Some(Auth(())))
        } else {
            Ok(None)
        }
    }

    /// `Ok(Some(Auth))` if created, `Ok(None)` if the user id already exists, `Err` if
    /// logins.txt could not be read/argon2 hashing failed
    pub async fn write_entry(
        username: &str,
        password: String,
    ) -> Result<Option<Auth>, Box<dyn std::error::Error>> {
        let logins_file = read_logins().await?;
        if logins_file.get(username).is_some() {
            return Ok(None);
        }

        let hash = tokio::task::spawn_blocking(move || hash_password(&password))
            .await
            .expect("task should not panic")?;

        tokio::fs::OpenOptions::new()
            .append(true)
            .open(std::path::Path::new(STORE_PATH).join("logins.txt"))
            .await?
            .write_all(format!("\n{username}\t{hash}").as_bytes())
            .await?;

        Ok(Some(Auth(())))
    }
}
