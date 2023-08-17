use std::collections::HashMap;
use std::error::Error;
use bytes::Bytes;
use crate::resp_type::{RESPData, RESPType};


pub struct DB {
    pub data: HashMap<String, Bytes>,
}

pub trait Command {
    fn execute(&self, db: &mut DB) -> Result<(), Box<dyn Error>>;
}

pub enum Commands {
    Ping,
    Set(String, Bytes),
    Get(String),
}

impl Command for Commands::Ping {
    fn execute(&self, db: &mut DB) -> Result<RESPData, Box<dyn Error>> {
        return Ok(RESPData::new(RESPType::SimpleString("PONG".to_string()), 4));
    }
}