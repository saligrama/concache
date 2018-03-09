//used following link in creation: https://coderwall.com/p/wohavg/creating-a-simple-tcp-server-in-go

package main

import (
	"net"
	"fmt"
	"encoding/json"
	"bufio"
	"sync"
)

var data = make(map[string]string)
var mutex = &sync.Mutex{}

type Request struct {
	Request_type string
	Key string
	Value string
}

type Client struct {
	reader *bufio.Reader
	writer *bufio.Writer
}

type Response struct {
	Message string
}

func main() {
	ln, err := net.Listen("tcp", "127.0.0.1:7000")
	if err != nil {
		fmt.Println("Error accepting: ", err.Error())
	}
	fmt.Println("Connection opened on 127.0.0.1:7000")
	for {
		conn, err := ln.Accept()
		if err != nil {
			// handle error
			fmt.Println("Failed")
		}
		go handleConnection(conn)
	}
}

func handleConnection(conn net.Conn) {
	client := &Client {
		reader: bufio.NewReader(conn),
		writer: bufio.NewWriter(conn),
	}
	decoder := json.NewDecoder(client.reader)
	for {
		var rqst Request
		err := decoder.Decode(&rqst)
		if err != nil {
			conn.Close()
			return
		}
		handleRequest(rqst, *client)
	}
}

func handleRequest(rqst Request, client Client) {
	encoder := json.NewEncoder(client.writer)
	var response Response

	if rqst.Request_type == "GET" {
		fmt.Println("Getting: ", rqst.Key)
		//need to write something to check if the key even exists
		mutex.Lock()
		ret, ok := data[rqst.Key]
		mutex.Unlock()
		if ok {
			response = Response {
				Message: ret,
			}
		} else {
			response = Response {
				Message: "No such key",
			}
		}
	} else if rqst.Request_type == "PUT" {
		fmt.Println("Putting: ", rqst.Value)
		//need to write something to check if the key is already being used, write somehting to check this
		mutex.Lock()
		data[rqst.Key] = rqst.Value
		mutex.Unlock()
		response = Response {
			Message: "Success",
		};
	} else if rqst.Request_type == "DEL" {
		fmt.Println("Deleting: ", rqst.Key)
		mlen := len(data)
		mutex.Lock()
		delete(data, rqst.Key)
		mutex.Unlock()
		if len(data) != mlen {
			response = Response {
				Message: "Success",
			}
		} else {
			response = Response {
				Message: "No such key",
			}
		}
	} else {
		response = Response {
			Message: "Error: Bad command",
		}
	}

	encoder.Encode(response)

	client.writer.Flush()
}
