use std::{
    error::Error,
    io::{Read, Write},
    process::ExitCode,
    str::FromStr,
};

use clap::Parser;
use clio::*;
use rimu::{evaluate, parse, Environment, ErrorReport, SourceId, Value};

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
    #[clap(long, short, value_parser)]
    input: Input,

    #[arg(long, short, value_parser)]
    env: Option<Input>,

    #[clap(long, short, value_parser, default_value = "-")]
    output: Output,

    #[clap(long, short, default_value = "yaml")]
    format: Format,
}

fn main() -> std::result::Result<ExitCode, Box<dyn Error>> {
    let mut args = Args::parse();

    let mut input = String::new();
    args.input.read_to_string(&mut input)?;
    let input_source = SourceId::from_path(args.input.path().path());

    let env = if let Some(mut env_arg) = args.env {
        let mut env_string = String::new();
        env_arg.read_to_string(&mut env_string)?;
        let env_value: Value = env_string.into();
        Environment::from_value(&env_value, None)?
    } else {
        Environment::new()
    };

    let (block, errors) = parse(input.as_str(), input_source.clone());

    if !errors.is_empty() {
        for error in errors {
            let report: ErrorReport = error.into();
            report.display(input.as_str(), input_source.clone());
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
            let report: ErrorReport = error.into();
            report.display(input.as_str(), input_source.clone());
            return Ok(ExitCode::FAILURE);
        }
    };

    let output: String = match args.format {
        Format::Yaml => serde_yaml::to_string(&value)?,
        Format::Json => serde_json::to_string(&value)?,
        Format::Toml => toml::to_string(&value)?,
    };

    args.output.write_all(output.as_bytes())?;

    Ok(ExitCode::SUCCESS)
}
