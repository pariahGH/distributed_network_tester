use super::util::Message;
use super::util::PeerMode;
use super::peer_discovery::PeerFinder;
use super::error::{ClientResult, ClientError};
use std::os::unix::net::{UnixStream, UnixListener};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::prelude::*;
use std::thread;
use std::path::Path;
use std::net::Shutdown;
use rand::Rng;
extern crate signal_hook;
use std::sync::Arc;

#[derive(Debug)]
pub struct Client{
    ip: String,
    client_socket: String,
    emulated_network_socket: String,
    mode: PeerMode,
    pub peer_finder: PeerFinder,
    client_id: u32
}

//TODO: maybe should be able to take an optional function that will be used to immediately reply to incoming messages
//vs just closing socket after read and delegating responses to caller code?
impl<'a> Client{
    pub fn register(emulated_network_socket:String) -> ClientResult<(Client, Receiver<String>), ClientError>{
        let ip = request_ip(&emulated_network_socket)?;
        let mode = determine_peer_mode(&emulated_network_socket);
        let (client_id,client_socket) = gen_name();
        let read_stream = UnixListener::bind(&client_socket)?;
        let (sender, receiver) = channel();
        thread::spawn(move || listener(&read_stream, sender));

        let peer_finder = PeerFinder::new(mode, client_id.to_string());
        let client = Client{
            ip,
            client_socket,
            emulated_network_socket,
            mode: PeerMode::DIRECT,
            peer_finder,
            client_id
        };
        let cloned_id = Arc::new(client_id);
        unsafe {
            signal_hook::register(signal_hook::SIGINT, move || clean_up(cloned_id.clone())).unwrap();
        }
        return Ok((client,receiver))
    }

    pub fn send_many(&self, payload: &str, peers:&Vec<String>) -> ClientResult<&str, ClientError>{
        for destination in peers {
            let mut err = false;
            
            match &self.send_one(destination, payload) {
                Ok(_) => {},
                Err(_) => err = true
            };
            
            //If we had an error at this level, any remaining will also probably break
            //TODO: i do NOT like this - its ugly, but i cant put a `break value` statement in that match
            if err {break}

        };
        Ok("Messages Sent")
    }

    pub fn send_one(&'a self, destination:&'a str, payload: &str) -> ClientResult<&'a str, ClientError>{
        let source = match self.ip.as_str() {
            "" => &self.client_socket,
            _ => &self.ip
        };
        let message = Message {
            payload,
            source,
            destination
        }.stringify();

        match self.mode {
            PeerMode::DIRECT => transmit(message, destination),
            PeerMode::EMULATED => transmit(message, &self.emulated_network_socket)
        }
    }
}
fn clean_up(id:Arc<u32>){
    println!("{:?}", &id);
    std::fs::remove_file(format!("./sockets/{}.client", &id)).unwrap();
    std::process::exit(0);
}

fn listener(listener: &UnixListener, sender:Sender<String>){
    loop {
        match listener.accept() {
            Ok((socket, _)) => {
               handle_client(socket, sender.clone())
            },
            Err(err) => {println!("{:?}", err); return;}
        };
    }
}

fn handle_client(mut socket:UnixStream, sender:Sender<String>){
    let mut incoming = String::new();
    match socket.read_to_string(&mut incoming){
        Ok(_) => {
            sender.send(incoming).unwrap();
            socket.shutdown(Shutdown::Both).unwrap();
        },
        Err(e) => println!("Error reading message from socket! {}", e)
    }
}

fn transmit(data:String, destination:&str) -> ClientResult<&str, ClientError>{
    let mut write_stream = UnixStream::connect(destination).expect("Could not connect to local socket as writer");

    let sent = write_stream.write(&data.into_bytes()).expect("could not write");
    write_stream.flush()?;
    println!("Message sent! {:?} bytes", sent);
    write_stream.shutdown(Shutdown::Write).unwrap();
    //TODO: if we arent handling replying directly here, then we can skip this code.
    let mut response = String::new();
    write_stream.read_to_string(&mut response)?;
    println!("Response from listener: {}", response);
    write_stream.shutdown(Shutdown::Read).unwrap();
    Ok("Transmitted")
}

fn gen_name<'a> () -> (u32, String){
    loop {
        let client_id = rand::thread_rng().gen_range(100000000,1000000000);
        let name  = format!("./sockets/{}.client", client_id);
        if !Path::new(&name).exists() {
            break (client_id, name);
        }
    }
}

fn request_ip(emulated_network_socket: &String) -> Result<String, ClientError> {
    Ok(if emulated_network_socket.as_str() == "" {
        "".to_string()
    } else {
        //TODO: inform network of our existence & request IP from network
        //graceful handling if network emulator isn't running
        "".to_string()
    })
}

fn determine_peer_mode(emulated_network_socket: &String) -> PeerMode {
    if emulated_network_socket.as_str() == "" {
        PeerMode::DIRECT
    } else {
        PeerMode::EMULATED
    }
}