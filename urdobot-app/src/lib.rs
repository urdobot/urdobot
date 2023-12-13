use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
};

use anyhow::{Context, Result};
use lol_html::{element, HtmlRewriter, Settings};
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserData {
    #[serde(rename = "userID", with = "string_as_number")]
    pub user_id: u32,
    #[serde(rename = "factionID", with = "string_as_number")]
    pub faction_id: u16,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "pid", with = "string_as_number")]
    pub instance_id: u32,
}

mod string_as_number {
    use std::{fmt::Display, str::FromStr};

    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
    where
        T: FromStr,
        <T as FromStr>::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<T>()
            .map_err(serde::de::Error::custom)
    }
}

pub async fn hello() -> Result<UserData> {
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .user_agent("BigpointClient/1.6.9")
        .build()?;

    let html = client
        .get("https://darkorbit.com/index.es?lang=en")
        .send()
        .await?
        .text()
        .await?;

    let mut file = std::fs::File::create("./index.html")?;
    file.write_all(html.as_bytes())?;

    let mut action = String::new();

    let mut parser = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!(r#"[name="bgcdw_login_form"]"#, |e| {
                action += &e.get_attribute("action").context("No attribute 'action'")?;
                Ok(())
            })],
            ..Default::default()
        },
        |_: &[u8]| {},
    );
    parser.write(html.as_bytes())?;
    parser.end()?;

    let action = html_escape::decode_html_entities(&action).to_string();

    let mut username = String::new();
    print!("Username: ");
    stdout().flush()?;
    stdin().read_line(&mut username)?;

    let mut password = String::new();
    print!("Password: ");
    stdout().flush()?;
    stdin().read_line(&mut password)?;

    let username = username.trim();
    let password = password.trim();

    let mut form = HashMap::new();
    form.insert("username", username);
    form.insert("password", password);
    let res = client.post(action).form(&form).send().await?;
    let host = res.url().host().context("No host")?.to_string();
    let html = res.text().await?;
    let mut file = std::fs::File::create("./account.html")?;
    file.write_all(html.as_bytes())?;

    let html = client
        .get(format!(
            "https://{host}/indexInternal.es?action=internalMapRevolution"
        ))
        .send()
        .await?
        .text()
        .await?;
    let mut file = std::fs::File::create("./revolution.html")?;
    file.write_all(html.as_bytes())?;

    let re = Regex::new(r#"flashembed\("container", .*?, (\{.*\})"#).unwrap();
    let data = re
        .captures(&html)
        .and_then(|c| c.get(1))
        .context("No login data found")?
        .as_str();

    Ok(serde_json::from_str(data)?)
}
