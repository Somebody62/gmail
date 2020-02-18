use openssl::ssl::{SslConnector, SslMethod};
use serde_json::{Value, json};
use email_format::Email;
use chrono::prelude::*;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;

fn send(host: &str, message: &str) -> String {
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect((host, 443)).unwrap();
    let mut ssl_stream = connector.connect(host, stream).unwrap();
    ssl_stream.write_all(message.as_bytes()).unwrap();
    let mut res = vec![];
    ssl_stream.read_to_end(&mut res).unwrap();
    String::from_utf8(res).unwrap()
}

fn make_form_request(host: &str, location: &str, params: HashMap<&str, &str>) -> String {
    let mut body = String::new();
    for (name, value) in params {
        body += &format!("{}={}&", name, value);
    }
    if body.len() > 0 {
        body = body[0..body.len() - 1].to_string();
    }
    let message = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-length: {}\r\nContent-type: application/x-www-form-urlencoded\r\n\r\n{}",
        location,
        host,
        body.len(),
        body
    );
    let response = send(host, &message);
    response
        .split("\r\n\r\n")
        .into_iter()
        .nth(1)
        .unwrap()
        .to_string()
}

pub fn send_email(names: Vec<String>, addresses: Vec<String>, subject: &str, body: &str, auth: &str) -> String {
    let mut email = Email::new("justus@olmmcc.tk", Utc::now().to_rfc2822().as_str()).unwrap();
    let mut to_string = String::new();
    for i in 0..names.len() {
        to_string += &format!("{} <{}>,", names[i], addresses[i]);
    }
    email.set_to(&to_string[0..to_string.len() - 2]).unwrap();
    email.set_subject(subject).unwrap();
    email.set_body(body).unwrap();
    let raw = base64::encode(&email.as_bytes());
    let body = json!({
        "raw": raw
    }).to_string();
    let message = format!("POST /gmail/v1/users/me/messages/send HTTP/1.1\r\nHost: www.googleapis.com\r\nAccept: application/json\r\nConnection: close\r\nAuthorization: Bearer {}\r\nContent-length: {}\r\nContent-type: application/json\r\n\r\n{}", auth, body.len(), body);
    let response = send("www.googleapis.com", &message);
    response
        .split("\r\n\r\n")
        .into_iter()
        .nth(1)
        .unwrap()
        .to_string()
}

pub fn get_refresh_token(code: &str) -> String {
    let file = File::open("/home/justus/client_secret.json").unwrap();
    let json: Value = serde_json::from_reader(file).unwrap();
    let mut hash = HashMap::new();
    hash.insert("code", code);
    hash.insert("access_type", "offline");
    hash.insert("client_id", json["client_id"].as_str().unwrap());
    hash.insert("client_secret", json["client_secret"].as_str().unwrap());
    hash.insert("redirect_uri", "https://www.olmmcc.tk/admin/email/");
    hash.insert("grant_type", "authorization_code");
    let request = make_form_request("www.googleapis.com", "/oauth2/v4/token", hash);
    let request_json: Value = serde_json::from_str(&request).unwrap();
    request_json["refresh_token"].as_str().unwrap().to_string()
}

pub fn get_access_token(refresh_token: &str) -> String {
    let file = File::open("/home/justus/client_secret.json").unwrap();
    let json: Value = serde_json::from_reader(file).unwrap();
    let mut hash = HashMap::new();
    hash.insert("grant_type", "refresh_token");
    hash.insert("client_id", json["client_id"].as_str().unwrap());
    hash.insert("client_secret", json["client_secret"].as_str().unwrap());
    hash.insert("refresh_token", &refresh_token);
    let request = make_form_request("www.googleapis.com", "/oauth2/v4/token", hash);
    let request_json: Value = serde_json::from_str(&request).unwrap();
    request_json["access_token"].as_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut hash = HashMap::new();
        hash.insert("access_token", "ya29.ImCRB_FFKKObj_Y4TsKVKNT-ASHka-ZNeRWilEP6PPpHXAcebeD2grYfZV_MlvD-nCh59Jh03WIjAv9cVt6oc6Wix8pCIVuVXYuFAn33VYm0op-SrSR9lmueS6Gst-nO3UE");
        println!(
            "{}",
            make_form_request("www.googleapis.com", "/gmail/v1/users/me/profile", hash)
        )
    }
}
