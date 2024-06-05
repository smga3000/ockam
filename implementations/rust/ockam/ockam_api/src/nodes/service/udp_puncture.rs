use ockam::udp::{UdpHolePuncherNegotiation, UdpHolePuncherNegotiationListener};
use ockam::{Result, Route};
use ockam_core::route;

use crate::nodes::service::default_address::DefaultAddress;
use crate::nodes::NodeManager;

/// SECURE CHANNELS
impl NodeManager {
    #[allow(clippy::too_many_arguments)]
    pub async fn create_udp_puncture(&self, rendezvous_route: Route) -> Result<()> {
        debug!("Starting UDP puncture negotiation",);

        // FIXME
        let _receiver = UdpHolePuncherNegotiation::start_negotiation(
            self.udp_transport.ctx(),
            route![], // FIXME
            &self.udp_transport,
            rendezvous_route,
        )
        .await?;

        Ok(())
    }
}

/// SECURE CHANNEL LISTENERS
impl NodeManager {
    pub async fn create_udp_puncture_listener(&self, rendezvous_route: Route) -> Result<()> {
        debug!(
            "Starting UDP puncture listener at: {}",
            DefaultAddress::UDP_HOLE_PUNCHER_LISTENER
        );

        UdpHolePuncherNegotiationListener::create(
            self.udp_transport.ctx(),
            DefaultAddress::UDP_HOLE_PUNCHER_LISTENER,
            &self.udp_transport,
            rendezvous_route,
        )
        .await?;

        // TODO: Access control
        // TODO: Flow control
        // TODO: Registry

        Ok(())
    }
}
