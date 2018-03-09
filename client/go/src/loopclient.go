package main

import (
  "net"
  "bufio"
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

func main() {
	conn, err_connect := net.Dial("tcp", "127.0.0.1:7000")
	if err_connect != nil {
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
		encoder.Encode(Request_PUT {
			Request_type: "PUT",
			Key: "key",
      Value: "val",
		})

		server.writer.Flush()

		var response Response
		err_read := decoder.Decode(&response)
		if err_read != nil {
			println("Connection terminated by server")
			return
		}

		println(response.Message)
	}
}
