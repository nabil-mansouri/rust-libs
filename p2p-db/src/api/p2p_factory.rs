use super::p2p_common::CustomBehaviour;
pub use crate::api::wrapper::Wrapper;
pub use libp2p::gossipsub::{
    self, Behaviour as GossipBehaviour, ConfigBuilder, MessageAuthenticity, MessageId,
    ValidationMode,
};
use libp2p::identify::Behaviour as IdentifyBehaviour;
pub use libp2p::identity::{Keypair, PeerId, PublicKey};
use libp2p::rendezvous::client::Behaviour as rdv_clientBehaviour;
use libp2p::rendezvous::server::{self as rdvserver, Behaviour as RdvServerBehaviour};
pub use libp2p::request_response::{self as request_response, ProtocolSupport, ResponseChannel};
use libp2p::{
    allow_block_list, autonat, connection_limits, dcutr, identify, memory_connection_limits, noise,
    relay, tcp, upnp, yamux, StreamProtocol,
};
use std::sync::Arc;
use std::time::Duration;

pub async fn create_libp2p_instance(
    keypair: Keypair,
    tcp_nodelay: bool,
    tcp_reuse_port: bool,
    pubsub_heartbeat_delay: u64,
    pubsub_heartbeat_interval: u64,
    nat_boot_delay: u64,
    nat_retry_interval: u64,
    nat_only_global_ips: bool,
    nat_use_connected: bool,
    connection_max_inout: Option<u32>,
    connection_max_inbound: Option<u32>,
    connection_max_outgoing: Option<u32>,
    connection_max_pending_inbound: Option<u32>,
    connection_max_pending_outgoing: Option<u32>,
    memory_max_percentage: f64,
) -> Result<Arc<Wrapper>, String> {
    let gossip_factory = |key: &Keypair| -> Result<GossipBehaviour, String> {
        let gossip_config_unsafe = ConfigBuilder::default()
            // send unsubscribe to self
            .allow_self_origin(true)
            .validate_messages()
            .heartbeat_initial_delay(Duration::from_millis(pubsub_heartbeat_delay))
            .heartbeat_interval(Duration::from_millis(pubsub_heartbeat_interval))
            .validation_mode(ValidationMode::Strict)
            .build()
            .map_err(|e| e.to_string());
        match gossip_config_unsafe {
            Ok(gossip_config) => {
                return GossipBehaviour::new(
                    MessageAuthenticity::Signed(key.clone()),
                    gossip_config,
                )
                .map_err(|e| e.to_string())
            }
            Err(err) => return Err(err),
        }
    };
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default()
                .nodelay(tcp_nodelay)
                .port_reuse(tcp_reuse_port),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| e.to_string())?
        //.with_quic()
        .with_dns()
        .map_err(|e| e.to_string())?
        .with_websocket(noise::Config::new, yamux::Config::default)
        .await
        .map_err(|e| e.to_string())?
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .map_err(|e| e.to_string())?
        .with_behaviour(|key, relay_client: relay::client::Behaviour| {
            let gossip_unsafe = gossip_factory(key);
            match gossip_unsafe {
                Ok(gossip) => {
                    return Ok(CustomBehaviour {
                        blacklist: allow_block_list::Behaviour::default(),
                        //whitelist: allow_block_list::Behaviour::default(),
                        connection_limits: connection_limits::Behaviour::new(
                            connection_limits::ConnectionLimits::default()
                                .with_max_established_incoming(connection_max_inbound)
                                .with_max_pending_incoming(connection_max_pending_inbound)
                                .with_max_established_outgoing(connection_max_outgoing)
                                .with_max_pending_outgoing(connection_max_pending_outgoing)
                                .with_max_established(connection_max_inout),
                        ),
                        memory_limits: memory_connection_limits::Behaviour::with_max_percentage(
                            memory_max_percentage,
                        ),
                        upnp: upnp::tokio::Behaviour::default(),
                        auto_nat: autonat::Behaviour::new(
                            key.public().to_peer_id(),
                            autonat::Config {
                                boot_delay: Duration::from_millis(nat_boot_delay),
                                retry_interval: Duration::from_millis(nat_retry_interval),
                                only_global_ips: nat_only_global_ips,
                                use_connected: nat_use_connected,
                                ..autonat::Config::default()
                            },
                        ),
                        identify: IdentifyBehaviour::new(identify::Config::new(
                            "rendezvous/1.0.0".to_string(),
                            key.public(),
                        )),
                        relay_server: relay::Behaviour::new(
                            key.public().to_peer_id(),
                            relay::Config::default(),
                        ),
                        relay_client,
                        dcutr: dcutr::Behaviour::new(key.public().to_peer_id()),
                        rdv_server: RdvServerBehaviour::new(rdvserver::Config::default()),
                        rdv_client: rdv_clientBehaviour::new(key.clone()),
                        pubsub: gossip,
                        request_response: request_response::cbor::Behaviour::new(
                            [(
                                StreamProtocol::new("/transfer/1.0.0"),
                                ProtocolSupport::Full,
                            )],
                            request_response::Config::default(),
                        ),
                    });
                }
                Err(err) => return Err(err.into()),
            };
        })
        .map_err(|e| e.to_string())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();
    Ok(Arc::new(Wrapper::new(swarm)))
}
