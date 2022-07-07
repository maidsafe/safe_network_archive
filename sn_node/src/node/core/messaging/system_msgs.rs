// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    comm::Comm,
    dbs::Error as DbError,
    node::{
        api::cmds::Cmd,
        core::{DkgSessionInfo, Node, Proposal as CoreProposal},
        messages::WireMsgUtils,
        Error, Event, MembershipEvent, Result, MIN_LEVEL_WHEN_FULL,
    },
};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use sn_interface::{
    messaging::{
        data::StorageLevel,
        signature_aggregator::Error as AggregatorError,
        system::{
            // SectionAuth is gonna cause issue
            JoinResponse,
            NodeCmd,
            NodeEvent,
            NodeMsgAuthorityUtils,
            NodeQuery,
            Proposal as ProposalMsg,
            SectionAuth as SectionAuthAgreement,
            SystemMsg,
        },
        AuthKind, AuthorityProof, DstLocation, MsgId, NodeMsgAuthority, SectionAuth, WireMsg,
    },
    network_knowledge::{NetworkKnowledge, NodeState},
    types::{log_markers::LogMarker, Keypair, Peer, PublicKey},
};
use xor_name::XorName;

impl Node {
    /// Send a direct (`SystemMsg`) message to a node in the specified section
    pub(crate) async fn send_direct_msg(
        &self,
        recipient: Peer,
        node_msg: SystemMsg,
        section_pk: BlsPublicKey,
    ) -> Result<Cmd> {
        let section_name = recipient.name();
        self.send_direct_msg_to_nodes(vec![recipient], node_msg, section_name, section_pk)
            .await
    }

    /// Send a direct (`SystemMsg`) message to a set of nodes in the specified section
    pub(crate) async fn send_direct_msg_to_nodes(
        &self,
        recipients: Vec<Peer>,
        node_msg: SystemMsg,
        section_name: XorName,
        section_pk: BlsPublicKey,
    ) -> Result<Cmd> {
        trace!("{}", LogMarker::SendDirectToNodes);
        let our_node = self.info();
        let our_section_key = self.network_knowledge.section_key();

        let wire_msg = WireMsg::single_src(
            &our_node,
            DstLocation::Section {
                name: section_name,
                section_pk,
            },
            node_msg,
            our_section_key,
        )?;

        Ok(Cmd::SendMsg {
            recipients,
            wire_msg,
        })
    }

    /// Send a `Relocate` message to the specified node
    pub(crate) async fn send_relocate(
        &self,
        recipient: Peer,
        node_state: SectionAuthAgreement<NodeState>,
    ) -> Result<Cmd> {
        let node_msg = SystemMsg::Relocate(node_state.into_authed_msg());
        let section_pk = self.network_knowledge.section_key();
        self.send_direct_msg(recipient, node_msg, section_pk).await
    }

    /// Send a direct (`SystemMsg`) message to all Elders in our section
    pub(crate) async fn send_msg_to_our_elders(&self, node_msg: SystemMsg) -> Result<Cmd> {
        let sap = self.network_knowledge.authority_provider();
        let dst_section_pk = sap.section_key();
        let section_name = sap.prefix().name();
        let elders = sap.elders_vec();
        self.send_direct_msg_to_nodes(elders, node_msg, section_name, dst_section_pk)
            .await
    }

    // Send the message to all `recipients`. If one of the recipients is us, don't send it over the
    // network but handle it directly (should only be used when accumulation is necessary)
    pub(crate) async fn send_messages_to_all_nodes_or_directly_handle_for_accumulation(
        &self,
        recipients: Vec<Peer>,
        mut wire_msg: WireMsg,
    ) -> Result<Vec<Cmd>> {
        let mut cmds = vec![];
        let mut others = Vec::new();
        let mut handle = false;

        trace!("Send {:?} to {:?}", wire_msg, recipients);

        let our_name = self.info().name();
        for recipient in recipients.into_iter() {
            if recipient.name() == our_name {
                match wire_msg.auth_kind() {
                    AuthKind::NodeBlsShare(_) => {
                        // do nothing, continue we should be accumulating this
                        handle = true;
                    }
                    _ => return Err(Error::SendOrHandlingNormalMsg),
                }
            } else {
                others.push(recipient);
            }
        }

        if !others.is_empty() {
            let dst_section_pk = self.section_key_by_name(&others[0].name()).await;
            wire_msg.set_dst_section_pk(dst_section_pk);

            trace!("{}", LogMarker::SendOrHandle);
            cmds.push(Cmd::SendMsg {
                recipients: others,
                wire_msg: wire_msg.clone(),
            });
        }

        if handle {
            wire_msg.set_dst_section_pk(self.network_knowledge.section_key());
            wire_msg.set_dst_xorname(our_name);

            cmds.push(Cmd::HandleMsg {
                sender: Peer::new(our_name, self.addr),
                wire_msg,
                original_bytes: None,
            });
        }

        Ok(cmds)
    }

    // Handler for all system messages
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn handle_system_msg(
        &mut self,
        sender: Peer,
        msg_id: MsgId,
        mut msg_authority: NodeMsgAuthority,
        msg: SystemMsg,
        payload: Bytes,
        known_keys: Vec<BlsPublicKey>,
        comm: &Comm,
    ) -> Result<Vec<Cmd>> {
        trace!("{:?}", LogMarker::SystemMsgToBeHandled);

        // We assume to be aggregated if it contains a BLS Share sig as authority.
        match self
            .aggregate_msg_and_stop(&mut msg_authority, payload)
            .await
        {
            Ok(false) => {
                self.handle_valid_msg(msg_id, msg_authority, msg, sender, known_keys, comm)
                    .await
            }
            Err(Error::InvalidSignatureShare) => {
                warn!(
                    "Invalid signature on received system message, dropping the message: {:?}",
                    msg_id
                );
                Ok(vec![])
            }
            Ok(true) => Ok(vec![]),
            Err(err) => {
                trace!("handle_system_msg got error {:?}", err);
                Ok(vec![])
            }
        }
    }

    // Handler for data messages which have successfully
    // passed all signature checks and msg verifications
    pub(crate) async fn handle_valid_msg(
        &mut self,
        msg_id: MsgId,
        msg_authority: NodeMsgAuthority,
        node_msg: SystemMsg,
        sender: Peer,
        known_keys: Vec<BlsPublicKey>,
        comm: &Comm,
    ) -> Result<Vec<Cmd>> {
        let src_name = msg_authority.name();
        match node_msg {
            SystemMsg::AntiEntropyUpdate {
                section_auth,
                section_signed,
                proof_chain,
                members,
            } => {
                trace!("Handling msg: AE-Update from {}: {:?}", sender, msg_id,);
                self.handle_anti_entropy_update_msg(
                    section_auth.into_state(),
                    section_signed,
                    proof_chain,
                    members,
                )
                .await
            }
            SystemMsg::Relocate(node_state) => {
                trace!("Handling msg: Relocate from {}: {:?}", sender, msg_id);
                Ok(self
                    .handle_relocate(node_state)
                    .await?
                    .into_iter()
                    .collect())
            }
            SystemMsg::StartConnectivityTest(name) => {
                trace!(
                    "Handling msg: StartConnectivityTest from {}: {:?}",
                    sender,
                    msg_id
                );
                if self.is_not_elder() {
                    return Ok(vec![]);
                }

                Ok(vec![Cmd::TestConnectivity(name)])
            }
            SystemMsg::JoinAsRelocatedResponse(join_response) => {
                trace!("Handling msg: JoinAsRelocatedResponse from {}", sender);
                if let Some(ref mut joining_as_relocated) = self.relocate_state {
                    if let Some(cmd) = joining_as_relocated
                        .handle_join_response(*join_response, sender.addr())
                        .await?
                    {
                        return Ok(vec![cmd]);
                    }
                } else {
                    error!(
                        "No relocation in progress upon receiving {:?}",
                        join_response
                    );
                }

                Ok(vec![])
            }
            SystemMsg::NodeMsgError {
                error,
                correlation_id,
            } => {
                trace!(
                    "From {:?}({:?}), received error {:?} correlated to {:?}",
                    msg_authority.src_location(),
                    msg_id,
                    error,
                    correlation_id
                );
                Ok(vec![])
            }
            SystemMsg::AntiEntropyRetry {
                section_auth,
                section_signed,
                proof_chain,
                bounced_msg,
            } => {
                trace!("Handling msg: AE-Retry from {}: {:?}", sender, msg_id,);
                self.handle_anti_entropy_retry_msg(
                    section_auth.into_state(),
                    section_signed,
                    proof_chain,
                    bounced_msg,
                    sender,
                )
                .await
            }
            SystemMsg::AntiEntropyRedirect {
                section_auth,
                section_signed,
                section_chain,
                bounced_msg,
            } => {
                trace!("Handling msg: AE-Redirect from {}: {:?}", sender, msg_id);
                self.handle_anti_entropy_redirect_msg(
                    section_auth.into_state(),
                    section_signed,
                    section_chain,
                    bounced_msg,
                    sender,
                )
                .await
            }
            SystemMsg::AntiEntropyProbe => {
                trace!("Received Probe message from {}: {:?}", sender, msg_id);
                Ok(vec![])
            }
            #[cfg(feature = "back-pressure")]
            SystemMsg::BackPressure(msgs_per_s) => {
                trace!(
                    "Handling msg: BackPressure with requested {} msgs/s, from {}: {:?}",
                    msgs_per_s,
                    sender,
                    msg_id
                );
                // TODO: Factor in med/long term backpressure into general node liveness calculations
                Ok(vec![Cmd::Comm(crate::comm::Cmd::Regulate {
                    peer: sender,
                    msgs_per_s,
                })])
            }
            // The AcceptedOnlineShare for relocation will be received here.
            SystemMsg::JoinResponse(join_response) => {
                match *join_response {
                    JoinResponse::Approval {
                        section_auth,
                        section_chain,
                        ..
                    } => {
                        info!(
                            "Relocation: Aggregating received ApprovalShare from {:?}",
                            sender
                        );
                        info!("Relocation: Successfully aggregated ApprovalShares for joining the network");

                        if let Some(ref mut joining_as_relocated) = self.relocate_state {
                            let new_node = joining_as_relocated.node.clone();
                            let new_name = new_node.name();
                            let previous_name = self.info().name();
                            let new_keypair = new_node.keypair.clone();

                            info!(
                                "Relocation: switching from {:?} to {:?}",
                                previous_name, new_name
                            );

                            let genesis_key = *self.network_knowledge.genesis_key();
                            let prefix_map = self.network_knowledge.prefix_map().clone();

                            let recipients = section_auth.value.elders.clone();

                            let new_network_knowledge = NetworkKnowledge::new(
                                genesis_key,
                                section_chain,
                                section_auth.into_authed_state(),
                                Some(prefix_map),
                            )?;

                            // TODO: confirm whether carry out the switch immediately here
                            //       or still using the cmd pattern.
                            //       As the sending of the JoinRequest as notification
                            //       may require the `node` to be switched to new already.

                            self.relocate(new_keypair.clone(), new_network_knowledge)
                                .await?;

                            trace!(
                                "Relocation: Sending aggregated JoinRequest to {:?}",
                                recipients
                            );

                            self.send_event(Event::Membership(MembershipEvent::Relocated {
                                previous_name,
                                new_keypair,
                            }))
                            .await;

                            trace!("{}", LogMarker::RelocateEnd);
                        } else {
                            warn!("Relocation:  self.relocate_state is not in Progress");
                            return Ok(vec![]);
                        }

                        Ok(vec![])
                    }
                    _ => {
                        debug!("Relocation: Ignoring unexpected join response message: {join_response:?}");
                        Ok(vec![])
                    }
                }
            }
            SystemMsg::DkgFailureAgreement(sig_set) => {
                trace!("Handling msg: Dkg-FailureAgreement from {}", sender);
                self.handle_dkg_failure_agreement(&src_name, &sig_set).await
            }
            SystemMsg::HandoverVotes(votes) => self.handle_handover_msg(sender, votes).await,
            SystemMsg::HandoverAE(gen) => self.handle_handover_anti_entropy(sender, gen).await,
            SystemMsg::JoinRequest(join_request) => {
                trace!("Handling msg: JoinRequest from {}", sender);
                self.handle_join_request(sender, *join_request, comm).await
            }
            SystemMsg::JoinAsRelocatedRequest(join_request) => {
                trace!("Handling msg: JoinAsRelocatedRequest from {}", sender);
                if self.is_not_elder()
                    && join_request.section_key == self.network_knowledge.section_key()
                {
                    return Ok(vec![]);
                }

                self.handle_join_as_relocated_request(sender, *join_request, known_keys, comm)
                    .await
            }
            SystemMsg::MembershipVotes(votes) => {
                let mut cmds = vec![];
                cmds.extend(self.handle_membership_votes(sender, votes).await?);

                Ok(cmds)
            }
            SystemMsg::MembershipAE(gen) => self.handle_membership_anti_entropy(sender, gen).await,
            SystemMsg::Propose {
                proposal,
                sig_share,
            } => {
                if self.is_not_elder() {
                    trace!("Adult handling a Propose msg from {}: {:?}", sender, msg_id);
                }

                trace!("Handling msg: Propose from {}: {:?}", sender, msg_id);

                // lets convert our message into a usable proposal for core
                let core_proposal = match proposal {
                    ProposalMsg::Offline(node_state) => {
                        CoreProposal::Offline(node_state.into_state())
                    }
                    ProposalMsg::SectionInfo { sap, generation } => CoreProposal::SectionInfo {
                        sap: sap.into_state(),
                        generation,
                    },
                    ProposalMsg::NewElders(sap) => CoreProposal::NewElders(sap.into_authed_state()),
                    ProposalMsg::JoinsAllowed(allowed) => CoreProposal::JoinsAllowed(allowed),
                };

                Node::handle_proposal(
                    msg_id,
                    core_proposal,
                    sig_share,
                    sender,
                    &self.network_knowledge,
                    &self.proposal_aggregator,
                )
                .await
            }
            SystemMsg::DkgStart(session_id) => {
                trace!("Handling msg: Dkg-Start {:?} from {}", session_id, sender);
                self.log_dkg_session(&sender.name()).await;
                let our_name = self.info().name();
                if !session_id.contains_elder(our_name) {
                    return Ok(vec![]);
                }
                if let NodeMsgAuthority::Section(authority) = msg_authority {
                    let _existing = self.dkg_sessions.insert(
                        session_id.hash(),
                        DkgSessionInfo {
                            session_id: session_id.clone(),
                            authority,
                        },
                    );
                }
                self.handle_dkg_start(session_id).await
            }
            SystemMsg::DkgMessage {
                session_id,
                message,
            } => {
                trace!(
                    "Handling msg: Dkg-Msg ({:?} - {:?}) from {}",
                    session_id,
                    message,
                    sender
                );
                self.log_dkg_session(&sender.name()).await;
                self.handle_dkg_msg(session_id, message, sender).await
            }
            SystemMsg::DkgFailureObservation {
                session_id,
                sig,
                failed_participants,
            } => {
                trace!("Handling msg: Dkg-FailureObservation from {}", sender);
                self.handle_dkg_failure_observation(session_id, &failed_participants, sig)
            }
            SystemMsg::DkgNotReady {
                message,
                session_id,
            } => {
                self.log_dkg_session(&sender.name()).await;

                self.handle_dkg_not_ready(
                    sender,
                    message,
                    session_id,
                    self.network_knowledge.section_key(),
                )
                .await
            }
            SystemMsg::DkgRetry {
                message_history,
                message,
                session_id,
            } => {
                self.log_dkg_session(&sender.name()).await;

                self.handle_dkg_retry(&session_id, message_history, message, sender)
                    .await
            }
            SystemMsg::NodeCmd(NodeCmd::RecordStorageLevel { node_id, level, .. }) => {
                let changed = self.set_storage_level(&node_id, level).await;
                if changed && level.value() == MIN_LEVEL_WHEN_FULL {
                    // ..then we accept a new node in place of the full node
                    self.joins_allowed = true;
                }
                Ok(vec![])
            }
            SystemMsg::NodeCmd(NodeCmd::ReceiveMetadata { metadata }) => {
                info!("Processing received MetadataExchange packet: {:?}", msg_id);
                self.set_adult_levels(metadata).await;
                Ok(vec![])
            }
            SystemMsg::NodeEvent(NodeEvent::CouldNotStoreData {
                node_id,
                data,
                full,
            }) => {
                info!(
                    "Processing CouldNotStoreData event with MsgId: {:?}",
                    msg_id
                );

                if self.is_elder() {
                    if full {
                        let changed = self
                            .set_storage_level(&node_id, StorageLevel::from(StorageLevel::MAX)?)
                            .await;
                        if changed {
                            // ..then we accept a new node in place of the full node
                            self.joins_allowed = true;
                        }
                    }
                    self.replicate_data(data).await
                } else {
                    error!("Received unexpected message while Adult");
                    Ok(vec![])
                }
            }
            SystemMsg::NodeCmd(NodeCmd::ReplicateData(data_collection)) => {
                info!("ReplicateData MsgId: {:?}", msg_id);
                if self.is_elder() {
                    error!("Received unexpected message while Elder");
                    Ok(vec![])
                } else {
                    let mut cmds = vec![];

                    let section_pk = PublicKey::Bls(self.network_knowledge.section_key());
                    let own_keypair = Keypair::Ed25519(self.keypair.clone());

                    for data in data_collection {
                        // We are an adult here, so just store away!
                        // This may return a DatabaseFull error... but we should have reported storage increase
                        // well before this
                        match self
                            .data_storage
                            .store(&data, section_pk, own_keypair.clone())
                            .await
                        {
                            Ok(level_report) => {
                                info!("Storage level report: {:?}", level_report);
                                cmds.extend(self.record_storage_level_if_any(level_report).await);
                            }
                            Err(DbError::NotEnoughSpace) => {
                                // db full
                                error!("Not enough space to store more data");

                                let node_id = PublicKey::from(self.keypair.public);
                                let msg = SystemMsg::NodeEvent(NodeEvent::CouldNotStoreData {
                                    node_id,
                                    data,
                                    full: true,
                                });

                                cmds.push(self.send_msg_to_our_elders(msg).await?)
                            }
                            Err(error) => {
                                // the rest seem to be non-problematic errors.. (?)
                                error!("Problem storing data, but it was ignored: {error}");
                            }
                        }
                    }

                    Ok(cmds)
                }
            }
            SystemMsg::NodeCmd(NodeCmd::SendAnyMissingRelevantData(known_data_addresses)) => {
                info!(
                    "{:?} MsgId: {:?}",
                    LogMarker::RequestForAnyMissingData,
                    msg_id
                );

                self.get_missing_data_for_node(sender, known_data_addresses)
            }
            SystemMsg::NodeQuery(node_query) => {
                match node_query {
                    // A request from EndUser - via elders - for locally stored data
                    NodeQuery::Data {
                        query,
                        auth,
                        origin,
                        correlation_id,
                    } => {
                        debug!(
                            "Handle NodeQuery with msg_id {:?} and correlation_id {:?}",
                            msg_id, correlation_id,
                        );
                        // There is no point in verifying a sig from a sender A or B here.
                        // Send back response to the sending elder
                        let sender_xorname = msg_authority.get_auth_xorname();
                        self.handle_data_query_at_adult(
                            correlation_id,
                            &query,
                            auth,
                            origin,
                            sender_xorname,
                        )
                        .await
                    }
                }
            }
            SystemMsg::NodeQueryResponse {
                response,
                correlation_id,
                user,
            } => {
                debug!(
                    "{:?}: op_id {:?}, correlation_id: {correlation_id:?}, sender: {sender} origin msg_id: {:?}",
                    LogMarker::ChunkQueryResponseReceviedFromAdult,
                    response.operation_id()?,
                    msg_id
                );
                let sending_nodes_pk = match msg_authority {
                    NodeMsgAuthority::Node(auth) => PublicKey::from(auth.into_inner().node_ed_pk),
                    _ => return Err(Error::InvalidQueryResponseAuthority),
                };

                self.handle_data_query_response_at_elder(
                    correlation_id,
                    response,
                    user,
                    sending_nodes_pk,
                )
                .await
            }
            SystemMsg::DkgSessionUnknown {
                session_id,
                message,
            } => {
                if let Some(session_info) = self.dkg_sessions.get(&session_id.hash()).cloned() {
                    let message_cache = self.dkg_voter.get_cached_msgs(&session_info.session_id);
                    trace!(
                        "Sending DkgSessionInfo {{ {:?}, ... }} to {}",
                        &session_info.session_id,
                        &sender
                    );

                    let node_msg = SystemMsg::DkgSessionInfo {
                        session_id,
                        section_auth: session_info.authority,
                        message_cache,
                        message,
                    };
                    let section_pk = self.network_knowledge.section_key();
                    let wire_msg = WireMsg::single_src(
                        &self.info(),
                        DstLocation::Node {
                            name: sender.name(),
                            section_pk,
                        },
                        node_msg,
                        section_pk,
                    )?;

                    Ok(vec![Cmd::SendMsg {
                        recipients: vec![sender],
                        wire_msg,
                    }])
                } else {
                    warn!("Unknown DkgSessionInfo: {:?} requested", &session_id);
                    Ok(vec![])
                }
            }
            SystemMsg::DkgSessionInfo {
                session_id,
                message_cache,
                section_auth,
                message,
            } => {
                let mut cmds = vec![];
                // Reconstruct the original DKG start message and verify the section signature
                let payload =
                    WireMsg::serialize_msg_payload(&SystemMsg::DkgStart(session_id.clone()))?;
                let auth = section_auth.clone().into_inner();
                if self.network_knowledge.section_key() == auth.sig.public_key {
                    if let Err(err) = AuthorityProof::verify(auth, payload) {
                        error!("Error verifying signature for DkgSessionInfo: {:?}", err);
                        return Ok(cmds);
                    } else {
                        trace!("DkgSessionInfo signature verified");
                    }
                } else {
                    warn!(
                        "Cannot verify DkgSessionInfo: {:?}. Unknown key: {:?}!",
                        &session_id, auth.sig.public_key
                    );
                    let chain = self.network_knowledge().section_chain().await;
                    warn!("Chain: {:?}", chain);
                    return Ok(cmds);
                };
                let _existing = self.dkg_sessions.insert(
                    session_id.hash(),
                    DkgSessionInfo {
                        session_id: session_id.clone(),
                        authority: section_auth,
                    },
                );
                trace!("DkgSessionInfo handling {:?}", session_id);
                cmds.extend(self.handle_dkg_start(session_id.clone()).await?);
                cmds.extend(
                    self.handle_dkg_retry(&session_id, message_cache, message, sender)
                        .await?,
                );
                Ok(cmds)
            }
        }
    }

    async fn record_storage_level_if_any(&self, level: Option<StorageLevel>) -> Vec<Cmd> {
        let mut cmds = vec![];
        if let Some(level) = level {
            info!("Storage has now passed {} % used.", 10 * level.value());
            let node_id = PublicKey::from(self.keypair.public);
            let node_xorname = XorName::from(node_id);

            // we ask the section to record the new level reached
            let msg = SystemMsg::NodeCmd(NodeCmd::RecordStorageLevel {
                section: node_xorname,
                node_id,
                level,
            });

            let dst = DstLocation::Section {
                name: node_xorname,
                section_pk: self.network_knowledge.section_key(),
            };

            cmds.push(Cmd::SignOutgoingSystemMsg { msg, dst });
        }
        cmds
    }

    // Convert the provided NodeMsgAuthority to be a `Section` message
    // authority on successful accumulation. Also return 'true' if
    // current message shall not be processed any further.
    async fn aggregate_msg_and_stop(
        &self,
        msg_authority: &mut NodeMsgAuthority,
        payload: Bytes,
    ) -> Result<bool> {
        let bls_share_auth = if let NodeMsgAuthority::BlsShare(bls_share_auth) = msg_authority {
            bls_share_auth
        } else {
            return Ok(false);
        };

        match SectionAuth::try_authorize(
            self.message_aggregator.clone(),
            bls_share_auth.clone().into_inner(),
            &payload,
        )
        .await
        {
            Ok(section_auth) => {
                info!("Successfully aggregated message");
                *msg_authority = NodeMsgAuthority::Section(section_auth);
                Ok(false)
            }
            Err(AggregatorError::NotEnoughShares) => {
                info!("Not enough shares to aggregate received message");
                Ok(true)
            }
            Err(err) => {
                error!("Error accumulating message at dst: {:?}", err);
                Err(Error::InvalidSignatureShare)
            }
        }
    }
}