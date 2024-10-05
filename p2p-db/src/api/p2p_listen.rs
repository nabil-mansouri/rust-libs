use std::sync::Arc;
use libp2p::{core::transport::ListenerId, Swarm};
use super::{
    p2p_common::{CustomBehaviour, GenericError},
    wrapper::Wrapper,
};

pub async fn libp2p_listen(
    wrapper: &Arc<Wrapper>,
    address: String,
) -> Result<ListenerId, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        if let Ok(multiaddr) = address.parse() {
            let res = swarm.listen_on(multiaddr);
            match res {
                Ok(res_ok) => return Ok(res_ok),
                Err(err) => return Err(GenericError::Other(err.to_string())),
            }
        } else {
            Err(GenericError::BadAddress)
        }
    } else {
        Err(GenericError::InstanceNotFound)
    }
}
pub async fn libp2p_unlisten(
    wrapper: &Arc<Wrapper>,
    listener_id: &ListenerId,
) -> Result<bool, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        let res = swarm.remove_listener(*listener_id);
        Ok(res)
    } else {
        Err(GenericError::InstanceNotFound)
    }
}
