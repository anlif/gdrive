use crate::config;
use crate::hub;
use std::io;
use std::io::Write;

pub async fn add() -> Result<(), Error> {
    let secret = secret_prompt().map_err(Error::Prompt)?;

    let tmp_dir = tempfile::tempdir().map_err(Error::Tempdir)?;
    let tokens_path = tmp_dir.path().join("tokens.json");

    let auth = hub::Auth::new(&secret, &tokens_path)
        .await
        .map_err(Error::Auth)?;

    // Get access tokens
    auth.token(&[
        "https://www.googleapis.com/auth/drive",
        "https://www.googleapis.com/auth/drive.metadata.readonly",
    ])
    .await
    .map_err(Error::AccessToken)?;

    let hub = hub::Hub::new(auth).await;
    let (_, about) = hub
        .about()
        .get()
        .param("fields", "user")
        .doit()
        .await
        .map_err(Error::About)?;

    let email = about
        .user
        .and_then(|u| u.email_address)
        .unwrap_or_else(|| String::from("unknown"));

    let config = config::add_account(&email, &secret, &tokens_path).map_err(Error::Config)?;
    config::switch_account(&config).map_err(Error::Config)?;

    println!("Logged in as {}", config.account.name);

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Prompt(io::Error),
    Tempdir(io::Error),
    Config(config::Error),
    Auth(io::Error),
    AccessToken(google_drive3::oauth2::Error),
    About(google_drive3::Error),
}

fn secret_prompt() -> Result<config::Secret, io::Error> {
    println!("A client id and client secret are required to use this application.");
    println!();

    let client_id = prompt_input("Client ID")?;
    let client_secret = prompt_input("Client secret")?;

    Ok(config::Secret {
        client_id,
        client_secret,
    })
}

fn prompt_input(msg: &str) -> Result<String, io::Error> {
    print!("{}: ", msg);
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
