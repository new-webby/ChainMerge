use crate::errors::DecodeError;
use crate::types::{DecodeRequest, NormalizedTransaction};

pub trait ChainDecoder {
    fn chain_name(&self) -> &'static str;
    fn decode(&self, request: &DecodeRequest) -> Result<NormalizedTransaction, DecodeError>;
}
