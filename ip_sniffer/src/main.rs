use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

const MAX: u16 = 65535;

struct Arguments {
    threads: u32,
    ipaddr: IpAddr,
    flag: String,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }
        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                threads: 4,
                ipaddr,
                flag: String::from(""),
            });
        } else {
            let flag = args[1].clone();

            if flag.contains("-h") || flag.contains("--help") && args.len() == 2 {
                println!(
                    "Usage: -j to select how many threads you want
                \r\n    -h or --help to show this help message"
                );
                return Err("help");
            } else if flag.contains("-j") {
                let ipaddr = IpAddr::from_str(&args[3]).expect("Not a valid IP Address");
                let threads = args[2]
                    .parse::<u32>()
                    .expect("Failed to parse thread number");

                return Ok(Arguments {
                    threads,
                    ipaddr,
                    flag,
                });
            } else {
                return Err("Invalid syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }
        if (MAX - port) <= num_threads { 
            break;
        }

        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("Problem parsing arguments: {}", err);
            process::exit(0);
        }
    });
    let num_threads = arguments.threads;
    let addr = arguments.ipaddr;
    let (tx, rx) = channel();

    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i as u16, addr, num_threads as u16);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    } 

    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }  
}
