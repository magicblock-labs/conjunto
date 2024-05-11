use jsonrpsee::{
    core::{traits::ToRpcParams, Error, JsonRawValue},
    types::Params,
};

#[derive(Clone, Debug)]
pub struct RawParams(pub Params<'static>);
impl RawParams {
    pub fn new(params: Option<&'static str>) -> Self {
        Self(Params::new(params))
    }
}

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
