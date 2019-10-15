// Copyright 2019, The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::{
    envelope::{DhtEnvelope, DhtHeader, DhtMessageFlags, DhtMessageType, NodeDestination},
    inbound::DhtInboundMessage,
};
use rand::rngs::OsRng;
use std::sync::Arc;
use tari_comms::{
    connection::NetAddress,
    message::{InboundMessage, MessageEnvelopeHeader, MessageFlags},
    peer_manager::{NodeIdentity, Peer, PeerFeatures, PeerFlags, PeerManager},
    types::CommsDatabase,
    utils::signature,
};
use tari_storage::lmdb_store::LMDBBuilder;
use tari_test_utils::{paths::create_random_database_path, random};
use tari_utilities::message_format::MessageFormat;

pub fn make_node_identity() -> Arc<NodeIdentity> {
    Arc::new(
        NodeIdentity::random(
            &mut OsRng::new().unwrap(),
            "127.0.0.1:9000".parse().unwrap(),
            PeerFeatures::communication_node_default(),
        )
        .unwrap(),
    )
}

pub fn make_client_identity() -> Arc<NodeIdentity> {
    Arc::new(
        NodeIdentity::random(
            &mut OsRng::new().unwrap(),
            "127.0.0.1:9000".parse().unwrap(),
            PeerFeatures::communication_client_default(),
        )
        .unwrap(),
    )
}

pub fn make_comms_inbound_message(
    node_identity: &NodeIdentity,
    message: Vec<u8>,
    flags: MessageFlags,
) -> InboundMessage
{
    InboundMessage::new(
        Peer::new(
            node_identity.identity.public_key.clone(),
            node_identity.identity.node_id.clone(),
            Vec::<NetAddress>::new().into(),
            PeerFlags::empty(),
            PeerFeatures::communication_node_default(),
        ),
        MessageEnvelopeHeader {
            version: 0,
            message_public_key: node_identity.identity.public_key.clone(),
            message_signature: Vec::new(),
            flags,
        },
        0,
        message,
    )
}

pub fn make_dht_header(node_identity: &NodeIdentity, message: &Vec<u8>, flags: DhtMessageFlags) -> DhtHeader {
    DhtHeader {
        version: 0,
        destination: NodeDestination::Undisclosed,
        origin_public_key: node_identity.public_key().clone(),
        origin_signature: signature::sign(&mut OsRng::new().unwrap(), node_identity.secret_key.clone(), message)
            .unwrap()
            .to_binary()
            .unwrap(),
        message_type: DhtMessageType::None,
        flags,
    }
}

pub fn make_dht_inbound_message(
    node_identity: &NodeIdentity,
    body: Vec<u8>,
    flags: DhtMessageFlags,
) -> DhtInboundMessage
{
    DhtInboundMessage::new(
        make_dht_header(node_identity, &body, flags),
        Peer::new(
            node_identity.identity.public_key.clone(),
            node_identity.identity.node_id.clone(),
            Vec::<NetAddress>::new().into(),
            PeerFlags::empty(),
            PeerFeatures::communication_node_default(),
        ),
        MessageEnvelopeHeader {
            version: 0,
            message_public_key: node_identity.identity.public_key.clone(),
            message_signature: Vec::new(),
            flags: MessageFlags::empty(),
        },
        body,
    )
}

pub fn make_dht_envelope(node_identity: &NodeIdentity, message: Vec<u8>, flags: DhtMessageFlags) -> DhtEnvelope {
    DhtEnvelope::new(make_dht_header(node_identity, &message, flags), message)
}

pub fn make_peer_manager() -> Arc<PeerManager> {
    let database_name = random::string(8);
    let path = create_random_database_path();
    let datastore = LMDBBuilder::new()
        .set_path(path.to_str().unwrap())
        .set_environment_size(10)
        .set_max_number_of_databases(1)
        .add_database(&database_name, lmdb_zero::db::CREATE)
        .build()
        .unwrap();

    let peer_database = datastore.get_handle(&database_name).unwrap();

    PeerManager::new(CommsDatabase::new(Arc::new(peer_database)))
        .map(Arc::new)
        .unwrap()
}