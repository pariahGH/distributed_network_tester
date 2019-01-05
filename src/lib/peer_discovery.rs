//this fle will contain all of our default implementations of peer discovery
use super::util::PeerMode;
use super::error::{ClientResult, ClientError};
use std::fs;

#[derive(Debug)]
pub struct PeerFinder {
    mode: PeerMode,
    client_id: String
}

impl PeerFinder {
    pub fn new(mode: PeerMode, client_id: String) -> PeerFinder{
        return PeerFinder{
            mode,
            client_id
        }
    }
    pub fn get_peers(&self) -> ClientResult<Vec<String>, ClientError> {
        match &self.mode {
            PeerMode::DIRECT => direct_peer_search(&self.client_id),
            PeerMode::EMULATED => emulated_peer_search()
        }
    }
}

fn direct_peer_search(client_id: &String) -> ClientResult<Vec<String>, ClientError>{
    //currently this just scans sockets for any properly prefixed filenames
    let mut peers = Vec::new();
    let paths = fs::read_dir("./sockets/")?;
    for path in paths {
        let path = path?;
        let path_name = path.file_name().into_string()?;
        if path_name.contains(".client") && !path_name.contains(client_id) {
            peers.push(format!("./sockets/{}",path_name));
        }
    }
    Ok(peers)
}

//TODO: probably what will happen is I will send a request to the network emulator which 
//will act as a bootstrap node for peers - send a random list of known clients.
fn emulated_peer_search() ->  ClientResult<Vec<String>, ClientError>{
    let peers = Vec::new();
    
    Ok(peers)
}