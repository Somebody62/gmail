use openssl::ssl::{SslConnector, SslMethod};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;

pub fn make_secure_request(url: &str, params: HashMap<&str, &str>, method: &str) -> String {
    let partial_url = url.split("https://").into_iter().nth(1).unwrap();
    let mut iter = partial_url.splitn(2, '/').into_iter();
    let host = iter.next().unwrap();
    let location = format!("/{}", iter.next().unwrap_or(""));
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect((host, 443)).unwrap();
    let mut stream = connector.connect(host, stream).unwrap();
    let message = if method == "GET" {
        let mut location = location.to_string();
        location += "?";
        for (name, value) in params {
            location += &format!("{}={}&", name, value);
        }
        location = location[0..location.len() - 1].to_string();
        format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            location,
            host
        )
    } else {
        let mut body = String::new();
        for (name, value) in params {
            body += &format!("{}={}&", name, value);
        }
        if body.len() > 0 {
            body = body[0..body.len() - 1].to_string();
        }
        format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-length: {}\r\nContent-type: application/x-www-form-urlencoded\r\n\r\n{}",
            location,
            host,
            body.len(),
            body
        )
    };
    stream.write_all(message.as_bytes()).unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    let response = String::from_utf8(res).unwrap();
    response
        .split("\r\n\r\n")
        .into_iter()
        .nth(1)
        .unwrap()
        .to_string()
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
            make_secure_request(
                "https://www.googleapis.com/gmail/v1/users/me/profile",
                hash,
                "GET"
            )
        )
    }
}
