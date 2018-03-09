package main

import (
	"bufio"
	"fmt"
	"os"
	"net"
	"strings"
	"encoding/json"
)

type Request_PUT struct {
	Request_type string
	Key string
	Value string
}

type Request_GETDEL struct {
	Request_type string
	Key string
}

type Server struct {
	reader *bufio.Reader
	writer *bufio.Writer
}

type Response struct {
	Message string
}

func main () {
	reader := bufio.NewReader(os.Stdin)

	var cmd string

	conn, connErr := net.Dial("tcp", "127.0.0.1:7000")
	if connErr != nil {
		println("Could not connect to server")
		return
	}

	server := &Server {
		reader: bufio.NewReader(conn),
		writer: bufio.NewWriter(conn),
	}

	encoder := json.NewEncoder(server.writer)
	decoder := json.NewDecoder(server.reader)

	for {
		fmt.Print("Enter command: ")
		cmd, _ = reader.ReadString('\n')

		cmd = strings.Trim(cmd, "\n")

		if cmd == "END" {
			break
		}

		cmd_arr := strings.Split(cmd, " ")

		var writeErr error

		if len(cmd_arr) <= 1 || len(cmd_arr) >= 4 {
			fmt.Println("Please enter a command.")
			continue
		} else if len(cmd_arr) == 2 {
			// GET or DEL
			send := Request_GETDEL {
				Request_type: cmd_arr[0],
				Key: cmd_arr[1],
			}

			writeErr = encoder.Encode(send)
		} else if len(cmd_arr) == 3 {
			// PUT
			send := Request_PUT {
				Request_type: cmd_arr[0],
				Key: cmd_arr[1],
				Value: cmd_arr[2],
			}

			writeErr = encoder.Encode(send)
		}

		if writeErr != nil {
			println("Connection terminated by server")
			return
		}

		server.writer.Flush()

		var response Response
		readErr := decoder.Decode(&response)
		if readErr != nil {
			println("Connection terminated by server")
			return
		}

		println(response.Message)
	}
}
