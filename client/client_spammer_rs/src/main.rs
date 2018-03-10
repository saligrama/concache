#![allow(unused)]
#[macro_use]
extern crate serde_json;

use std::io;
use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use serde_json::{Deserializer, Value};


fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7000").expect("Couldn't connect to server.");
    //read from the user here
    loop { 
        // let mut input = String::new();
        // io::stdin().read_line(&mut input)
        //     .ok()
        //     .expect("Couldn't read line");

        // input.pop(); //remove the new line character
        // let json_input: Vec<&str> = input.split(" ").collect();
        // println!("json_input {:?} ", json_input);

        // let mut key = String::new(); //retrieve key
        // if json_input.len() >= 2  {
        //     key = json_input[1].to_string();
        // }

        // let mut value = String::new();
        // if json_input.len() >= 3 { //retrieve value
        //     value = json_input[2].to_string();
        // }

        // println!("key: {}", key);
        // println!("value: {}", value);

        let json_value = json!({
        	"Request_type": "GET",
        	"Key": "5",
            "Value": "",
        });
        serde_json::to_writer(&mut stream, &json_value);
        // stream.flush();//send eof
        let mut value_iter = Deserializer::from_reader(&stream).into_iter::<Value>();

        println!("size iter: {:?}", value_iter.size_hint());
        
        match value_iter.next() {
            Some(Ok(res)) => {
                println!("{:?}", res);
            } 
            Some(Err(e)) => {
                println!("{:?}", e);
                return; //exit
            }
            None => unreachable!()
        }


        // if json_input[0] == "END" {
        //     break; 
        // }
    }

	// the stream is closed here
}
