// Common code for both submit.rs and read.rs

use hyper::{Body, HeaderMap, Response, StatusCode};
use ic_crypto_tree_hash::Path;
use ic_crypto_tree_hash::{sparse_labeled_tree_from_paths, Label};
use ic_interfaces::state_manager::StateReader;
use ic_logger::{info, ReplicaLogger};
use ic_replicated_state::ReplicatedState;
use ic_types::{
    canonical_error::{invalid_argument_error, permission_denied_error, CanonicalError},
    messages::MessageId,
};
use ic_validator::RequestValidationError;
use prost::Message;
use serde::Serialize;
use std::sync::Arc;

pub const CONTENT_TYPE_HTML: &str = "text/html";
pub const CONTENT_TYPE_CBOR: &str = "application/cbor";
pub const CONTENT_TYPE_PROTOBUF: &str = "application/x-protobuf";

/// Add CORS headers to provided Response. In particular we allow
/// wildcard origin, POST and GET and allow Accept, Authorization and
/// Content Type headers.
pub(crate) fn get_cors_headers() -> HeaderMap {
    use hyper::header;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("POST, GET"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::HeaderValue::from_static("Accept, Authorization, Content-Type"),
    );
    headers
}

/// Convert an object into CBOR binary.
pub(crate) fn into_cbor<R: Serialize>(r: &R) -> Vec<u8> {
    let mut ser = serde_cbor::Serializer::new(Vec::new());
    ser.self_describe().expect("Could not write magic tag.");
    r.serialize(&mut ser).expect("Serialization failed.");
    ser.into_inner()
}

/// Write the "self describing" CBOR tag and serialize the response
pub(crate) fn cbor_response<R: Serialize>(r: &R) -> Response<Body> {
    use hyper::header;
    let mut response = Response::new(Body::from(into_cbor(r)));
    *response.status_mut() = StatusCode::OK;
    *response.headers_mut() = get_cors_headers();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(CONTENT_TYPE_CBOR),
    );
    response
}

/// Empty response.
pub(crate) fn empty_response() -> Response<Body> {
    let mut response = Response::new(Body::from(""));
    *response.status_mut() = StatusCode::NO_CONTENT;
    response
}

/// Encode the provided prost::Message implementing type as a protobuf Vec<u8>.
fn encode_as_protobuf_vec<R: Message>(r: &R) -> Vec<u8> {
    let mut buf = Vec::<u8>::new();
    r.encode(&mut buf)
        .expect("impossible: Serialization failed");
    buf
}

/// Write the provided prost::Message as a serialized protobuf into a Response
/// object.
pub(crate) fn protobuf_response<R: Message>(r: &R) -> Response<Body> {
    use hyper::header;
    let mut response = Response::new(Body::from(encode_as_protobuf_vec(r)));
    *response.status_mut() = StatusCode::OK;
    *response.headers_mut() = get_cors_headers();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static(CONTENT_TYPE_PROTOBUF),
    );
    response
}

pub(crate) fn make_response_on_validation_error(
    message_id: MessageId,
    err: RequestValidationError,
    log: &ReplicaLogger,
) -> CanonicalError {
    match err {
        RequestValidationError::InvalidIngressExpiry(msg)
        | RequestValidationError::InvalidDelegationExpiry(msg) => invalid_argument_error(&msg),
        _ => {
            let message = format!(
                "Failed to authenticate request {} due to: {}",
                message_id, err
            );
            info!(log, "{}", message);
            permission_denied_error(&message)
        }
    }
}

pub(crate) fn get_latest_certified_state(
    state_reader: &dyn StateReader<State = ReplicatedState>,
) -> Option<Arc<ReplicatedState>> {
    let paths = &mut [Path::from(Label::from("time"))];
    let labeled_tree = sparse_labeled_tree_from_paths(paths);
    state_reader
        .read_certified_state(&labeled_tree)
        .map(|r| r.0)
}

// A few test helpers, improving readability in the tests
#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use hyper::header;
    use ic_types::messages::{Blob, CertificateDelegation};
    use maplit::btreemap;
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use serde_cbor::Value;

    fn check_cors_headers(hm: &HeaderMap) {
        let acl_headers = hm.get_all(header::ACCESS_CONTROL_ALLOW_HEADERS).iter();
        assert!(acl_headers.eq(["Accept, Authorization, Content-Type"].iter()));
        let acl_methods = hm.get_all(header::ACCESS_CONTROL_ALLOW_METHODS).iter();
        assert!(acl_methods.eq(["POST, GET"].iter()));
        let acl_origin = hm.get_all(header::ACCESS_CONTROL_ALLOW_ORIGIN).iter();
        assert!(acl_origin.eq(["*"].iter()));
    }

    #[test]
    fn test_add_headers() {
        let hm = get_cors_headers();
        assert_eq!(hm.len(), 3);
        check_cors_headers(&hm);
    }

    #[test]
    fn test_cbor_response() {
        let response = cbor_response(b"");
        assert_eq!(response.headers().len(), 4);
        assert_eq!(
            response
                .headers()
                .get_all(header::CONTENT_TYPE)
                .iter()
                .count(),
            1
        );
        check_cors_headers(response.headers());
    }

    /// Makes sure that the serialized CBOR version of `obj` is the same as
    /// `Value`. Used when testing _outgoing_ messages from the HTTP
    /// Handler's point of view
    pub(crate) fn assert_cbor_ser_equal<T>(obj: &T, val: Value)
    where
        for<'de> T: Serialize,
    {
        assert_eq!(serde_cbor::value::to_value(obj).unwrap(), val)
    }

    pub(crate) fn text(text: &'static str) -> Value {
        Value::Text(text.to_string())
    }

    pub(crate) fn int(i: i128) -> Value {
        Value::Integer(i)
    }

    pub(crate) fn bytes(bs: &[u8]) -> Value {
        Value::Bytes(bs.to_vec())
    }

    pub(crate) fn array(values: Vec<Value>) -> Value {
        Value::Array(values)
    }

    #[test]
    fn encoding_delegation() {
        let delegation = CertificateDelegation {
            subnet_id: Blob(vec![1, 2, 3]),
            certificate: Blob(vec![4, 5, 6]),
        };
        assert_cbor_ser_equal(
            &delegation,
            Value::Map(btreemap! {
                text("subnet_id") => bytes(&[1, 2, 3]),
                text("certificate") => bytes(&[4, 5, 6]),
            }),
        );
    }
}
