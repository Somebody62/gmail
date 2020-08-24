use chrono::prelude::*;
use email_format::Email;
use hyper::{Body, Client, Method, Request};
use serde_json::{json, Value};
use hyper_alpn::AlpnConnector;

use std::collections::HashMap;
use std::fs::File;

async fn make_email_req(body: String, auth: &str) -> String {
    let request = Request::builder()
        .method(Method::POST)
        .uri("https://www.googleapis.com/gmail/v1/users/me/messages/send")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", auth))
        .body(Body::from(body))
        .unwrap();
    make_post_req(request).await
}

async fn make_form_req(hash: HashMap<&str, &str>) -> String {
    let mut body = String::new();
    for (name, value) in hash {
        body += &format!("{}={}&", name, value);
    }
    if body.len() > 0 {
        body = body[0..body.len() - 1].to_string();
    }
    let built_req = Request::builder()
        .method(Method::POST)
        .uri("https://www.googleapis.com/oauth2/v4/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    make_post_req(built_req).await
}

async fn make_post_req(req: Request<Body>) -> String {
    let client = &Client::builder().http2_only(true).build(AlpnConnector::new());
    let resp = client.request(req).await.unwrap();
    String::from_utf8(
        hyper::body::to_bytes(resp.into_body())
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

pub async fn send_email(addresses: Vec<String>, subject: &str, body: &str, auth: &str) -> String {
    let mut email = Email::new("justus@olmmcc.tk", Utc::now().to_rfc2822().as_str()).unwrap();
    let mut to_string = String::new();
    for i in 0..addresses.len() {
        to_string += &format!("{},", addresses[i]);
    }
    if addresses.len() > 1 {
        email.set_bcc(&to_string[0..to_string.len() - 1]).unwrap();
        email.set_to("justus@olmmcc.tk").unwrap();
    } else if addresses.len() == 1 {
        email.set_to(&to_string[0..to_string.len() - 1]).unwrap();
    } else {
        return "Error! No to email address specified".to_string();
    }
    email.set_subject(subject).unwrap();
    email.set_body(body).unwrap();
    let raw = base64::encode(&email.as_bytes());
    let body = json!({ "raw": raw }).to_string();
    make_email_req(body, auth).await
}

pub async fn get_refresh_token(code: &str) -> String {
    let file = File::open("/home/justus/client_secret.json").unwrap();
    let json: Value = serde_json::from_reader(file).unwrap();
    let mut hash = HashMap::new();
    hash.insert("code", code);
    hash.insert("access_type", "offline");
    hash.insert("client_id", json["client_id"].as_str().unwrap());
    hash.insert("client_secret", json["client_secret"].as_str().unwrap());
    hash.insert("redirect_uri", "https://www.olmmcc.tk/admin/email/");
    hash.insert("grant_type", "authorization_code");
    let request = make_form_req(hash).await;
    let request_json: Value = serde_json::from_str(&request).unwrap();
    request_json["refresh_token"].as_str().unwrap().to_string()
}

pub async fn get_access_token(refresh_token: &str) -> String {
    let file = File::open("/home/justus/client_secret.json").unwrap();
    let json: Value = serde_json::from_reader(file).unwrap();
    let mut hash = HashMap::new();
    hash.insert("grant_type", "refresh_token");
    hash.insert("client_id", json["client_id"].as_str().unwrap());
    hash.insert("client_secret", json["client_secret"].as_str().unwrap());
    hash.insert("refresh_token", &refresh_token);
    let request = make_form_req(hash).await;
    let request_json: Value = serde_json::from_str(&request).unwrap();
    println!("{}", request_json);
    request_json["access_token"].as_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            tokio::spawn(async {
                let body = get_access_token(include_str!("../token.txt")).await;
                println!("{:?}", send_email(vec!["justus.croskery@gmail.com".to_string()], "hi", "hi", &body).await);
            });
            loop {

            }
        });
    }
}
