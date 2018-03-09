#![allow(unused)]
#[macro_use]
extern crate serde_json;

use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use serde_json::{Deserializer, Value};

fn handle_connection(
    mut stream: &TcpStream,
    cache_map: &mut HashMap<String, String>,
) -> Result<(), Error> {

    println!("herea");

    let value_iter = Deserializer::from_reader(stream).into_iter::<Value>();
    println!("hereb");

    for v in value_iter {
        println!("hereca");
        let v = v.unwrap();
        println!("herecb");
        //values for return
        let mut result = String::new();
        let mut value = String::new();
        result = "NONE".to_string();
        value = "NONE".to_string();

        println!("herecc");
        if v["Request_type"] == "PUT" {
            let key = match &v["Key"].as_str() {
                &Some(ret) => ret,
                &None => return Err(Error::new(ErrorKind::Other, "Error with bad key")),
            }.to_string();
            let val = match &v["Value"].as_str() {
                &Some(ret) => ret,
                &None => return Err(Error::new(ErrorKind::Other, "Error with bad value")),
            }.to_string();
            cache_map.insert(key, val);
            result = "Success!".to_string();
        } else if v["Request_type"] == "GET" {
            let key = match &v["Key"].as_str() {
                &Some(ret) => ret,
                &None => return Err(Error::new(ErrorKind::Other, "Error with bad key")),
            };
            if let Some(ret) = cache_map.get(key) {
                result = "Success!".to_string();
                value = ret.to_string();
            } else {
                result = "No such key!".to_string();
            }
        } else if v["Request_type"] == "DEL" {
            let key = match &v["Key"].as_str() {
                &Some(ret) => ret,
                &None => return Err(Error::new(ErrorKind::Other, "Error with bad key")),
            };
            if let Some(_ret) = cache_map.remove(key) {
                result = "Success!".to_string();
            } else {
                result = "No such key!".to_string();
            }
        } else {
            result = "Invalid Key!".to_string();
        }

        //create json to send back
        println!("hered");
        let json_value = json!({
            "Result": result,
            "Value": value,
        });
        serde_json::to_writer(&mut stream, &json_value);
        println!("heree");
    }
    println!("heref");
    // stream.flush()? ; // not sure if need to send the EOF every time
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7000");
    let listener = match listener {
        Ok(conn) => conn,
        Err(err) => {
            panic!("Error opening the connection {:?}", err);
        }
    };

    let mut cache_map = HashMap::<String, String>::new();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                println!("Problem with connection {:?}", err);
                continue;
            }
        };
        if let Err(err) = handle_connection(&stream, &mut cache_map) {
            if let Err(e) = stream.write(format!("{:?}", err).as_bytes()) {
                println!("Error writing to stream {:?}", e);
            }
            println!("{:?}", err);
            continue;
        }
    }
}
