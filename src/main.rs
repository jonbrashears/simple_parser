use std::{io::prelude::*, net::ToSocketAddrs};
use std::net::{TcpListener, TcpStream, UdpSocket, Shutdown};
use std::thread;
use std::fs::OpenOptions;
use std::io;

extern crate clap;
use clap::{Command,Arg};

extern crate chrono;
use chrono::{DateTime,Local};

const EOF_SIZE: usize = 0;
const MAX_PACKET_SIZE: usize = 2048;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ConnectionTypes {
    TCP,
    UDP,
    SERIAL,
    UNKNOWN
}

trait Parse {
    fn parse(&self);
}

struct UdpParser {
    socket: String
}

impl UdpParser {
    fn new(socket: &str) -> UdpParser {
        UdpParser{
            socket: socket.to_string()
        }
    }
}

struct TcpParser {
    socket: String
}

impl TcpParser {
    fn new(socket: &str) -> TcpParser {
        TcpParser{
            socket: socket.to_string()
        }
    }
}

struct SerialParser {
    device_path: String,
    baud_rate: String
}

impl SerialParser {
    fn new(device_path: &str, baud_rate: &str) -> SerialParser {
        SerialParser{
            device_path: device_path.to_string(),
            baud_rate: baud_rate.to_string()
        }
    }
}

impl Parse for UdpParser {
    fn parse(&self) {
        start_udp(self.socket.clone())
    }
}

impl Parse for TcpParser {
    fn parse(&self) {
        start_tcp(self.socket.clone())
    }
}

impl Parse for SerialParser {
    fn parse(&self) {
        start_serial(self.device_path.clone(), self.baud_rate.clone())
    }
}

// Reads a line from the Tcp Stream
fn handle_tcp_connection(mut stream: TcpStream){
    // Reading the stream consumes it, so we have to clone the stream we wish to reply to
    let mut data = [0 as u8; MAX_PACKET_SIZE];
    while match stream.read(&mut data) {
        
        Ok(size) => {
            match size {
                EOF_SIZE=>{
                    println!("Client {} disconnected.", stream.peer_addr().unwrap());
                    false
                },
                _=>{
                        println!("{:?}", std::str::from_utf8(&data[0..size]).unwrap().to_string());
                
                        // Echo message back to client
                        stream.write(&data[0..size]).unwrap();
                        true
                }
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn start_tcp(socket: String) {
    println!("Starting up TCP server with Socket: {:?}...", socket);
    // Start server and listen on port
    let listener = TcpListener::bind(socket).expect("Error: Could not start server!");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_tcp_connection(stream)
                });
            }
            Err(e) => { println!("Error: Connection failed {}", e);}
        }
    }
}

fn start_udp(socket: String){
    let listen_socket = socket.clone();
    let socket = UdpSocket::bind(socket.clone()).expect("Error: Could not bind!");
    println!("Listening on UDP socket: {:?}...", listen_socket);
    loop {
        // Receives a single datagram of size 'MAX_PACKET_SIZE'. Data exceeding that size will be cut off.
        let mut data = vec![0; MAX_PACKET_SIZE];
        let (size, source_socket) = socket.recv_from(&mut data).expect("Error getting data");

        println!("Received packet of size {} from socket: {}", size, source_socket);
        println!("got {}", std::str::from_utf8(&data[0..size]).unwrap().to_string());
    }
}

fn start_serial(device_path: String, baud_rate: String) {
    println!("Parsing with connection {} and baud rate {}", device_path, baud_rate);
    println!("THIS IS NOT YET IMPLEMENTED!")
}

fn command_args() -> clap::ArgMatches {
    Command::new("simple-parser-app").version("v1.0-beta")
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
                    .get_matches()
}

fn main() {
    let matches = command_args();
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
            let tcp_server = TcpParser::new(&tcp_socket);
            tcp_server.parse()
        },
        ConnectionTypes::UDP=> {
            let udp_socket = connection_params;
            let udp_listener = UdpParser::new(&udp_socket);
            udp_listener.parse();
        },
        ConnectionTypes::SERIAL=>{
            let baud_rate = connection_params;
            let serial_connection = SerialParser::new("/dev/ttyS0", &baud_rate);
            serial_connection.parse();
        },
        _=>println!("Unknown connection type!"),
    }
}
