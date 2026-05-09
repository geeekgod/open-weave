package main

import (
	"fmt"
	"log"

	"github.com/openweave/go/openweave"
)

func main() {
	agent, err := openweave.New(openweave.AgentConfig{
		SystemPrompt: "You are a research agent.",
	})
	if err != nil {
		log.Fatalf("Failed to create agent: %v", err)
	}
	defer agent.Destroy()

	fmt.Println("Running research agent...")
	res, err := agent.Run("Research the history of Rust programming language")
	if err != nil {
		log.Fatalf("Agent run failed: %v", err)
	}

	fmt.Println(res)
}