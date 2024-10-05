use flutter_rust_bridge::frb;
pub use libp2p::identity::{Keypair, PeerId, PublicKey};
//
// ENUM
//
#[frb(external)]
#[frb(non_opaque)]
pub enum KeyType {
    Ed25519,
    Rsa,
    Secp256k1,
    Ecdsa,
}
//
// OVERRIDE
//
#[frb(external)]
#[frb(opaque)]
impl Keypair {
    #[frb(sync)]
    pub fn public(&self) -> PublicKey {}
}

#[frb(external)]
#[frb(opaque)]
impl PublicKey {
    #[frb(sync)]
    pub fn encode_protobuf(&self) -> Vec<u8> {}
    #[frb(sync)]
    pub fn to_peer_id(&self) -> PeerId {}
}

#[frb(external)]
#[frb(opaque)]
impl PeerId {
    #[frb(sync)]
    pub fn to_bytes(self) -> Vec<u8> {}
    #[frb(sync)]
    pub fn to_base58(self) -> String {}
}
//
// FUNCTIONS
//
pub fn create_keypair_from_bytes(key_type: KeyType, mut bytes: Vec<u8>) -> Result<Keypair, String> {
    match key_type {
        KeyType::Ed25519 => Keypair::ed25519_from_bytes(bytes).map_err(|e| e.to_string()),
        KeyType::Rsa => {
            let slice: &mut [u8] = bytes.as_mut();
            Keypair::rsa_from_pkcs8(slice).map_err(|e| e.to_string())
        }
        KeyType::Secp256k1 => {
            let slice: &mut [u8] = bytes.as_mut();
            Keypair::secp256k1_from_der(slice).map_err(|e| e.to_string())
        }
        KeyType::Ecdsa => todo!(),
    }
}

pub fn create_keypair_using_random(key_type: KeyType) -> Keypair {
    match key_type {
        KeyType::Ed25519 => Keypair::generate_ed25519(),
        KeyType::Rsa => todo!(),
        KeyType::Secp256k1 => Keypair::generate_secp256k1(),
        KeyType::Ecdsa => Keypair::generate_ecdsa(),
    }
}

#[frb(sync)]
pub fn keypair_to_protobuf(keypair: &Keypair) -> Result<Vec<u8>, String> {
    keypair
        .to_protobuf_encoding()
        .map_err(|err| err.to_string())
}

#[frb(sync)]
pub fn keypair_from_protobuf(proto: Vec<u8>) -> Result<Keypair, String> {
    Keypair::from_protobuf_encoding(proto.as_slice()).map_err(|err| err.to_string())
}

pub fn keypair_sign(keypair: &Keypair, msg: Vec<u8>) -> Result<Vec<u8>, String> {
    keypair.sign(msg.as_slice()).map_err(|err| err.to_string())
}

pub fn keypair_verify(key: &PublicKey, msg: Vec<u8>, sig: Vec<u8>) -> bool {
    key.verify(msg.as_slice(), sig.as_slice())
}
