use jsonrpsee::{
    core::{traits::ToRpcParams, Error, JsonRawValue},
    types::Params,
};

pub struct RawParams(pub Params<'static>);

impl ToRpcParams for RawParams {
    fn to_rpc_params(self) -> Result<Option<Box<JsonRawValue>>, Error> {
        match self.0.as_str() {
            Some(s) => {
                let raw_value = JsonRawValue::from_string(s.to_string())?;
                Ok(Some(raw_value))
            }
            None => Ok(None),
        }
    }
}
