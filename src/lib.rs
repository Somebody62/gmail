use openssl::ssl::{SslConnector, SslMethod};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;

pub fn make_secure_request(url: &str, params: HashMap<&str, &str>) -> String {
    let partial_url = url.split("https://").into_iter().nth(1).unwrap();
    let mut iter = partial_url.splitn(2, '/').into_iter();
    let host = iter.next().unwrap();
    let location = format!("/{}", iter.next().unwrap_or(""));
    let mut body = String::new();
    for (name, value) in params {
        body += &format!("{} = {}\r\n", name, value);
    }
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let stream = TcpStream::connect((host, 443)).unwrap();
    let mut stream = connector.connect(host, stream).unwrap();
    let message = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-length: {}\r\n\r\n{}",
        location,
        host,
        body.len(),
        body
    );
    stream.write_all(message.as_bytes()).unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    String::from_utf8(res).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut hash = HashMap::new();
        hash.insert("hi", "bye");
        println!(
            "{}",
            make_secure_request("https://accounts.google.com/o/oauth2/v2/auth", hash)
        )
    }
}
