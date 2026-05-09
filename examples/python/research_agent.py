import sys
import os

# Add local path for testing
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../bindings/python')))

from openweave import Agent

def main():
    agent = Agent(system_prompt="You are a research agent.")
    print("Running research agent...")
    res = agent.run("Research the history of Rust programming language")
    print(res)

if __name__ == "__main__":
    main()