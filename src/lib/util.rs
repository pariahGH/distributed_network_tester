#[derive(Debug)]
pub enum PeerMode {
    EMULATED,
    DIRECT
}

#[derive(Debug)]
pub struct Message<'a> {
    pub destination: &'a str,
    pub source: &'a str,
    pub payload: &'a str,
}

impl<'a> Message<'a> {
    pub fn stringify(&self) -> String{
        self.destination.to_owned() + "---" + &self.source + "---" + &self.payload
    }

    pub fn decode(data:&'a String) -> Message<'a> {
        let data_split: Vec<&str> = data.split_terminator("---").collect();
        return Message{
            destination: data_split[0],
            source: data_split[1],
            payload: data_split[2]
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub socket: String,
    pub console: bool
}

impl Config {
    pub fn parse_args(args: std::env::Args) -> Result<Config, String> {
        let mut conf = Config{socket:String::from(""),console:false};
        for arg in args.skip(1) {
            if arg == "-c" { conf.console = true }
            else { conf.socket = arg }
        }
        Ok(conf)
    }
}