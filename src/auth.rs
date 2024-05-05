use crate::schemas::UserNick;
use actix_web::{http::header::HeaderValue, HttpRequest};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{env, io::Read, num::ParseIntError};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, PartialEq)]
pub enum AuthorizationLevel {
    Bot,
    Frontend(UserNick),
}

#[derive(Deserialize, Debug, Clone)]
struct TelegramAuthData {
    auth_date: String,
    first_name: Option<String>,
    last_name: Option<String>,
    id: String,
    photo_url: Option<String>,
    username: String,
    hash: String,
}

pub fn check_authorization_level(request: HttpRequest) -> Option<AuthorizationLevel> {
    let authorization = request
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .map(HeaderValue::to_str)?
        .ok()?;
    let bot_token = env::var("BOT_API_TOKEN").unwrap();
    if authorization == bot_token {
        return Some(AuthorizationLevel::Bot);
    }
    let auth_data: TelegramAuthData = match serde_json::from_str(authorization) {
        Ok(json) => json,
        Err(_) => return Some(AuthorizationLevel::Bot),
    };
    let hash = auth_data
        .hash
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|n| u8::from_str_radix(&String::from_iter(n), 16))
        .collect::<Result<Vec<u8>, ParseIntError>>()
        .ok()?;
    let computed_hash = compute_hash(auth_data.clone(), bot_token);
    if computed_hash == hash {
        Some(AuthorizationLevel::Frontend(auth_data.username))
    } else {
        None
    }
}

fn compute_hash(auth_data: TelegramAuthData, bot_token: String) -> Vec<u8> {
    let hash_content = vec![
        ("auth_date", Some(auth_data.auth_date)),
        ("first_name", auth_data.first_name),
        ("id", Some(auth_data.id)),
        ("last_name", auth_data.last_name),
        ("photo_url", auth_data.photo_url),
        ("username", Some(auth_data.username)),
    ]
    .into_iter()
    .filter_map(|pair| pair.1.map(|val| format!("{}={}", pair.0, val)))
    .collect::<Vec<_>>();
    let hash_content = hash_content.join("\n");
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(bot_token.as_bytes());
    let bot_hash = sha256_hasher.finalize();

    let mut hmac_hasher = HmacSha256::new_from_slice(&bot_hash).unwrap();
    hmac_hasher.update(hash_content.as_bytes());
    let computed_hash = hmac_hasher
        .finalize()
        .into_bytes()
        .bytes()
        .filter_map(|a| a.ok())
        .collect::<Vec<u8>>();
    computed_hash
}
