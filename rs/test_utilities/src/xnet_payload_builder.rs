use ic_interfaces::messaging::{XNetPayloadBuilder, XNetPayloadValidationError};
use ic_types::{
    batch::{ValidationContext, XNetPayload},
    xnet::CertifiedStreamSlice,
    NumBytes, SubnetId,
};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct FakeXNetPayloadBuilder(BTreeMap<SubnetId, CertifiedStreamSlice>);

impl FakeXNetPayloadBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn make(provided_streams: BTreeMap<SubnetId, CertifiedStreamSlice>) -> Self {
        Self(provided_streams)
    }
}

impl XNetPayloadBuilder for FakeXNetPayloadBuilder {
    fn get_xnet_payload(
        &self,
        _validation_context: &ValidationContext,
        _past_payloads: &[&XNetPayload],
        _byte_limit: NumBytes,
    ) -> XNetPayload {
        XNetPayload {
            stream_slices: self.0.clone(),
        }
    }

    fn validate_xnet_payload(
        &self,
        _payload: &XNetPayload,
        _validation_context: &ValidationContext,
        _past_payloads: &[&XNetPayload],
    ) -> Result<NumBytes, XNetPayloadValidationError> {
        Ok(0.into())
    }
}
