use super::{
    p2p_common::{CustomBehaviour, CustomNatStatus, GenericError},
    wrapper::Wrapper,
};
use libp2p::{Multiaddr, PeerId, Swarm};
use std::sync::Arc;

pub async fn libp2p_autonat_add_server(
    wrapper: &Arc<Wrapper>,
    peer: String,
    address: Option<String>,
) -> Result<(), GenericError> {
    let peer_unsafe = peer.parse::<PeerId>();
    match peer_unsafe {
        Ok(peer_safe) => {
            let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
            if let Some(swarm) = swarm_opt {
                let behaviour = swarm.behaviour_mut();
                let safe_address = address.map(|f| f.parse::<Multiaddr>().ok());
                let res = behaviour
                    .auto_nat
                    .add_server(peer_safe, safe_address.flatten());
                Ok(res)
            } else {
                Err(GenericError::InstanceNotFound)
            }
        }
        Err(e) => Err(GenericError::Other(e.to_string())),
    }
}

pub async fn libp2p_autonat_status(
    wrapper: &Arc<Wrapper>,
) -> Result<CustomNatStatus, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        let status = swarm.behaviour().auto_nat.nat_status();
        Ok(CustomNatStatus::from(status))
    } else {
        Err(GenericError::InstanceNotFound)
    }
}
