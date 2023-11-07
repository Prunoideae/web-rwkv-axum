use anyhow::{Error, Result};
use serde_json::Value;

fn all_bool(state_ids: &Vec<String>, value: bool) -> Vec<bool> {
    state_ids.iter().map(|_| value).collect()
}

pub fn state_flag_from_value(state_ids: &Vec<String>, value: Option<Value>) -> Result<Vec<bool>> {
    if let Some(value) = value {
        if value.is_array() {
            let flags = serde_json::from_value::<Vec<bool>>(value)?;
            if flags.len() != state_ids.len() {
                return Err(Error::msg(
                    "update_state should have same length as state_ids!",
                ));
            }
            return Ok(flags);
        }
        match value {
            Value::Bool(flag) => Ok(all_bool(state_ids, flag)),
            Value::Null => Ok(all_bool(state_ids, true)),
            _ => Err(Error::msg(
                "update_state must be a bool, null or an array of bool!",
            )),
        }
    } else {
        Ok(all_bool(state_ids, true))
    }
}
