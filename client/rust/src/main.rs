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
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .ok()
            .expect("Couldn't read line");

        println!("herea");
        input.pop(); //remove the new line character
        let json_input: Vec<&str> = input.split(" ").collect();
        println!("json_input {:?} ", json_input);

        let mut key = String::new(); //retrieve key
        if json_input.len() >= 2  {
            key = json_input[1].to_string();
        }

        let mut value = String::new();
        if json_input.len() >= 3 { //retrieve value
            value = json_input[2].to_string();
        }

        println!("key: {}", key);
        println!("value: {}", value);

        println!("hereb");
        let json_value = json!({
        	"Request_type": json_input[0],
        	"Key": key,
            "Value": value,
        });
        serde_json::to_writer(&mut stream, &json_value);
        println!("herec");
        // stream.flush();//send eof
        let value_iter = Deserializer::from_reader(&stream).into_iter::<Value>();
        println!("hered");

        println!("size iter: {:?}", value_iter.size_hint());
        for response in value_iter {
            println!("heree");
            let response = response.unwrap();
            println!("result: {}", response["Result"]);
            println!("value: {}", response["Value"]);
            break; // maybe a little bit hack-y, need to figure out proper way to read only one iter
        }
        println!("heref");

        if json_input[0] == "END" {
            break; //maybe a little hack-y but I think it should be ok, it does the job because the stream goes out of scope after main() ends
        }
    }

	// the stream is closed here
}
