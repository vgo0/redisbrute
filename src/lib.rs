use clap::Parser;
use crossbeam_queue::SegQueue;
use parking_lot::RwLock;
use std::{
    error::Error,
    fs::File,
    io::{prelude::*, BufReader},
    process,
    sync::Arc,
    thread,
};
use worker::Worker;
use std::fs::OpenOptions;


pub mod connection;
pub mod worker;

/// Redis brute forcer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Timeout in seconds for each connection attempt
    #[arg(long, default_value_t = 5)]
    timeout: u64,
    
    /// Redis target in the format ip:port
    #[arg(short = 'T', long, default_value = "127.0.0.1:6379")]
    target: String,

    /// A file containing a list of targets in the format ip:port (one per line)
    #[arg(short = 'l', long)]
    target_list: Option<String>,

    /// Username list for ACL brute forcing
    #[arg(short, long, default_value = "")]
    users: String,

    /// Password file
    #[arg(short, long)]
    passwords: String,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 5)]
    threads: u8,

    /// Output file to save successful credentials in JSON format
    #[arg(short, long)]
    output_file: Option<String>,
}

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    // Ensure the output file is cleaned at the start if it exists
    if let Some(ref output_file) = args.output_file {
        initialize_output_file(output_file);  // Clean the file at the start
    }

    let arc_args = Arc::new(args); // Convert args to Arc<Args>

    let targets = if let Some(target_list) = &arc_args.target_list {
        load_targets_from_file(target_list)?
    } else {
        vec![arc_args.target.clone()]
    };

    for target in targets {
        run_target(&target, arc_args.clone())?; // Pass Arc<Args>
    }

    Ok(())
}


fn initialize_output_file(file_path: &str) {
    // Open the file in write mode and truncate it to clean it
    let _ = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .expect("Unable to clean the output file");
}


pub fn run_target(target: &String, args: Arc<Args>) -> Result<(), Box<dyn Error>> {
    let arc_users = Arc::new(RwLock::new(load_users(&args.users)));
    let arc_q = Arc::new(SegQueue::new());

    let mut worker = Worker::new(arc_users.clone(), arc_q.clone(), args.clone(), target.clone());

    // Perform the authentication check once before spawning threads
    if !worker.check_authentication() {
        println!("[-] Skipping target {}:{} since no authentication is required.", worker.get_ip(), worker.get_port());
        return Ok(());
    }

    let mut thread_handles_aq: Vec<thread::JoinHandle<()>> = Vec::new();

    thread_handles_aq.push(run_producer(arc_q.clone(), args.clone())); // Use arc_args directly

    for _ in 1..args.threads + 1 {
        let mut worker = Worker::new(arc_users.clone(), arc_q.clone(), args.clone(), target.clone());
        thread_handles_aq.push(thread::spawn(move || {
            worker.run_queue();
        }));
    }

    thread_handles_aq
        .into_iter()
        .for_each(|th| th.join().expect("can't join thread"));

    Ok(())
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

fn load_targets_from_file(target_file: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(target_file)?;
    let reader = BufReader::new(file);
    let mut targets = Vec::new();

    for line in reader.lines() {
        if let Ok(target) = line {
            targets.push(target);
        }
    }

    Ok(targets)
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