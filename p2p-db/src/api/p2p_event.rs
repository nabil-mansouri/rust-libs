use super::{
    p2p_common::{CustomBehaviour, CustomBehaviourEvent, CustomPeerRecord, GenericError},
    wrapper::Wrapper,
};
use crate::api::p2p_common::{CustomNatStatus, CustomSwarmEvent};
use flutter_rust_bridge::{frb, DartFnFuture, RustAutoOpaqueNom};
use libp2p::futures::StreamExt;
use libp2p::{autonat, gossipsub, identify, request_response};
use libp2p::{swarm::SwarmEvent, Swarm};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[frb(sync)]
pub fn libp2p_get_event_name(event: CustomSwarmEvent) -> String {
    let right: &str = event.as_ref();
    return right.to_string();
}
pub async fn libp2p_add_event_listener(
    wrapper: &Arc<Wrapper>,
    cancellation_token: &CancellationToken,
    callback: impl Fn(CustomSwarmEvent) -> DartFnFuture<()> + Send + 'static,
) -> Result<bool, GenericError> {
    let swarm_opt: Option<&mut Swarm<CustomBehaviour>> = wrapper.as_mut();
    match swarm_opt {
        None => return Err(GenericError::InstanceNotFound),
        Some(swarm) => {
            let boxed_callback: Box<dyn Fn(CustomSwarmEvent) -> DartFnFuture<()> + Send + 'static> =
                Box::new(callback);
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        return Ok(false);
                    },
                    unsafe_event = swarm.next() => {
                        match unsafe_event {
                            None => return Ok(true),
                            Some(event) => {
                                match event {
                                    SwarmEvent::Behaviour(custom) => match custom {
                                        //CustomBehaviourEvent::Whitelist{..} => {},
                                        CustomBehaviourEvent::Blacklist{..} => {},
                                        CustomBehaviourEvent::MemoryLimits{..} => {},
                                        CustomBehaviourEvent::ConnectionLimits{..} => {},
                                        CustomBehaviourEvent::Upnp(e)=>{
                                            match e {
                                                libp2p::upnp::Event::NewExternalAddr(val) => {
                                                    boxed_callback(CustomSwarmEvent::UpnpNewExternalAddr(val.to_string()))
                                                    .await;
                                                    ()
                                                },
                                                libp2p::upnp::Event::ExpiredExternalAddr(val) => {
                                                    boxed_callback(CustomSwarmEvent::UpnpExpiredExternalAddr(val.to_string()))
                                                    .await;
                                                    ()
                                                },
                                                libp2p::upnp::Event::GatewayNotFound => {
                                                    boxed_callback(CustomSwarmEvent::UpnpGatewayNotFound)
                                                    .await;
                                                    ()
                                                },
                                                libp2p::upnp::Event::NonRoutableGateway => {
                                                    boxed_callback(CustomSwarmEvent::UpnpNonRoutableGateway)
                                                    .await;
                                                    ()
                                                },
                                            }
                                        },
                                        CustomBehaviourEvent::AutoNat(e) => {
                                            match e {
                                                autonat::Event::InboundProbe(_) => {},
                                                autonat::Event::OutboundProbe(_) => {},
                                                autonat::Event::StatusChanged { old, new } => {
                                                    boxed_callback(CustomSwarmEvent::NATStatusChanged {
                                                        new_value: CustomNatStatus::from(new),
                                                        old_value: CustomNatStatus::from(old),
                                                    })
                                                    .await;
                                                    ()
                                                },
                                            }
                                        },
                                        CustomBehaviourEvent::Dcutr(_) => {},
                                        CustomBehaviourEvent::Identify(id_event) => match id_event {
                                            identify::Event::Received { peer_id, info } => {
                                                boxed_callback(CustomSwarmEvent::IdentifyReceived {
                                                    peer_id: peer_id.to_string(),
                                                    public_key: info.public_key.encode_protobuf(),
                                                    protocol_version: info.protocol_version,
                                                    agent_version: info.agent_version,
                                                    listen_addrs: info
                                                        .listen_addrs
                                                        .iter()
                                                        .map(|f| f.to_string())
                                                        .collect(),
                                                    protocols: info
                                                        .protocols
                                                        .iter()
                                                        .map(|f| f.to_string())
                                                        .collect(),
                                                    observed_addr: info.observed_addr.to_string(),
                                                })
                                                .await;
                                                ()
                                            }
                                            identify::Event::Sent { .. } => {}
                                            identify::Event::Pushed { .. } => {}
                                            identify::Event::Error { .. } => {}
                                        },
                                        CustomBehaviourEvent::RequestResponse(req_event) => {
                                            match req_event {
                                                request_response::Event::Message { peer, message } => match message
                                                {
                                                    request_response::Message::Request {
                                                        request_id,
                                                        request,
                                                        channel,
                                                    } => {
                                                        boxed_callback(CustomSwarmEvent::RequestMessage {
                                                            request_id: request_id.to_string(),
                                                            peer: peer.to_string(),
                                                            message: request,
                                                            channel: RustAutoOpaqueNom::new(channel),
                                                        })
                                                        .await;
                                                        ()
                                                    }
                                                    request_response::Message::Response {
                                                        request_id,
                                                        response,
                                                    } => {
                                                        boxed_callback(CustomSwarmEvent::ResponseMessage {
                                                            request_id: request_id.to_string(),
                                                            peer: peer.to_string(),
                                                            message: response,
                                                        })
                                                        .await;
                                                        ()
                                                    }
                                                },
                                                request_response::Event::OutboundFailure {
                                                    peer,
                                                    request_id,
                                                    error,
                                                } => {
                                                    boxed_callback(CustomSwarmEvent::RequestOutboundFailure {
                                                        peer: peer.to_string(),
                                                        request_id: request_id.to_string(),
                                                        error: error.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                                request_response::Event::InboundFailure {
                                                    peer,
                                                    request_id,
                                                    error,
                                                } => {
                                                    boxed_callback(CustomSwarmEvent::RequestInboundFailure {
                                                        peer: peer.to_string(),
                                                        request_id: request_id.to_string(),
                                                        error: error.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                                request_response::Event::ResponseSent { peer, request_id } => {
                                                    boxed_callback(CustomSwarmEvent::ResponseSent {
                                                        peer: peer.to_string(),
                                                        request_id: request_id.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                            }
                                        }
                                        CustomBehaviourEvent::RdvServer(rdv_event) => {
                                            match rdv_event {
                                                libp2p::rendezvous::server::Event::DiscoverServed { .. } => {},
                                                libp2p::rendezvous::server::Event::DiscoverNotServed { .. } => {},
                                                libp2p::rendezvous::server::Event::PeerNotRegistered { .. } => {},
                                                libp2p::rendezvous::server::Event::PeerRegistered { peer, registration } => {
                                                    boxed_callback(CustomSwarmEvent::RdvServerPeerRegistered { peerid: peer.to_string(), addresses: registration.record.addresses().iter().map(|e|e.to_string()).collect() } )
                                                    .await;
                                                    ()
                                                },
                                                libp2p::rendezvous::server::Event::PeerUnregistered { peer, .. } => {
                                                    boxed_callback(CustomSwarmEvent::RdvServerPeerUnRegistered { peerid: peer.to_string() } )
                                                    .await;
                                                    ()
                                                },
                                                libp2p::rendezvous::server::Event::RegistrationExpired(registration) => {
                                                    boxed_callback(CustomSwarmEvent::RdvServerPeerUnRegistered { peerid: registration.record.peer_id().to_string() } )
                                                    .await;
                                                    ()
                                                },
                                            }
                                        }
                                        CustomBehaviourEvent::RdvClient(rdv_event) => match rdv_event
                                        {
                                            libp2p::rendezvous::client::Event::Discovered {
                                                rendezvous_node,
                                                registrations,
                                                ..
                                            } => {
                                                boxed_callback(CustomSwarmEvent::RdvClientDiscovered {
                                                    rendezvous_node: rendezvous_node.to_string(),
                                                    registrations: registrations
                                                        .iter()
                                                        .map(|f| CustomPeerRecord{
                                                            peer_id:f.record.peer_id().to_string(),
                                                            addresses: f.record.addresses().iter().map(|e|e.to_string()).collect()
                                                        })
                                                        .collect(),
                                                })
                                                .await;
                                                ()
                                            }
                                            libp2p::rendezvous::client::Event::DiscoverFailed { rendezvous_node,.. } => {
                                                boxed_callback(CustomSwarmEvent::RdvClientDiscoveryFail { rendezvous_node: rendezvous_node.to_string() })
                                                .await;
                                                ()
                                            }
                                            libp2p::rendezvous::client::Event::Registered { rendezvous_node, .. } => {
                                                boxed_callback(CustomSwarmEvent::RdvClientRegistered { rendezvous_node: rendezvous_node.to_string() })
                                                .await;
                                                ()
                                            }
                                            libp2p::rendezvous::client::Event::RegisterFailed { rendezvous_node,.. } => {
                                                boxed_callback(CustomSwarmEvent::RdvClientRegisteredFailed { rendezvous_node: rendezvous_node.to_string() })
                                                .await;
                                                ()
                                            }
                                            libp2p::rendezvous::client::Event::Expired { peer } => {
                                                boxed_callback(CustomSwarmEvent::RdvClientDiscoveryExpired { peer_id: peer.to_string() })
                                                .await;
                                                ()
                                            }
                                        },
                                        CustomBehaviourEvent::RelayServer(_) => {}
                                        CustomBehaviourEvent::RelayClient(_) => {}
                                        CustomBehaviourEvent::Pubsub(pub_event) => {
                                            match pub_event {
                                                gossipsub::Event::Message {
                                                    propagation_source,
                                                    message_id,
                                                    message,
                                                } => {
                                                    boxed_callback(CustomSwarmEvent::GossipMessage {
                                                        propagation_source: propagation_source.to_string(),
                                                        message_id: message_id.0,
                                                        message: message.data,
                                                        source: message.source.map(|f|f.to_string()),
                                                        topic_hash: message.topic.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                                gossipsub::Event::Subscribed { peer_id, topic } => {
                                                    boxed_callback(CustomSwarmEvent::GossipSubscribed {
                                                        peer_id: peer_id.to_string(),
                                                        topic: topic.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                                gossipsub::Event::Unsubscribed { peer_id, topic } => {
                                                    boxed_callback(CustomSwarmEvent::GossipUnsubscribed {
                                                        peer_id: peer_id.to_string(),
                                                        topic: topic.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                                gossipsub::Event::GossipsubNotSupported { peer_id } => {
                                                    boxed_callback(CustomSwarmEvent::GossipsubNotSupported {
                                                        peer_id: peer_id.to_string(),
                                                    })
                                                    .await;
                                                    ()
                                                }
                                            };
                                        }
                                    },
                                    SwarmEvent::ConnectionEstablished {
                                        peer_id,
                                        endpoint,
                                        num_established,
                                        established_in,
                                        connection_id,..
                                    } => {
                                        boxed_callback(CustomSwarmEvent::ConnectionEstablished {
                                            peer_id: peer_id.to_string(),
                                            endpoint: endpoint.get_remote_address().to_string(),
                                            num_established: num_established.get(),
                                            established_in: established_in.as_millis(),
                                            connection_id,
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::ConnectionClosed {
                                        peer_id,
                                        endpoint,
                                        num_established,
                                        cause,
                                        connection_id
                                    } => {
                                        boxed_callback(CustomSwarmEvent::ConnectionClosed {
                                            peer_id: peer_id.to_string(),
                                            endpoint: endpoint.get_remote_address().to_string(),
                                            num_established: num_established,
                                            cause: cause.map(|f| f.to_string()),
                                            connection_id
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::IncomingConnection {
                                        local_addr,
                                        send_back_addr,
                                        connection_id
                                    } => {
                                        boxed_callback(CustomSwarmEvent::IncomingConnection {
                                            local_addr: local_addr.to_string(),
                                            send_back_addr: send_back_addr.to_string(),
                                            connection_id
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::IncomingConnectionError {
                                        local_addr,
                                        send_back_addr,
                                        error,
                                        connection_id
                                    } => {
                                        boxed_callback(CustomSwarmEvent::IncomingConnectionError {
                                            local_addr: local_addr.to_string(),
                                            send_back_addr: send_back_addr.to_string(),
                                            error: error.to_string(),
                                            connection_id
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::OutgoingConnectionError { peer_id, error, connection_id} => {
                                        boxed_callback(CustomSwarmEvent::OutgoingConnectionError {
                                            peer_id: peer_id.map(|f| f.to_string()),
                                            error: error.to_string(),
                                            connection_id
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::NewListenAddr { address, listener_id } => {
                                        boxed_callback(CustomSwarmEvent::NewListenAddr {
                                            address: address.to_string(),
                                            listener_id: listener_id.to_string()
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::ExpiredListenAddr { address, listener_id } => {
                                        boxed_callback(CustomSwarmEvent::ExpiredListenAddr {
                                            address: address.to_string(),
                                            listener_id: listener_id.to_string()
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::ListenerClosed {
                                        addresses, reason, listener_id
                                    } => {
                                        boxed_callback(CustomSwarmEvent::ListenerClosed {
                                            addresses: addresses.iter().map(|f| f.to_string()).collect(),
                                            reason: reason.err().map(|f| f.to_string()),
                                            listener_id: listener_id.to_string()
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::ListenerError { error, listener_id } => {
                                        boxed_callback(CustomSwarmEvent::ListenerError {
                                            error: error.to_string(),
                                            listener_id: listener_id.to_string()
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::Dialing { peer_id, connection_id } => {
                                        boxed_callback(CustomSwarmEvent::Dialing {
                                            peer_id: peer_id.map(|f| f.to_string()),
                                            connection_id
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::NewExternalAddrCandidate { address } => {
                                        boxed_callback(CustomSwarmEvent::NewExternalAddrCandidate {
                                            address: address.to_string(),
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::ExternalAddrConfirmed { address } => {
                                        boxed_callback(CustomSwarmEvent::ExternalAddrConfirmed {
                                            address: address.to_string(),
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::ExternalAddrExpired { address } => {
                                        boxed_callback(CustomSwarmEvent::ExternalAddrExpired {
                                            address: address.to_string(),
                                        })
                                        .await;
                                        ()
                                    }
                                    SwarmEvent::NewExternalAddrOfPeer { peer_id, address } => {
                                        boxed_callback(CustomSwarmEvent::NewExternalAddrOfPeer {
                                            peer_id: peer_id.to_string(),
                                            address: address.to_string(),
                                        })
                                        .await;
                                        ()
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
