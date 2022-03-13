use std::{io::prelude::*, net::ToSocketAddrs};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::fs::OpenOptions;
use std::io;

extern crate clap;
use clap::{Command,Arg};

extern crate chrono;
use chrono::{DateTime,Local};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ConnectionTypes {
    TCP,
    UDP,
    SERIAL,
    UNKNOWN
}

// Reads a line from the Tcp Stream
fn handle_tcp_connection(stream: TcpStream) -> io::Result<()>{
    // Reading the stream consumes it, so we have to clone the stream we wish to reply to
    let mut ostream = stream.try_clone()?;
    let mut rdr = io::BufReader::new(stream);
    let mut text = String::new();
    rdr.read_line(&mut text)?;
    println!("got {}", text.trim_end());

    // Echo received line back to client
    ostream.write_all(text.as_bytes())?;
    Ok(())
}

fn run_tcp_server(socket: String) {
    println!("Starting up TCP server with Socket: {:?}...", socket);
    // Start server and listen on port
    let listener = TcpListener::bind(socket).expect("Error: Could not start server!");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                if let Err(e) = handle_tcp_connection(stream) {
                    println!("error {:?}", e);
                }
            }
            Err(e) => { println!("Error: Connection failed {}", e);}
        }
    }
}

fn start_udp(socket: String) -> std::io::Result<()> {
    println!("Listening on UDP socket: {:?}...", socket);
    let socket = UdpSocket::bind(socket).expect("Error: Could not bind!");

    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = vec![0; 2048];
    let (amt, src) = socket.recv_from(&mut buf)?;

    println!("Received packet of size {} from socket: {}", amt, src);
    for index in 0..amt {
        println!("got {}", buf[index] as char);
    }
    
    Ok(())

}

fn main() {
    let matches = Command::new("simple-parser-app").version("v1.0-beta")
                    .arg(Arg::new("CONNECTION TYPE")
                         .short('c')
                         .long("connection")
                         .required(true)
                         .takes_value(true))
                    .arg(Arg::new("PARAMETERS")
                         .short('p')
                         .long("parameters")
                         .required(true)
                         .takes_value(true))
                    .get_matches();

    let connection_type_arg = matches.value_of("CONNECTION TYPE").unwrap().to_string().to_ascii_lowercase();
    let connection_params= matches.value_of("PARAMETERS").unwrap().to_string();
    let connection_type: ConnectionTypes;

    match connection_type_arg.as_str() {
        "tcp"=>connection_type = ConnectionTypes::TCP,
        "udp"=>connection_type = ConnectionTypes::UDP,
        "serial"=>connection_type = ConnectionTypes::SERIAL,
        _=>connection_type = ConnectionTypes::UNKNOWN,
    }

    match connection_type {
        ConnectionTypes::TCP=>{
            let tcp_socket = connection_params;
            run_tcp_server(tcp_socket)
        },
        ConnectionTypes::UDP=> {
            let udp_socket = connection_params;
            loop {
                start_udp(udp_socket.clone()).unwrap();
            }
        },
        ConnectionTypes::SERIAL=>{
            let baud_rate = connection_params.parse::<u64>();
            println!("Setting baud rate to {:?}", baud_rate);
            println!("Serial connections are not implemented yet!")
        },
        _=>println!("Unknown connection type!"),
    }
}
