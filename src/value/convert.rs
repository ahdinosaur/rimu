use serde::{de::DeserializeOwned, Serialize};

use crate::value::{from_value, to_value, ValueError};

// https://stackoverflow.com/a/57488708
pub fn convert<Input, Output>(input: &Input) -> Result<Output, ValueError>
where
    Input: Serialize,
    Output: DeserializeOwned,
{
    let value = to_value(input)?;
    let output: Output = from_value(value)?;
    Ok(output)
}
