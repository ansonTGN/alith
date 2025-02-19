# Alith Rust SDK

## Installation

```shell
cargo add alith --git https://github.com/0xLazAI/alith
```

## Quick Start

- Simple Agent

```rust
use alith::{Agent, LLM};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let model = LLM::from_model_name("gpt-4")?;
    let agent = Agent::new("simple agent", model, vec![])
        .preamble("You are a comedian here to entertain the user using humour and jokes.");

    let response = agent.prompt("Entertain me!").await?;

    println!("{}", response);

    Ok(())
}
```

- Agent with Tools

```rust
use alith::{Agent, StructureTool, Tool, ToolError, LLM};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct Input {
    pub x: usize,
    pub y: usize,
}

pub struct Adder;
#[async_trait]
impl StructureTool for Adder {
    type Input = Input;
    type Output = String;

    fn name(&self) -> &str {
        "adder"
    }

    fn description(&self) -> &str {
        "Add x and y together"
    }

    async fn run_with_args(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let result = input.x + input.y;
        Ok(result.to_string())
    }
}

pub struct Subtract;
#[async_trait]
impl StructureTool for Subtract {
    type Input = Input;
    type Output = String;

    fn name(&self) -> &str {
        "subtract"
    }

    fn description(&self) -> &str {
        "Subtract y from x (i.e.: x - y)"
    }

    async fn run_with_args(&self, input: Self::Input) -> Result<Self::Output, ToolError> {
        let result = input.x - input.y;
        Ok(result.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let tools: [Box<dyn Tool>; 2] = [Box::new(Adder), Box::new(Subtract)];
    let model = LLM::from_model_name("gpt-4")?;
    let agent = Agent::new("simple agent", model, tools)
        .preamble("You are a calculator here to help the user perform arithmetic operations. Use the tools provided to answer the user's question.");
    let response = agent.prompt("Calculate 10 - 3").await?;

    println!("{}", response);

    Ok(())
}
```

## Examples

See [here](../../crates/alith/examples/README.md) for more examples.
