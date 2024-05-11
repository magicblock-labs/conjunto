use jsonrpsee::{
    core::{traits::ToRpcParams, JsonRawValue},
    types::Params,
};

pub struct RawParams(pub Params<'static>);

impl ToRpcParams for RawParams {
    fn to_rpc_params(
        self,
    ) -> Result<Option<Box<JsonRawValue>>, serde_json::Error> {
        match self.0.as_str() {
            Some(s) => {
                let raw_value = JsonRawValue::from_string(s.to_string())?;
                Ok(Some(raw_value))
            }
            None => Ok(None),
        }
    }
}

impl jsonrpsee_core::traits::ToRpcParams for RawParams {
    fn to_rpc_params(
        self,
    ) -> Result<Option<Box<JsonRawValue>>, serde_json::Error> {
        match self.0.as_str() {
            Some(s) => {
                let raw_value = JsonRawValue::from_string(s.to_string())?;
                Ok(Some(raw_value))
            }
            None => Ok(None),
        }
    }
}
