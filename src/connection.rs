use std::{ 
    net::TcpStream, 
    process,
    io::{Write, BufReader, BufRead},
    time::Duration
};

pub struct Connection {
    addr: String,
    stream: TcpStream
}

impl Connection {
    pub fn new(host: &String, port: &String) -> Connection {
        assert!(!host.is_empty());
        assert!(!port.is_empty());

        let addr = format!("{host}:{port}");
        let init_stream = Connection::connect(&addr);

        Connection {
            addr: addr,
            stream: init_stream
        }
    }

    fn try_send(&mut self, payload: &String) -> Result<(), String> {
        if let Err(_) = self.stream.write_all(payload.as_bytes()) {
            return Err("Error writing to connection".to_string());
        }

        self.stream.flush().unwrap_or_else(|err| {
            eprintln!("[-] Error flushing socket: {err}");
            process::exit(1);
        });

        Ok(())
    }

    fn try_receive(&mut self) -> Result<Vec<u8>, String> {
        let mut buf_reader = BufReader::new(&self.stream);
        
        let received = buf_reader.fill_buf().unwrap_or_else(|err| {
            eprintln!("[-] Unable to read response: {err}");
            process::exit(1);
        }).to_vec();
    
        buf_reader.consume(received.len());
        
        if received.len() == 0 {
            return Err("Empty response buffer".to_string())
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
        self.stream = Connection::connect(&self.addr);
    }

    fn connect(addr: &String) -> TcpStream {
        let stream = TcpStream::connect(addr).unwrap_or_else(|err| {
            eprintln!("[-] Error connecting to address {addr}: {err}");
            process::exit(1);
        });

        if let Err(err) = stream.set_read_timeout(Some(Duration::new(15, 0))) {
            eprintln!("[-] Error setting connection read timeout: {err}");
            process::exit(1);
        }

        if let Err(err) = stream.set_write_timeout(Some(Duration::new(15, 0))) {
            eprintln!("[-] Error setting connection write timeout: {err}");
            process::exit(1);
        }

        stream
    }
}