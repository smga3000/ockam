use crate::kafka::outlet_controller::KafkaOutletController;
use alloc::sync::Arc;
use bytes::{Bytes, BytesMut};
use kafka_protocol::messages::fetch_request::FetchRequest;
use kafka_protocol::messages::produce_request::ProduceRequest;
use kafka_protocol::messages::request_header::RequestHeader;
use kafka_protocol::messages::{ApiKey, MetadataResponse, ResponseHeader};
use kafka_protocol::protocol::buf::ByteBuf;
use kafka_protocol::protocol::Decodable;
use kafka_protocol::records::{
    Compression, RecordBatchDecoder, RecordBatchEncoder, RecordEncodeOptions,
};
use minicbor::encode::Encoder;
use ockam_core::async_trait;
use ockam_core::compat::collections::HashMap;
use ockam_core::compat::sync::Mutex;
#[cfg(feature = "tag")]
use ockam_core::TypeTag;
use ockam_node::Context;
use std::convert::TryFrom;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use tinyvec::alloc;
use tracing::warn;

use crate::kafka::portal_worker::InterceptError;
use crate::kafka::protocol_aware::utils::{decode_body, encode_request};
use crate::kafka::protocol_aware::{
    CorrelationId, InletInterceptorImpl, KafkaMessageInterceptor, MessageWrapper, RequestInfo,
};

#[derive(Clone)]
pub(crate) struct OutletInterceptorImpl {
    request_map: Arc<Mutex<HashMap<CorrelationId, RequestInfo>>>,
    outlet_controller: KafkaOutletController,
}

impl OutletInterceptorImpl {
    pub(crate) fn new(outlet_controller: KafkaOutletController) -> Self {
        Self {
            request_map: Arc::new(Mutex::new(HashMap::new())),
            outlet_controller,
        }
    }
}

#[async_trait]
impl KafkaMessageInterceptor for OutletInterceptorImpl {
    async fn intercept_request(
        &self,
        _context: &mut Context,
        mut original: BytesMut,
    ) -> Result<BytesMut, InterceptError> {
        let mut buffer = original.peek_bytes(0..original.len());

        let api_key_num = buffer
            .peek_bytes(0..2)
            .try_get_i16()
            .map_err(|_| InterceptError::Io(Error::from(ErrorKind::InvalidData)))?;

        let api_key = ApiKey::try_from(api_key_num).map_err(|_| {
            warn!("unknown request api: {api_key_num}");
            InterceptError::Io(Error::from(ErrorKind::InvalidData))
        })?;

        let version = buffer
            .peek_bytes(2..4)
            .try_get_i16()
            .map_err(|_| InterceptError::Io(Error::from(ErrorKind::InvalidData)))?;

        let result = RequestHeader::decode(&mut buffer, api_key.request_header_version(version));
        let header = match result {
            Ok(header) => header,
            Err(_) => {
                //the error doesn't contain any useful information
                warn!("cannot decode request kafka header");
                return Err(InterceptError::Io(Error::from(ErrorKind::InvalidData)));
            }
        };

        let api_key = ApiKey::try_from(api_key_num).map_err(|_| {
            warn!("unknown request api: {api_key_num}");
            InterceptError::Io(Error::from(ErrorKind::InvalidData))
        })?;

        debug!(
            "request: length: {}, correlation {}, version {}, api {:?}",
            buffer.len(),
            header.correlation_id,
            header.request_api_version,
            api_key
        );

        if api_key == ApiKey::MetadataKey {
            self.request_map.lock().unwrap().insert(
                header.correlation_id,
                RequestInfo {
                    request_api_key: ApiKey::MetadataKey,
                    request_api_version: header.request_api_version,
                },
            );
        }

        Ok(original)
    }

    async fn intercept_response(
        &self,
        context: &mut Context,
        mut original: BytesMut,
    ) -> Result<BytesMut, InterceptError> {
        let mut buffer = original.peek_bytes(0..original.len());

        //we can/need to decode only mapped requests
        let correlation_id = buffer
            .peek_bytes(0..4)
            .try_get_i32()
            .map_err(|_| InterceptError::Io(Error::from(ErrorKind::InvalidData)))?;

        let result = self
            .request_map
            .lock()
            .unwrap()
            .get(&correlation_id)
            .cloned();

        if let Some(request_info) = result {
            let result = ResponseHeader::decode(
                &mut buffer,
                request_info
                    .request_api_key
                    .response_header_version(request_info.request_api_version),
            );

            let _header = match result {
                Ok(header) => header,
                Err(_) => {
                    //the error doesn't contain any useful information
                    warn!("cannot decode response kafka header");
                    return Err(InterceptError::Io(Error::from(ErrorKind::InvalidData)));
                }
            };

            debug!(
                "response: length: {}, correlation {}, version {}, api {:?}",
                buffer.len(),
                correlation_id,
                request_info.request_api_version,
                request_info.request_api_key
            );

            if request_info.request_api_key == ApiKey::MetadataKey {
                let mut response: MetadataResponse =
                    decode_body(&mut buffer, request_info.request_api_version)?;

                for (broker_id, metadata) in response.brokers {
                    self.outlet_controller
                        .assert_outlet_for_broker(
                            context,
                            broker_id.0,
                            format!("{}:{}", metadata.host, metadata.port),
                        )
                        .await
                        .map_err(InterceptError::Ockam)?;
                }
            }
        } else {
            debug!(
                "response unmapped: length: {}, correlation {}",
                buffer.len(),
                correlation_id,
            );
        }
        Ok(original)
    }
}
