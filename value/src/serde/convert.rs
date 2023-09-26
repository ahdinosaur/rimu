use serde::{de::DeserializeOwned, Serialize};

use super::{from_serde_value, to_serde_value, SerdeValueError};

// https://stackoverflow.com/a/57488708
pub fn convert<Input, Output>(input: &Input) -> Result<Output, SerdeValueError>
where
    Input: Serialize,
    Output: DeserializeOwned,
{
    let value = to_serde_value(input)?;
    let output: Output = from_serde_value(value)?;
    Ok(output)
}
