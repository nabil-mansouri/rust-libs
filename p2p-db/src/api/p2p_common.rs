use flutter_rust_bridge::{frb, RustAutoOpaqueNom};
use libp2p::autonat::NatStatus;
pub use libp2p::core::transport::ListenerId;
pub use libp2p::gossipsub::{Behaviour as GossipBehaviour, MessageId};
use libp2p::identify::Behaviour as IdentifyBehaviour;
pub use libp2p::identity::{Keypair, PeerId, PublicKey};
use libp2p::rendezvous::client::Behaviour as RdvClientBehaviour;
use libp2p::rendezvous::server::Behaviour as RdvServerBehaviour;
pub use libp2p::request_response::{self as request_response, ResponseChannel};
pub use libp2p::swarm::ConnectionId;
pub use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{
    allow_block_list, autonat, connection_limits, dcutr, memory_connection_limits, relay, upnp,
};
use strum_macros::{AsRefStr, IntoStaticStr};

//
// ERRORS
//
#[frb(external)]
pub enum GenericError {
    InstanceNotFound,
    BadAddress,
    Other(String),
    Bytes(Vec<u8>),
}
//
// OVERRIDES
//
#[frb(external)]
#[frb(opaque)]
impl ListenerId {
    #[frb(sync)]
    pub fn to_string(&self) -> String {}
}
#[frb(external)]
#[frb(opaque)]
impl ConnectionId {
    #[frb(sync)]
    pub fn to_string(&self) -> String {}
}
#[frb(external)]
#[frb(opaque)]
impl MessageId {
    #[frb(sync)]
    pub fn to_string(&self) -> String {}
}
#[frb(external)]
pub struct CustomPeerRecord {
    pub peer_id: String,
    pub addresses: Vec<String>
}
//
// BEHAVIOURS
//
#[derive(NetworkBehaviour)]
#[frb(ignore)]
pub struct CustomBehaviour {
    pub blacklist: allow_block_list::Behaviour<allow_block_list::BlockedPeers>,
    //pub whitelist: allow_block_list::Behaviour<allow_block_list::AllowedPeers>,
    pub memory_limits: memory_connection_limits::Behaviour,
    pub connection_limits: connection_limits::Behaviour,
    pub upnp: upnp::tokio::Behaviour,
    pub auto_nat: autonat::Behaviour,
    pub identify: IdentifyBehaviour,
    pub rdv_server: RdvServerBehaviour,
    pub rdv_client: RdvClientBehaviour,
    pub pubsub: GossipBehaviour,
    pub dcutr: dcutr::Behaviour,
    pub relay_server: relay::Behaviour,
    pub relay_client: relay::client::Behaviour,
    pub request_response: request_response::cbor::Behaviour<Vec<u8>, Vec<u8>>,
}

//
// NAT STATUS
//
#[frb(external)]
#[frb(non_opaque)]
#[derive(Debug, PartialEq)]
pub enum CustomNatStatus {
    Public(String),
    Private,
    Unknown,
}
impl From<NatStatus> for CustomNatStatus {
    fn from(status: NatStatus) -> CustomNatStatus {
        match status {
            NatStatus::Public(add) => CustomNatStatus::Public(add.to_string()),
            NatStatus::Private => CustomNatStatus::Private,
            NatStatus::Unknown => CustomNatStatus::Unknown,
        }
    }
}
//
// EVENT
//
#[frb(external)]
#[frb(non_opaque)]
#[derive(IntoStaticStr, AsRefStr)]
pub enum CustomSwarmEvent {
    /// The multiaddress is reachable externally.
    UpnpNewExternalAddr(String),
    /// The renewal of the multiaddress on the gateway failed.
    UpnpExpiredExternalAddr(String),
    /// The IGD gateway was not found.
    UpnpGatewayNotFound,
    /// The Gateway is not exposed directly to the public network.
    UpnpNonRoutableGateway,
    NATStatusChanged {
        /// Former status.
        old_value: CustomNatStatus,
        /// New status.
        new_value: CustomNatStatus,
    },
    /// An incoming message (request or response).
    RequestMessage {
        request_id: String,
        /// The peer who sent the message.
        peer: String,
        /// The incoming message.
        message: Vec<u8>,
        channel: RustAutoOpaqueNom<ResponseChannel<Vec<u8>>>,
    },
    /// An incoming message (request or response).
    ResponseMessage {
        request_id: String,
        /// The peer who sent the message.
        peer: String,
        /// The incoming message.
        message: Vec<u8>,
    },
    /// An outbound request failed.
    RequestOutboundFailure {
        /// The peer to whom the request was sent.
        peer: String,
        /// The (local) ID of the failed request.
        request_id: String,
        /// The error that occurred.
        error: String,
    },
    /// An inbound request failed.
    RequestInboundFailure {
        /// The peer from whom the request was received.
        peer: String,
        /// The ID of the failed inbound request.
        request_id: String,
        /// The error that occurred.
        error: String,
    },
    /// A response to an inbound request has been sent.
    ///
    /// When this event is received, the response has been flushed on
    /// the underlying transport connection.
    ResponseSent {
        /// The peer to whom the response was sent.
        peer: String,
        /// The ID of the inbound request whose response was sent.
        request_id: String,
    },
    RdvClientDiscovered {
        rendezvous_node: String,
        registrations: Vec<CustomPeerRecord>,
    },
    RdvClientDiscoveryFail {
        rendezvous_node: String,
    },
    RdvClientRegistered {
        rendezvous_node: String,
    },
    RdvClientRegisteredFailed {
        rendezvous_node: String,
    },
    RdvClientDiscoveryExpired {
        peer_id: String,
    },
    RdvServerPeerRegistered {
        peerid: String,
        addresses: Vec<String>,
    },
    RdvServerPeerUnRegistered {
        peerid: String,
    },
    RdvServerPeerExpired {
        peerid: String,
    },
    IdentifyReceived {
        /// The peer that has been identified.
        peer_id: String,
        /// The information provided by the peer.
        /// The public key of the local peer.
        public_key: Vec<u8>,
        /// Application-specific version of the protocol family used by the peer,
        /// e.g. `ipfs/1.0.0` or `polkadot/1.0.0`.
        protocol_version: String,
        /// Name and version of the peer, similar to the `User-Agent` header in
        /// the HTTP protocol.
        agent_version: String,
        /// The addresses that the peer is listening on.
        listen_addrs: Vec<String>,
        /// The list of protocols supported by the peer, e.g. `/ipfs/ping/1.0.0`.
        protocols: Vec<String>,
        /// Address observed by or for the remote.
        observed_addr: String,
    },
    GossipMessage {
        /// The peer that forwarded us this message.
        propagation_source: String,
        /// The [`MessageId`] of the message. This should be referenced by the application when
        /// validating a message (if required).
        message_id: Vec<u8>,
        /// The decompressed message itself.
        message: Vec<u8>,
        /// Id of the peer that published this message.
        source: Option<String>,
        /// The topic this message belongs to
        topic_hash: String,
    },
    /// A remote subscribed to a topic.
    GossipSubscribed {
        /// Remote that has subscribed.
        peer_id: String,
        /// The topic it has subscribed to.
        topic: String,
    },
    /// A remote unsubscribed from a topic.
    GossipUnsubscribed {
        /// Remote that has unsubscribed.
        peer_id: String,
        /// The topic it has subscribed from.
        topic: String,
    },
    /// A peer that does not support gossipsub has connected.
    GossipsubNotSupported { peer_id: String },
    /// A connection to the given peer has been opened.
    ConnectionEstablished {
        /// Identity of the peer that we have connected to.
        peer_id: String,
        /// Identifier of the connection.
        connection_id: ConnectionId,
        /// Endpoint of the connection that has been opened.
        endpoint: String,
        /// Number of established connections to this peer, including the one that has just been
        /// opened.
        num_established: u32,
        /// [`Some`] when the new connection is an outgoing connection.
        /// Addresses are dialed concurrently. Contains the addresses and errors
        /// of dial attempts that failed before the one successful dial.
        //concurrent_dial_errors: Option<Vec<(Multiaddr, TransportError<io::Error>)>>,
        /// How long it took to establish this connection
        established_in: u128,
    },
    /// A connection with the given peer has been closed,
    /// possibly as a result of an error.
    ConnectionClosed {
        /// Identity of the peer that we have connected to.
        peer_id: String,
        /// Identifier of the connection.
        connection_id: ConnectionId,
        /// Endpoint of the connection that has been closed.
        endpoint: String,
        /// Number of other remaining connections to this same peer.
        num_established: u32,
        /// Reason for the disconnection, if it was not a successful
        /// active close.
        cause: Option<String>,
    },
    /// A new connection arrived on a listener and is in the process of protocol negotiation.
    ///
    /// A corresponding [`ConnectionEstablished`](SwarmEvent::ConnectionEstablished) or
    /// [`IncomingConnectionError`](SwarmEvent::IncomingConnectionError) event will later be
    /// generated for this connection.
    IncomingConnection {
        /// Identifier of the connection.
        connection_id: ConnectionId,
        /// Local connection address.
        /// This address has been earlier reported with a [`NewListenAddr`](SwarmEvent::NewListenAddr)
        /// event.
        local_addr: String,
        /// Address used to send back data to the remote.
        send_back_addr: String,
    },
    /// An error happened on an inbound connection during its initial handshake.
    ///
    /// This can include, for example, an error during the handshake of the encryption layer, or
    /// the connection unexpectedly closed.
    IncomingConnectionError {
        /// Identifier of the connection.
        connection_id: ConnectionId,
        /// Local connection address.
        /// This address has been earlier reported with a [`NewListenAddr`](SwarmEvent::NewListenAddr)
        /// event.
        local_addr: String,
        /// Address used to send back data to the remote.
        send_back_addr: String,
        /// The error that happened.
        error: String,
    },
    /// An error happened on an outbound connection.
    OutgoingConnectionError {
        /// Identifier of the connection.
        connection_id: ConnectionId,
        /// If known, [`PeerId`] of the peer we tried to reach.
        peer_id: Option<String>,
        /// Error that has been encountered.
        error: String,
    },
    /// One of our listeners has reported a new local listening address.
    NewListenAddr {
        /// The listener that is listening on the new address.
        listener_id: String,
        /// The new address that is being listened on.
        address: String,
    },
    /// One of our listeners has reported the expiration of a listening address.
    ExpiredListenAddr {
        /// The listener that is no longer listening on the address.
        listener_id: String,
        /// The expired address.
        address: String,
    },
    /// One of the listeners gracefully closed.
    ListenerClosed {
        /// The listener that closed.
        listener_id: String,
        /// The addresses that the listener was listening on. These addresses are now considered
        /// expired, similar to if a [`ExpiredListenAddr`](SwarmEvent::ExpiredListenAddr) event
        /// has been generated for each of them.
        addresses: Vec<String>,
        /// Reason for the closure. Contains `Ok(())` if the stream produced `None`, or `Err`
        /// if the stream produced an error.
        reason: Option<String>,
    },
    /// One of the listeners reported a non-fatal error.
    ListenerError {
        /// The listener that errored.
        listener_id: String,
        /// The listener error.
        error: String,
    },
    /// A new dialing attempt has been initiated by the [`NetworkBehaviour`]
    /// implementation.
    ///
    /// A [`ConnectionEstablished`](SwarmEvent::ConnectionEstablished) event is
    /// reported if the dialing attempt succeeds, otherwise a
    /// [`OutgoingConnectionError`](SwarmEvent::OutgoingConnectionError) event
    /// is reported.
    Dialing {
        /// Identity of the peer that we are connecting to.
        peer_id: Option<String>,
        // Identifier of the connection.
        connection_id: ConnectionId,
    },
    /// We have discovered a new candidate for an external address for us.
    NewExternalAddrCandidate { address: String },
    /// An external address of the local node was confirmed.
    ExternalAddrConfirmed { address: String },
    /// An external address of the local node expired, i.e. is no-longer confirmed.
    ExternalAddrExpired { address: String },
    /// We have discovered a new address of a peer.
    NewExternalAddrOfPeer { peer_id: String, address: String },
}
