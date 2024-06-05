use crate::hole_punching::rendezvous_service::RendezvousClient;
use crate::hole_punching::side_channel::message::UdpHolePuncherNegotiationMessage;
use crate::hole_punching::side_channel::worker::UdpHolePuncherNegotiationWorker;
use crate::{UdpBindArguments, UdpBindOptions, UdpTransport};
use ockam_core::{async_trait, Address, Any, Decodable, Result, Route, Routed, Worker};
use ockam_node::Context;
use tracing::info;

/// FIXME
pub struct UdpHolePuncherNegotiationListener {
    udp: UdpTransport,
    rendezvous_route: Route,
}

impl UdpHolePuncherNegotiationListener {
    /// FIXME
    pub async fn create(
        ctx: &Context,
        address: impl Into<Address>,
        udp: &UdpTransport,
        rendezvous_route: Route,
    ) -> Result<()> {
        let s = Self {
            udp: udp.clone(),
            rendezvous_route,
        };

        ctx.start_worker(address, s).await?; // FIXME: Access Control

        Ok(())
    }
}

#[async_trait]
impl Worker for UdpHolePuncherNegotiationListener {
    type Message = Any;
    type Context = Context;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        info!("Received a UDP puncture request");

        let src_addr = msg.src_addr();
        let msg_payload = UdpHolePuncherNegotiationMessage::decode(msg.payload())?;

        if let UdpHolePuncherNegotiationMessage::Initiate { .. } = msg_payload {
            let address = Address::random_tagged("UdpHolePunctureNegotiator.responder");

            if let Some(producer_flow_control_id) = ctx
                .flow_controls()
                .get_flow_control_with_producer(&src_addr)
                .map(|x| x.flow_control_id().clone())
            {
                // Allow a sender with corresponding flow_control_id send messages to this address
                ctx.flow_controls()
                    .add_consumer(address.clone(), &producer_flow_control_id);
            }

            let udp_bind = self
                .udp
                .bind(
                    UdpBindArguments::new().with_bind_address("0.0.0.0:0")?,
                    UdpBindOptions::new(),
                )
                .await?;
            let client =
                RendezvousClient::new(ctx, &udp_bind, self.rendezvous_route.clone()).await?;

            let worker = UdpHolePuncherNegotiationWorker::new_responder(&udp_bind, client);

            let msg = msg
                .into_local_message()
                .pop_front_onward_route()?
                .push_front_onward_route(&address);

            ctx.start_worker(address, worker).await?; // FIXME: Access Control

            ctx.forward(msg).await?;
        }

        Ok(())
    }
}
