use super::p2p_common::{CustomBehaviour, GenericError};
pub use crate::api::wrapper::Wrapper;
use flutter_rust_bridge::frb;
pub use libp2p::gossipsub::{
    self, Behaviour as GossipBehaviour, ConfigBuilder, MessageAcceptance, MessageAuthenticity,
    MessageId, ValidationMode,
};
pub use libp2p::identity::PeerId;
use libp2p::Swarm;
use std::borrow::Borrow;
use std::sync::Arc;
//
// ENUMS
//

#[derive(Debug)]
#[frb(external)]
#[frb(non_opaque)]
pub enum CustomMessageAcceptance {
    Reject,
    Ignore,
    Accept,
}

impl CustomMessageAcceptance {
    fn to_acceptance(self) -> MessageAcceptance {
        match self {
            CustomMessageAcceptance::Ignore => MessageAcceptance::Ignore,
            CustomMessageAcceptance::Reject => MessageAcceptance::Reject,
            CustomMessageAcceptance::Accept => MessageAcceptance::Accept,
        }
    }
}
//
// FUNCTIONS
//
pub async fn libp2p_unsubscribe(
    wrapper: &Arc<Wrapper>,
    topic: String,
) -> Result<bool, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let into_topic = gossipsub::IdentTopic::new(topic);
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .behaviour_mut()
        .pubsub
        .unsubscribe(into_topic.borrow())
        .map_err(|e| GenericError::Other(e.to_string()));
    res
}

pub async fn libp2p_subscribe(
    wrapper: &Arc<Wrapper>,
    topic: String,
) -> Result<Option<String>, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let into_topic = gossipsub::IdentTopic::new(topic);
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .behaviour_mut()
        .pubsub
        .subscribe(into_topic.borrow())
        .map_err(|e| GenericError::Other(e.to_string()));
    res.map(|f| {
        if f {
            Some(into_topic.hash().to_string())
        } else {
            None
        }
    })
}
pub async fn libp2p_publish(
    wrapper: &Arc<Wrapper>,
    topic: String,
    data: Vec<u8>,
) -> Result<MessageId, GenericError> {
    let swarm: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    let into_topic = gossipsub::IdentTopic::new(topic);
    let res = swarm
        .ok_or(GenericError::InstanceNotFound)?
        .behaviour_mut()
        .pubsub
        .publish(into_topic, data)
        .map_err(|e| GenericError::Other(e.to_string()));
    res
}

#[frb(sync)]
pub fn libp2p_topic_tohash(topic: String) -> String {
    let into_topic = gossipsub::IdentTopic::new(topic);
    return into_topic.hash().to_string();
}

pub async fn libp2p_pubsub_add_peer(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<(), GenericError> {
    let peer_unsafe = peer.parse::<PeerId>();
    match peer_unsafe {
        Ok(peer_id) => {
            let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
            if let Some(swarm) = swarm_opt {
                let behaviour = swarm.behaviour_mut();
                let res = behaviour.pubsub.add_explicit_peer(peer_id.borrow());
                Ok(res)
            } else {
                Err(GenericError::InstanceNotFound)
            }
        }
        Err(e) => Err(GenericError::Other(e.to_string())),
    }
}

pub async fn libp2p_pubsub_remove_peer(
    wrapper: &Arc<Wrapper>,
    peer: String,
) -> Result<(), GenericError> {
    let peer_unsafe = peer.parse::<PeerId>();
    match peer_unsafe {
        Ok(peer_id) => {
            let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
            if let Some(swarm) = swarm_opt {
                let behaviour = swarm.behaviour_mut();
                let res = behaviour.pubsub.remove_explicit_peer(peer_id.borrow());
                Ok(res)
            } else {
                Err(GenericError::InstanceNotFound)
            }
        }
        Err(e) => Err(GenericError::Other(e.to_string())),
    }
}

pub async fn libp2p_pubsub_validate(
    wrapper: &Arc<Wrapper>,
    msg_id: Vec<u8>,
    propagation_source: String,
    acceptance: CustomMessageAcceptance,
) -> Result<bool, GenericError> {
    let peer_unsafe = propagation_source.parse::<PeerId>();
    match peer_unsafe {
        Ok(propagation_source_safe) => {
            let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
            if let Some(swarm) = swarm_opt {
                let behaviour = swarm.behaviour_mut();
                let msg_id_safe = MessageId::new(msg_id.borrow());
                let acceptance_safe = acceptance.to_acceptance();
                let res = behaviour.pubsub.report_message_validation_result(
                    msg_id_safe.borrow(),
                    propagation_source_safe.borrow(),
                    acceptance_safe,
                );
                let res_safe = res.map_err(|e| GenericError::Other(e.to_string()));
                res_safe
            } else {
                Err(GenericError::InstanceNotFound)
            }
        }
        Err(e) => Err(GenericError::Other(e.to_string())),
    }
}
