use std::sync::Arc;
use flutter_rust_bridge::frb;
use libp2p::{rendezvous::Namespace, Swarm};
use super::{
    p2p_common::{CustomBehaviour, GenericError},
    wrapper::Wrapper,
};
pub use libp2p::rendezvous::Cookie;

#[frb(external)]
#[frb(opaque)]
impl Cookie {
    #[frb(sync)]
    pub fn clone(&self) -> Self {}
}
#[frb(sync)]
pub fn libp2p_rdv_cookie() -> Cookie {
    Cookie::for_all_namespaces()
}

pub async fn libp2p_rdv_discover(
    wrapper: &Arc<Wrapper>,
    rdv_peerid: String,
    limit: Option<u64>,
    cookie: Option<Cookie>,
) -> Result<bool, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        if let Ok(multiaddr) = rdv_peerid.parse() {
            let behaviour = swarm.behaviour_mut();
            behaviour.rdv_client.discover(None, cookie, limit, multiaddr);
            return Ok(true);
        } else {
            Err(GenericError::BadAddress)
        }
    } else {
        Err(GenericError::InstanceNotFound)
    }
}

pub async fn libp2p_rdv_register(
    wrapper: &Arc<Wrapper>,
    rdv_peerid: String,
    namespace: String,
    ttl: Option<u64>,
) -> Result<bool, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    if let Some(swarm) = swarm_opt {
        if let Ok(multiaddr) = rdv_peerid.parse() {
            let ns_unsafe = Namespace::new(namespace);
            match ns_unsafe {
                Ok(ns) => {
                    let behaviour = swarm.behaviour_mut();
                    let res = behaviour.rdv_client.register(ns, multiaddr, ttl);
                    let safe = res
                        .map(|_| true)
                        .map_err(|e| GenericError::Other(e.to_string()));
                    safe
                }
                Err(err) => return Err(GenericError::Other(err.to_string())),
            }
        } else {
            Err(GenericError::BadAddress)
        }
    } else {
        Err(GenericError::InstanceNotFound)
    }
}
