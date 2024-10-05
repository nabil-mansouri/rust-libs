pub use super::{
    p2p_common::{CustomBehaviour, GenericError},
    wrapper::Wrapper,
};
use flutter_rust_bridge::frb;
use libp2p::{swarm::ConnectionId, PeerId, Swarm};
use std::sync::Arc;
pub use tokio_util::sync::CancellationToken;

//
// CANCEL TOKEN
//
#[frb(external)]
#[frb(opaque)]
impl CancellationToken {
    #[frb(sync)]
    pub fn cancel(&self) {}
    #[frb(sync)]
    pub fn is_cancelled(&self) -> bool {}
}

#[frb(sync)]
pub fn create_cancellation_token() -> CancellationToken {
    return CancellationToken::new();
}

//
// LIB P2P UTILS
//
#[frb(sync)]
pub fn libp2p_peerid(wrapper: &Arc<Wrapper>) -> Result<String, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .local_peer_id()
        .to_string();
    Ok(res)
}
#[frb(sync)]
pub fn libp2p_peerid_random() -> String {
    PeerId::random().to_string()
}

#[frb(sync)]
pub fn libp2p_add_external_address(
    wrapper: &Arc<Wrapper>,
    address: String,
) -> Result<bool, GenericError> {
    if let Ok(multiaddr) = address.parse() {
        let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
        swarm
            .ok_or(GenericError::InstanceNotFound)?
            .add_external_address(multiaddr);
        Ok(true)
    } else {
        Err(GenericError::BadAddress)
    }
}

pub async fn libp2p_add_blacklist(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<bool, GenericError> {
    if let Ok(peerid) = peer.parse::<PeerId>() {
        let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
        swarm
            .ok_or(GenericError::InstanceNotFound)?
            .behaviour_mut()
            .blacklist
            .block_peer(peerid);
        Ok(true)
    } else {
        Err(GenericError::BadAddress)
    }
}

pub async fn libp2p_remove_blacklist(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<bool, GenericError> {
    if let Ok(peerid) = peer.parse::<PeerId>() {
        let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
        swarm
            .ok_or(GenericError::InstanceNotFound)?
            .behaviour_mut()
            .blacklist
            .unblock_peer(peerid);
        Ok(true)
    } else {
        Err(GenericError::BadAddress)
    }
}

pub async fn libp2p_close_connection(
    wrapper: &Arc<Wrapper>,
    connection_id: ConnectionId,
) -> Result<bool, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .close_connection(connection_id);
    Ok(res)
}
/*
pub async fn libp2p_add_whitelist(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<bool, GenericError> {
    if let Ok(peerid) = peer.parse::<PeerId>() {
        let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
        swarm
            .ok_or(GenericError::InstanceNotFound)?
            .behaviour_mut()
            .whitelist
            .allow_peer(peerid);
        Ok(true)
    } else {
        Err(GenericError::BadAddress)
    }
}

pub async fn libp2p_remove_whitelist(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<bool, GenericError> {
    if let Ok(peerid) = peer.parse::<PeerId>() {
        let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
        swarm
            .ok_or(GenericError::InstanceNotFound)?
            .behaviour_mut()
            .whitelist
            .disallow_peer(peerid);
        Ok(true)
    } else {
        Err(GenericError::BadAddress)
    }
}
*/
