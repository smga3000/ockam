use ockam_core::{
    route, Address, Any, IncomingAccessControl, OutgoingAccessControl, Routed, Worker,
};
use ockam_node::Context;
use std::sync::Arc;
use tracing::trace;

use crate::kafka::inlet_controller::KafkaInletController;
use crate::kafka::portal_worker::KafkaPortalWorker;
use crate::kafka::protocol_aware::TopicUuidMap;
use crate::kafka::secure_channel_map::controller::KafkaSecureChannelControllerImpl;

/// First point of ingress of kafka connections, at the first message it spawns new stateful workers
/// to take care of the connection.
pub(crate) struct KafkaPortalListener {
    inlet_controller: KafkaInletController,
    secure_channel_controller: KafkaSecureChannelControllerImpl,
    uuid_to_name: TopicUuidMap,
    request_outgoing_access_control: Arc<dyn OutgoingAccessControl>,
    response_incoming_access_control: Arc<dyn IncomingAccessControl>,
    encrypt_content: bool,
}

#[ockam::worker]
impl Worker for KafkaPortalListener {
    type Message = Any;
    type Context = Context;

    async fn handle_message(
        &mut self,
        context: &mut Self::Context,
        message: Routed<Self::Message>,
    ) -> ockam::Result<()> {
        trace!("received first message!");

        let mut message = message.into_local_message();

        // Remove our address
        message = message.pop_front_onward_route()?;

        let next_hop = message.next_on_onward_route()?;

        // Retrieve the flow id from the next hop if it exists
        let flow_control_id = context
            .flow_controls()
            .find_flow_control_with_producer_address(&next_hop)
            .map(|x| x.flow_control_id().clone());

        let inlet_responder_address = message.return_route_ref().next()?.clone();

        let worker_address = KafkaPortalWorker::create_inlet_side_kafka_portal(
            context,
            self.encrypt_content,
            self.secure_channel_controller.clone(),
            self.uuid_to_name.clone(),
            self.inlet_controller.clone(),
            None,
            flow_control_id,
            route![inlet_responder_address],
            self.request_outgoing_access_control.clone(),
            self.response_incoming_access_control.clone(),
        )
        .await?;

        message = message.push_front_onward_route(&worker_address);

        trace!(
            "forwarding message: onward={:?}; return={:?}; worker={:?}",
            &message.onward_route_ref(),
            &message.return_route_ref(),
            worker_address
        );

        context.forward(message).await?;

        Ok(())
    }
}

impl KafkaPortalListener {
    pub(crate) async fn create(
        context: &Context,
        encrypt_content: bool,
        inlet_controller: KafkaInletController,
        secure_channel_controller: KafkaSecureChannelControllerImpl,
        listener_address: Address,
        incoming_access_control: Arc<dyn IncomingAccessControl>,
        outgoing_access_control: Arc<dyn OutgoingAccessControl>,
    ) -> ockam_core::Result<()> {
        let s = Self {
            inlet_controller,
            secure_channel_controller,
            uuid_to_name: Default::default(),
            request_outgoing_access_control: outgoing_access_control,
            response_incoming_access_control: incoming_access_control,
            encrypt_content,
        };

        context.start_worker(listener_address, s).await
    }
}
