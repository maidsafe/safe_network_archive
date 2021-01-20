// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod client;
mod client_msg_analysis;
mod transfers;

use self::{
    client::ClientGateway,
    client_msg_analysis::ClientMsgAnalysis,
    transfers::{replicas::Replicas, Transfers},
};
use crate::{
    capacity::RateLimit,
    node::node_ops::{KeySectionDuty, NodeOperation},
    node::NodeInfo,
    ElderState, ReplicaInfo, Result,
};
use futures::lock::Mutex;
use log::{info, trace};
use sn_data_types::PublicKey;
use sn_routing::Prefix;
use std::path::PathBuf;
use std::sync::Arc;

/// A Key Section interfaces with clients,
/// who are essentially a public key,
/// (hence the name Key Section), used by
/// a specific socket address.
/// The Gateway deals with onboarding (handshakes etc)
/// and routing messages back and forth to clients.
/// Payments deals with the payment for data writes,
/// while transfers deals with sending money between keys.
pub struct KeySection {
    gateway: ClientGateway,
    transfers: Transfers,
    msg_analysis: ClientMsgAnalysis,
    elder_state: ElderState,
}

impl KeySection {
    pub async fn new(
        info: &NodeInfo,
        rate_limit: RateLimit,
        elder_state: ElderState,
    ) -> Result<Self> {
        let gateway = ClientGateway::new(info, elder_state.clone()).await?;
        let replicas = Self::transfer_replicas(info.root_dir.clone(), elder_state.clone())?;
        let transfers = Transfers::new(elder_state.clone(), replicas, rate_limit);
        let msg_analysis = ClientMsgAnalysis::new(elder_state.clone());

        Ok(Self {
            gateway,
            transfers,
            msg_analysis,
            elder_state,
        })
    }

    ///
    pub async fn increase_full_node_count(&mut self, node_id: PublicKey) -> Result<()> {
        self.transfers.increase_full_node_count(node_id)
    }

    /// Initiates as first node in a network.
    pub async fn init_genesis_node(&mut self) -> Result<NodeOperation> {
        self.transfers.genesis().await
    }

    /// Issues queries to Elders of the section
    /// as to catch up with shares state and
    /// start working properly in the group.
    pub async fn catchup_with_section(&mut self) -> Result<NodeOperation> {
        // currently only at2 replicas need to catch up
        self.transfers.catchup_with_replicas().await
    }

    pub async fn set_node_join_flag(&mut self, joins_allowed: bool) -> Result<NodeOperation> {
        match self.elder_state.set_joins_allowed(joins_allowed).await {
            Ok(()) => {
                info!("Successfully set joins_allowed to true");
                Ok(NodeOperation::NoOp)
            }
            Err(e) => Err(e),
        }
    }

    // Update our replica with the latest keys
    pub fn elders_changed(&mut self, state: ElderState, rate_limit: RateLimit) {
        // TODO: Query sn_routing for info for [new_section_key]
        // specifically (regardless of how far back that was) - i.e. not the current info!
        let secret_key_share = state.secret_key_share();
        let id = secret_key_share.public_key_share();
        let key_index = state.key_index();
        let peer_replicas = state.public_key_set().clone();
        let signing = sn_transfers::ReplicaSigning::new(
            secret_key_share.clone(),
            key_index,
            peer_replicas.clone(),
        );
        let info = ReplicaInfo {
            id,
            key_index,
            peer_replicas,
            section_proof_chain: state.section_proof_chain().clone(),
            signing: Arc::new(Mutex::new(signing)),
            initiating: false,
        };
        self.transfers.update_replica_info(info, rate_limit);
    }

    /// When section splits, the Replicas in either resulting section
    /// also split the responsibility of their data.
    pub async fn split_section(&mut self, prefix: Prefix) -> Result<()> {
        self.transfers.split_section(prefix).await
    }

    pub async fn process_key_section_duty(&self, duty: KeySectionDuty) -> Result<NodeOperation> {
        trace!("Processing as Elder KeySection");
        use KeySectionDuty::*;
        match duty {
            EvaluateClientMsg(msg) => self.msg_analysis.evaluate(&msg).await,
            RunAsGateway(duty) => self.gateway.process_as_gateway(duty).await,
            RunAsTransfers(duty) => self.transfers.process_transfer_duty(&duty).await,
            NoOp => Ok(NodeOperation::NoOp),
        }
    }

    fn transfer_replicas(root_dir: PathBuf, elder_state: ElderState) -> Result<Replicas> {
        let secret_key_share = elder_state.secret_key_share().clone();
        let key_index = elder_state.key_index();
        let peer_replicas = elder_state.public_key_set().clone();
        let id = secret_key_share.public_key_share();
        let signing =
            sn_transfers::ReplicaSigning::new(secret_key_share, key_index, peer_replicas.clone());
        let info = ReplicaInfo {
            id,
            key_index,
            peer_replicas,
            section_proof_chain: elder_state.section_proof_chain().clone(),
            signing: Arc::new(Mutex::new(signing)),
            initiating: true,
        };
        let replica_manager = Replicas::new(root_dir, info)?;
        Ok(replica_manager)
    }
}
