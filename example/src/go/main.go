package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

// ResponseData matches the structure of your Python dictionary
type ResponseData struct {
	ResponseType string `json:"responseType"`
	ResponseCode int    `json:"responseCode"`
	Data         string `json:"data"`
}

func main() {
	// Create a scanner to read from standard input line by line
	scanner := bufio.NewScanner(os.Stdin)

	for scanner.Scan() {
		// scanner.Text() contains the current line, similar to Python's "for line in sys.stdin"
		_ = scanner.Text() 

		// Wrap processing in a function or block to handle errors cleanly
		if err := processLine(); err != nil {
			// If anything breaks, print to stdout so Rust can see the error
			fmt.Printf("ERROR: %v\n", err)
		}
	}

	if err := scanner.Err(); err != nil {
		fmt.Printf("ERROR reading stdin: %v\n", err)
	}
}

func processLine() error {
	// Get the directory where the executable lives
	execPath, err := os.Executable()
	if err != nil {
		return err
	}
	scriptDir := filepath.Dir(execPath)
	htmlPath := filepath.Join(scriptDir, "index.html")

	// 1. Construct the data structure
	data := ResponseData{
		ResponseType: "htmlfile",
		ResponseCode: 200,
		Data:         htmlPath,
	}

	// 2. Convert struct to a clean single-line JSON payload
	jsonBytes, err := json.Marshal(data)
	if err != nil {
		return err
	}

	// 3. CRUCIAL: Print the JSON line. 
	// os.Stdout in Go is unbuffered by default when printing strings, 
	// but adding \n ensures a clean line for Rust's line-reader.
	fmt.Println(string(jsonBytes))

	return nil
}
