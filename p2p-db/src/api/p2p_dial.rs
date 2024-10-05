use super::{
    p2p_common::{CustomBehaviour, GenericError},
    wrapper::Wrapper,
};
use libp2p::{swarm::{dial_opts::DialOpts, ConnectionId}, Multiaddr, PeerId, Swarm};
use std::borrow::Borrow;
use std::sync::Arc;

pub async fn libp2p_dial_address(
    wrapper: &Arc<Wrapper>,
    address: String,
) -> Result<ConnectionId, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        // dial address
        let multi_unsafe: Result<Multiaddr, _> = address.parse::<Multiaddr>();
        if let Ok(multiaddr) = multi_unsafe {
            let dial_opts: DialOpts = multiaddr.into();
            let id = dial_opts.connection_id();
            let res_dial = swarm.dial(dial_opts);
            match res_dial {
                Ok(_) => return Ok(id),
                Err(err) => return Err(GenericError::Other(err.to_string())),
            }
        } else {
            Err(GenericError::BadAddress)
        }
    } else {
        Err(GenericError::InstanceNotFound)
    }
}

pub async fn libp2p_dial_peer(
    wrapper: &Arc<Wrapper>,
    peerid: String,
) -> Result<ConnectionId, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        // dial peer
        let peer_unsafe: Result<PeerId, _> = peerid.parse::<PeerId>();
        if let Ok(peer) = peer_unsafe {
            let dial_opts: DialOpts = peer.into();
            let id = dial_opts.connection_id();
            let res_dial = swarm.dial(dial_opts);
            match res_dial {
                Ok(_) => return Ok(id),
                Err(err) => return Err(GenericError::Other(err.to_string())),
            }
        } else {
            Err(GenericError::BadAddress)
        }
    } else {
        Err(GenericError::InstanceNotFound)
    }
}

pub async fn libp2p_isconnected(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<bool, GenericError> {
    let peer_unsafe = peer.parse::<PeerId>();
    match peer_unsafe {
        Ok(peer_id) => {
            let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
            let res = swarm
                .ok_or(GenericError::InstanceNotFound)?
                .is_connected(peer_id.borrow());
            Ok(res)
        }
        Err(e) => Err(GenericError::Other(e.to_string())),
    }
}

pub async fn libp2p_connected_peers(
    wrapper: &Arc<Wrapper>,
) -> Result<Vec<String>, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .connected_peers().map(|e|e.to_string()).collect();
    Ok(res)
}

pub async fn libp2p_connected_peers_count(
    wrapper: &Arc<Wrapper>,
) -> Result<usize, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .connected_peers().count();
    Ok(res)
}

pub async fn libp2p_disconnect_peer(
    wrapper: &Arc<Wrapper>,
    peerid: String
) -> Result<bool, GenericError> {
    let peer_unsafe = peerid.parse::<PeerId>();
    match peer_unsafe {
        Ok(peer_id) => {
            let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
            let was_connected = swarm
                .ok_or(GenericError::InstanceNotFound)?
                .disconnect_peer_id(peer_id).map_or_else(|_|false, |_|true);
            Ok(was_connected)
        }
        Err(e) => Err(GenericError::Other(e.to_string())),
    }
}
