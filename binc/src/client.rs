use crate::network_protocol::{NetworkRequest, NetworkResponse};
use std::io;
use std::net::TcpStream;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(url: &str) -> io::Result<Client> {
        let stream = TcpStream::connect(url)?;
        Ok(Client { stream })
    }

    pub fn request(&mut self, request: NetworkRequest) -> io::Result<NetworkResponse> {
        request.write(&mut self.stream)?;
        NetworkResponse::read(&mut self.stream)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        NetworkRequest::Disconnect.write(&mut self.stream).unwrap();
    }
}
