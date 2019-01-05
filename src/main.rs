mod lib;
use crate::lib::client::Client;
use crate::lib::util::Config;
use crate::lib::util::Message;
use crate::lib::error::ClientError;
use std::env;
use std::io;
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::fs;

fn main() {
    if !check_sockets_dir() {
        return
    }
    let conf = Config::parse_args(env::args()).expect("Error parsing args)"); 
    println!("Config: {:?}", conf);
    let (client, receiver) = Client::register(conf.socket).expect("Error creating client");
    println!("Client: {:?}", client);
    let threaded_client = Arc::new(client);
    let cloned_threaded_client = threaded_client.clone();
    //TODO: implement threaded updating of this
    let peers = threaded_client.peer_finder.get_peers().expect("Could not find peers");
    thread::spawn(move || handler(receiver, cloned_threaded_client));
    if conf.console {
        println!("Operating in console mode");
        loop{
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let status = match input.trim(){
                "send" => handle_send(&threaded_client, &peers),
                "show-peers" => show_peers(&peers),
                &_ => Err(ClientError::ClientError("Invalid Command".to_string()))
            };
            println!("{:?}\n", status);
        }
    }
}

fn handler(receiver: Receiver<String>, client: Arc<Client>){
    loop{
        match receiver.recv() {
            Ok(data) => process_message(data, &client),
            Err(e) => println!("{:?}", e)
        }
    }
}

fn show_peers(peers: &Vec<String>) -> Result<&str, ClientError>{
    println!("{:?}", peers);
    Ok("Peers Printed!")
}

//TODO: dangerous to do this without a mechanism for determining if incoming is response
//to sent vs a brand new message
fn process_message(data: String, client: &Client){
    println!("{}", data);
    let message = Message::decode(&data);
    println!("Received message from {}: {}", message.source, message.payload);
    //doing this until message tracking implemented
    if !message.payload.contains("Message Received") {
        client.send_one(message.source, "Message Received").unwrap();
    }
}

fn handle_send<'a>(client: &'a Client, peers: &Vec<String>) -> Result<&'a str, ClientError>{
    println!("Enter payload:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    client.send_many(input.as_str(), peers)
}

fn check_sockets_dir() -> bool {
    return match fs::create_dir("./sockets") {
        Ok(_) => true,
        Err(e) => match e.kind() {
            io::ErrorKind::AlreadyExists => true,
            _ => {
                println!("Could not create sockets dir");
                false
            }
        }
    }
}