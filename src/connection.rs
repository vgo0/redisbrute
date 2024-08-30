use std::{ 
    net::TcpStream, 
    io::{Write, BufReader, BufRead},
    time::Duration

};

pub struct Connection {
    addr: String,
    stream: TcpStream,
}

impl Connection {
    pub fn new(host: &String, port: &String, timeout: Duration) -> Connection {
        assert!(!host.is_empty());
        assert!(!port.is_empty());

        let addr = format!("{host}:{port}");
        let init_stream = Connection::connect(&addr, timeout);

        Connection {
            addr: addr,
            stream: init_stream,
        }
    }

    fn try_send(&mut self, payload: &String) -> Result<(), String> {
        if let Err(_) = self.stream.write_all(payload.as_bytes()) {
            return Err("Error writing to connection".to_string());
        }

        self.stream.flush().unwrap_or_else(|err| {
            eprintln!("[-] Error flushing socket: {err}");
        });

        Ok(())
    }

    fn try_receive(&mut self) -> Result<Vec<u8>, String> {
        let mut buf_reader = BufReader::new(&self.stream);
    
        let received = buf_reader.fill_buf().unwrap_or_else(|err| {
            eprintln!("[-] Unable to read response: {err}");
            &[]
        }).to_vec();
    
        buf_reader.consume(received.len());
    
        if received.is_empty() {
            return Err("Empty response buffer".to_string());
        }
    
        Ok(received)
    }

    fn try_send_and_receive(&mut self, payload: &String) -> Result<Vec<u8>, String> {
        if let Err(err) = self.try_send(payload) {
            return Err(err);
        }

        self.try_receive()
    }

    pub fn send_and_receive(&mut self, payload: String) -> Vec<u8> {
        loop {
            match self.try_send_and_receive(&payload) {
                Ok(result) => {
                    return result;
                },
                Err(_) => {
                    self.reconnect();
                }
            }
        }
    }

    fn reconnect(&mut self) {
        // Unwrap the Option<Duration> to get Duration, or use a default value of 5 seconds
        let timeout = self.stream.read_timeout().unwrap_or(Some(Duration::from_secs(5))).unwrap();
        self.stream = Connection::connect(&self.addr, timeout);
    }
    

    fn connect(addr: &String, timeout: Duration) -> TcpStream {
        let stream = TcpStream::connect(addr).unwrap_or_else(|err| {
            eprintln!("[-] Error connecting to address {addr}: {err}");
            std::process::exit(1);
        });

        stream.set_read_timeout(Some(timeout)).unwrap_or_else(|err| {
            eprintln!("[-] Error setting connection read timeout: {err}");
        });

        stream.set_write_timeout(Some(timeout)).unwrap_or_else(|err| {
            eprintln!("[-] Error setting connection write timeout: {err}");
        });

        stream
    }
}
