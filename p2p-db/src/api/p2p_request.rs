use super::{
    p2p_common::{CustomBehaviour, GenericError},
    wrapper::Wrapper,
};
use libp2p::{request_response::ResponseChannel, Swarm};
use std::borrow::Borrow;
use std::sync::Arc;

pub async fn libp2p_send_request(
    wrapper: &Arc<Wrapper>,
    peerid: String,
    request: Vec<u8>,
) -> Result<String, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        if let Ok(peer) = peerid.parse().borrow() {
            let behaviour = swarm.behaviour_mut();
            let res = behaviour.request_response.send_request(peer, request);
            Ok(res.to_string())
        } else {
            Err(GenericError::BadAddress)
        }
    } else {
        Err(GenericError::InstanceNotFound)
    }
}

pub async fn libp2p_send_response(
    wrapper: &Arc<Wrapper>,
    channel: ResponseChannel<Vec<u8>>,
    response: Vec<u8>,
) -> Result<(), GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        let behaviour = swarm.behaviour_mut();
        let res = behaviour.request_response.send_response(channel, response);
        let res_safe = res.map_err(|e| GenericError::Bytes(e));
        res_safe
    } else {
        Err(GenericError::InstanceNotFound)
    }
}

pub async fn libp2p_close_response(channel: ResponseChannel<Vec<u8>>) -> () {
    drop(channel);
}
