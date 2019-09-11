#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        println!("{}", secure_request::make("https://accounts.google.com/"))
    }
}

mod secure_request {
    use openssl::ssl::{SslConnector, SslMethod};
    use std::io::prelude::*;
    use std::net::TcpStream;

    pub fn make(url: &str) -> String {
        let partial_url = url.split("https://").into_iter().nth(1).unwrap();
        let mut iter = partial_url.split("/").into_iter();
        let host = iter.nth(0).unwrap();
        let location = format!("/{}", iter.nth(1).unwrap_or(""));
        let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
        let stream = TcpStream::connect((host, 443)).unwrap();
        let mut stream = connector.connect(host, stream).unwrap();
        stream
            .write_all(
                format!(
                    "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                    location, host
                )
                .as_bytes(),
            )
            .unwrap();
        let mut res = vec![];
        stream.read_to_end(&mut res).unwrap();
        String::from_utf8(res).unwrap()
    }
}
