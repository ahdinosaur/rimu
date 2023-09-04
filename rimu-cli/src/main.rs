use std::{error::Error, io::Read, process::ExitCode, str::FromStr};

use clap::Parser;
use clio::*;
use rimu::{evaluate, parse, Environment, ReportError, SourceId, Value};

#[derive(Debug, Clone, Copy)]
enum Format {
    Yaml,
    Json,
    Toml,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "yaml" | "yml" => Ok(Format::Yaml),
            "json" => Ok(Format::Json),
            "toml" => Ok(Format::Toml),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(version)]
struct Args {
    #[clap(long, short, value_parser, default_value = "-")]
    input: Input,

    #[arg(long, short, value_parser)]
    env: Vec<Input>,

    #[clap(long, short, value_parser, default_value = "-")]
    output: Output,

    #[clap(long, short, default_value = "yaml")]
    format: Format,
}

fn main() -> std::result::Result<ExitCode, Box<dyn Error>> {
    let mut args = Args::parse();
    println!("args: {:?}", args);

    let mut input = String::new();
    args.input.read_to_string(&mut input)?;
    let input_source = SourceId::from_path(args.input.path().path());

    let default_env = Environment::new();
    let mut envs: Vec<Environment> = Vec::new();
    for mut env_arg in args.env {
        let mut env_string = String::new();
        env_arg.read_to_string(&mut env_string)?;
        let env_value: Value = env_string.into();

        let last_env = envs.last().unwrap_or(&default_env);
        let env = Environment::from_value(&env_value, Some(last_env))?;
        envs.push(env);
    }
    let env = envs.last().unwrap_or(&default_env);

    let (block, errors) = parse(input.as_str(), input_source.clone());

    if !errors.is_empty() {
        for error in errors {
            error.display(input.as_str(), input_source.clone());
        }
        return Ok(ExitCode::FAILURE);
    }

    let Some(block) = block else {
        println!("No block.");
        return Ok(ExitCode::FAILURE);
    };

    // println!("Block: {}", block);

    let value = match evaluate(&block, &env) {
        Ok(value) => value,
        Err(error) => {
            error.display(input.as_str(), input_source);
            return Ok(ExitCode::FAILURE);
        }
    };

    println!("Value: {}", value);

    Ok(ExitCode::SUCCESS)
}
