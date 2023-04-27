package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"math/rand"
	"os"
	"time"
)

type Data struct {
	CPU     float64   `json:"cpu"`
	Memory  float64   `json:"memory"`
	Disk    float64   `json:"disk"`
	Network float64   `json:"network"`
	Time    time.Time `json:"time"`
}

func main() {
	rand.Seed(time.Now().UnixNano())

	file, err := os.Create("data.json")
	if err != nil {
		panic(err)
	}
	defer file.Close()

	writer := bufio.NewWriter(file)

	for {
		cpu := rand.NormFloat64()*20 + 70
		memory := rand.Float64()*20 + 70
		disk := rand.Float64()*20 + 50
		network := rand.NormFloat64()*50 + 1000

		data := Data{
			CPU:     cpu,
			Memory:  memory,
			Disk:    disk,
			Network: network,
			Time:    time.Now(),
		}

		jsonData, err := json.Marshal(data)
		if err != nil {
			panic(err)
		}

		fmt.Println(string(jsonData))
		_, err = writer.WriteString(string(jsonData) + "\n")
		if err != nil {
			panic(err)
		}

		err = writer.Flush()
		if err != nil {
			panic(err)
		}

		time.Sleep(time.Second * 1)
	}
}
