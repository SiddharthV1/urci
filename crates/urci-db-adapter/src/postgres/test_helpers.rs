//! Shared test helpers for PostgreSQL adapter tests
//!
//! This module contains common test utilities used by both writer and reader tests.


use alloy_primitives::{Address, B256, FixedBytes, U256};

use testcontainers::{
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

use urci_common::{
    core::{UrciEvent, UrciEventKind, UrciBlockUpdate, UrciTxEvent},
    OperatorRegistered,
};


pub async fn setup_test_db() -> (ContainerAsync<GenericImage>, String) {
    let image = GenericImage::new("timescale/timescaledb", "latest-pg15")
        .with_exposed_port(ContainerPort::Tcp(5432))
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "test_db");

    let container = image.start().await.expect("Failed to start container");
    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");

    let connection_string = format!(
        "postgresql://postgres:postgres@127.0.0.1:{}/test_db",
        host_port
    );

    (container, connection_string)
}


pub fn create_test_validation_result(
    registration_root: B256,
) -> urci_common::RegistrationValidationResult {
    urci_common::RegistrationValidationResult {
        has_fraudulent: false,
        tree: urci_common::MerkleTreeData {
            root: registration_root,
            leaves: vec![B256::from([1; 32]), B256::from([2; 32])],
        },
        signature: vec![urci_common::RegistrationSignatureValidation {
            is_fraudulent: false,
            validation_error: None,
            signature_index: 0,
            merkle_proof: vec![FixedBytes::from([3; 32]), FixedBytes::from([4; 32])],
            signature_data: urci_common::BLS::G2Point {
                x_c0_a: FixedBytes::from([5; 32]),
                x_c0_b: FixedBytes::from([6; 32]),
                x_c1_a: FixedBytes::from([7; 32]),
                x_c1_b: FixedBytes::from([8; 32]),
                y_c0_a: FixedBytes::from([9; 32]),
                y_c0_b: FixedBytes::from([10; 32]),
                y_c1_a: FixedBytes::from([11; 32]),
                y_c1_b: FixedBytes::from([12; 32]),
            },
            pubkey_data: urci_common::BLS::G1Point {
                x_a: FixedBytes::from([13; 32]),
                x_b: FixedBytes::from([14; 32]),
                y_a: FixedBytes::from([15; 32]),
                y_b: FixedBytes::from([16; 32]),
            },
        }],
    }
}


/// Comprehensive test data structure with all event types
pub struct TestFixture {
    pub registration_root: B256,
    pub owner_address: Address,
    pub slasher_address: Address,
    pub committer_address: Address,
    pub blocks: Vec<UrciBlockUpdate>,
    pub block_offset: u64, // Starting block number for this fixture
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture {
    /// Create a complete test fixture with all event types
    pub fn new() -> Self {
        let registration_root = B256::from([1; 32]);
        let owner_address = Address::from([1; 20]);
        let slasher_address = Address::from([50; 20]);
        let committer_address = Address::from([51; 20]);

        TestFixture {
            registration_root,
            owner_address,
            slasher_address,
            committer_address,
            blocks: vec![],
            block_offset: 0,
        }
    }

    /// Create a test fixture starting at a specific block number
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.block_offset = offset;
        self
    }

    /// Add a registration block (genesis block, no parent)
    pub fn with_registration(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let mut block = create_registration_block(
            block_num,
            None, // Genesis block has no parent
            self.registration_root,
            self.owner_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add a collateral block
    pub fn with_collateral(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_collateral_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add an opt-in commitment block
    pub fn with_commitment_opt_in(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_commitment_opt_in_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
            self.slasher_address,
            self.committer_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add an opt-out commitment block
    pub fn with_commitment_opt_out(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_commitment_opt_out_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
            self.slasher_address,
            self.committer_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add a collateral claimed block
    pub fn with_collateral_claimed(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_collateral_claimed_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add a slashing block (SlasherCommitment type by default)
    pub fn with_slashing(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_slashing_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
            self.slasher_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add a Fraud slashing block
    pub fn with_fraud_slashing(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_fraud_slashing_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
            self.slasher_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add a Commitment slashing block
    pub fn with_commitment_slashing(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_commitment_slashing_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
            self.slasher_address,
            self.committer_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add an Equivocation slashing block
    pub fn with_equivocation_slashing(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_equivocation_slashing_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
            self.slasher_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Add an unregistration block
    pub fn with_unregistration(mut self) -> Self {
        let block_num = self.block_offset + self.blocks.len() as u64 + 1;
        let parent_work_id = self.last_work_id();
        let mut block = create_unregistration_block(
            block_num,
            parent_work_id,
            self.registration_root,
            self.owner_address,
        );
        block.work_id = Some(self.compute_work_id(&block));
        self.blocks.push(block);
        self
    }

    /// Get the work_id of the last block
    fn last_work_id(&self) -> Option<B256> {
        self.blocks.last().and_then(|b| b.work_id)
    }

    /// Compute work_id matching the database function:
    /// sha256('WORK_ID||V1' || be64(chain_id) || be64(height) || block_hash || COALESCE(parent_work_id, zeros))
    fn compute_work_id(&self, block: &UrciBlockUpdate) -> B256 {
        use sha2::{Sha256, Digest};
        let chain_id = 1i64; // Test chain ID

        let mut hasher = Sha256::new();
        hasher.update(b"WORK_ID||V1");
        hasher.update(chain_id.to_be_bytes());
        hasher.update(block.block_number.to_be_bytes());
        hasher.update(block.block_hash.as_slice());

        // COALESCE(parent_work_id, zeros)
        if let Some(parent) = block.parent_work_id {
            hasher.update(parent.as_slice());
        } else {
            hasher.update([0u8; 32]);
        }

        B256::from_slice(&hasher.finalize())
    }

    /// Write all blocks from this fixture to the adapter in a single transaction
    pub async fn write_to_adapter<W: crate::traits::AdapterWriter>(&self, adapter: &mut W) -> eyre::Result<()> {
        for block in &self.blocks {
            adapter.write_block(block.clone()).await?;
        }
        Ok(())
    }
}

pub fn create_test_block_with_event(
    block_number: u64,
    parent_work_id: Option<B256>,
) -> UrciBlockUpdate {
    // Block 1: Registration
    // Block 2: Collateral
    if block_number == 1 {
        create_registration_block(
            block_number,
            parent_work_id,
            B256::from([1; 32]),
            Address::from([1; 20]),
        )
    } else {
        create_collateral_block(
            block_number,
            parent_work_id,
            B256::from([1; 32]),
            Address::from([1; 20]),
        )
    }
}

fn create_registration_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = if block_number == 1 {
        B256::ZERO
    } else {
        B256::from([(block_number - 1) as u8; 32])
    };

    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    let event_kind = UrciEventKind::Registration(
        owner,
        OperatorRegistered {
            registrationRoot: registration_root,
            collateralWei: U256::from(1000),
            owner,
        },
        Ok(create_test_validation_result(registration_root)),
    );

    let urc_event = UrciEvent {
        caller_address: owner,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_collateral_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    let event_kind = UrciEventKind::CollateralAdded(urci_common::CollateralAdded {
        registrationRoot: registration_root,
        collateralWei: U256::from(500 * block_number),
    });

    let urc_event = UrciEvent {
        caller_address: owner,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_commitment_opt_in_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    _owner: Address,
    slasher_address: Address,
    committer_address: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    let event_kind = UrciEventKind::OptIn(urci_common::OperatorOptedIn {
        registrationRoot: registration_root,
        slasher: slasher_address,
        committer: committer_address,
    });

    let urc_event = UrciEvent {
        caller_address: slasher_address,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: slasher_address,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_commitment_opt_out_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    _owner: Address,
    slasher_address: Address,
    _committer_address: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    let event_kind = UrciEventKind::OptOut(urci_common::OperatorOptedOut {
        registrationRoot: registration_root,
        slasher: slasher_address,
    });

    let urc_event = UrciEvent {
        caller_address: slasher_address,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: slasher_address,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_collateral_claimed_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    let event_kind = UrciEventKind::CollateralClamed(urci_common::CollateralClaimed {
        registrationRoot: registration_root,
        collateralWei: U256::from(100 * block_number),
    });

    let urc_event = UrciEvent {
        caller_address: owner,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_slashing_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
    slasher_address: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    use urci_common::SlashingCall;

    // Slash amount: 100 wei (reasonable test value that's less than initial 1000 collateral)
    let slash_amount = U256::from(100);

    let event_kind = UrciEventKind::Slashing(
        urci_common::OperatorSlashed {
            slashingType: 0u8, // Default slashing type
            registrationRoot: registration_root,
            owner,
            challenger: slasher_address,
            slasher: slasher_address,
            slashAmountWei: slash_amount,
        },
        slash_amount,
        Box::new(SlashingCall::SlasherCommitment {
            registration_root,
            commitment: Box::new(urci_common::ISlasher::SignedCommitment {
                commitment: urci_common::ISlasher::Commitment {
                    commitmentType: 1u64,
                    payload: vec![1, 2, 3].into(),
                    slasher: slasher_address,
                },
                signature: vec![4, 5, 6].into(),
            }),
            evidence: vec![7, 8, 9],
        }),
    );

    let urc_event = UrciEvent {
        caller_address: slasher_address,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_fraud_slashing_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
    challenger_address: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    use urci_common::SlashingCall;

    // Fraud slash amount: 200 wei
    let slash_amount = U256::from(200);

    let event_kind = UrciEventKind::Slashing(
        urci_common::OperatorSlashed {
            slashingType: 0u8, // Fraud type
            registrationRoot: registration_root,
            owner,
            challenger: challenger_address,
            slasher: registry_address, // Registry itself for Fraud
            slashAmountWei: slash_amount,
        },
        slash_amount,
        Box::new(SlashingCall::Fraud {
            registration_proof: Box::new(urci_common::IRegistry::RegistrationProof {
                registrationRoot: registration_root,
                registration: urci_common::IRegistry::SignedRegistration {
                    pubkey: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([10; 32]),
                        x_b: FixedBytes::from([11; 32]),
                        y_a: FixedBytes::from([12; 32]),
                        y_b: FixedBytes::from([13; 32]),
                    },
                    signature: urci_common::BLS::G2Point {
                        x_c0_a: FixedBytes::from([20; 32]),
                        x_c0_b: FixedBytes::from([21; 32]),
                        x_c1_a: FixedBytes::from([22; 32]),
                        x_c1_b: FixedBytes::from([23; 32]),
                        y_c0_a: FixedBytes::from([24; 32]),
                        y_c0_b: FixedBytes::from([25; 32]),
                        y_c1_a: FixedBytes::from([26; 32]),
                        y_c1_b: FixedBytes::from([27; 32]),
                    },
                },
                merkleProof: vec![FixedBytes::from([30; 32]), FixedBytes::from([31; 32])],
            }),
        }),
    );

    let urc_event = UrciEvent {
        caller_address: challenger_address,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12),
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_commitment_slashing_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
    slasher_address: Address,
    committer_address: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    use urci_common::SlashingCall;

    // Commitment slash amount: 150 wei
    let slash_amount = U256::from(150);

    let event_kind = UrciEventKind::Slashing(
        urci_common::OperatorSlashed {
            slashingType: 1u8, // Commitment type
            registrationRoot: registration_root,
            owner,
            challenger: slasher_address,
            slasher: slasher_address,
            slashAmountWei: slash_amount,
        },
        slash_amount,
        Box::new(SlashingCall::Commitment {
            registration_proof: Box::new(urci_common::IRegistry::RegistrationProof {
                registrationRoot: registration_root,
                registration: urci_common::IRegistry::SignedRegistration {
                    pubkey: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([10; 32]),
                        x_b: FixedBytes::from([11; 32]),
                        y_a: FixedBytes::from([12; 32]),
                        y_b: FixedBytes::from([13; 32]),
                    },
                    signature: urci_common::BLS::G2Point {
                        x_c0_a: FixedBytes::from([20; 32]),
                        x_c0_b: FixedBytes::from([21; 32]),
                        x_c1_a: FixedBytes::from([22; 32]),
                        x_c1_b: FixedBytes::from([23; 32]),
                        y_c0_a: FixedBytes::from([24; 32]),
                        y_c0_b: FixedBytes::from([25; 32]),
                        y_c1_a: FixedBytes::from([26; 32]),
                        y_c1_b: FixedBytes::from([27; 32]),
                    },
                },
                merkleProof: vec![FixedBytes::from([30; 32]), FixedBytes::from([31; 32])],
            }),
            delegation: Box::new(urci_common::ISlasher::SignedDelegation {
                delegation: urci_common::ISlasher::Delegation {
                    proposer: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([40; 32]),
                        x_b: FixedBytes::from([41; 32]),
                        y_a: FixedBytes::from([42; 32]),
                        y_b: FixedBytes::from([43; 32]),
                    },
                    delegate: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([44; 32]),
                        x_b: FixedBytes::from([45; 32]),
                        y_a: FixedBytes::from([46; 32]),
                        y_b: FixedBytes::from([47; 32]),
                    },
                    committer: committer_address,
                    slot: 100u64,
                    metadata: vec![1, 2, 3].into(),
                },
                signature: urci_common::BLS::G2Point {
                    x_c0_a: FixedBytes::from([50; 32]),
                    x_c0_b: FixedBytes::from([51; 32]),
                    x_c1_a: FixedBytes::from([52; 32]),
                    x_c1_b: FixedBytes::from([53; 32]),
                    y_c0_a: FixedBytes::from([54; 32]),
                    y_c0_b: FixedBytes::from([55; 32]),
                    y_c1_a: FixedBytes::from([56; 32]),
                    y_c1_b: FixedBytes::from([57; 32]),
                },
            }),
            commitment: Box::new(urci_common::ISlasher::SignedCommitment {
                commitment: urci_common::ISlasher::Commitment {
                    commitmentType: 2u64,
                    payload: vec![10, 11, 12].into(),
                    slasher: slasher_address,
                },
                signature: vec![13, 14, 15].into(),
            }),
        }),
    );

    let urc_event = UrciEvent {
        caller_address: slasher_address,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12),
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_equivocation_slashing_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
    slasher_address: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    use urci_common::SlashingCall;

    // Equivocation slash amount: 250 wei
    let slash_amount = U256::from(250);

    let event_kind = UrciEventKind::Slashing(
        urci_common::OperatorSlashed {
            slashingType: 2u8, // Equivocation type
            registrationRoot: registration_root,
            owner,
            challenger: slasher_address,
            slasher: slasher_address,
            slashAmountWei: slash_amount,
        },
        slash_amount,
        Box::new(SlashingCall::Equivocation {
            registration_proof: Box::new(urci_common::IRegistry::RegistrationProof {
                registrationRoot: registration_root,
                registration: urci_common::IRegistry::SignedRegistration {
                    pubkey: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([10; 32]),
                        x_b: FixedBytes::from([11; 32]),
                        y_a: FixedBytes::from([12; 32]),
                        y_b: FixedBytes::from([13; 32]),
                    },
                    signature: urci_common::BLS::G2Point {
                        x_c0_a: FixedBytes::from([20; 32]),
                        x_c0_b: FixedBytes::from([21; 32]),
                        x_c1_a: FixedBytes::from([22; 32]),
                        x_c1_b: FixedBytes::from([23; 32]),
                        y_c0_a: FixedBytes::from([24; 32]),
                        y_c0_b: FixedBytes::from([25; 32]),
                        y_c1_a: FixedBytes::from([26; 32]),
                        y_c1_b: FixedBytes::from([27; 32]),
                    },
                },
                merkleProof: vec![FixedBytes::from([30; 32]), FixedBytes::from([31; 32])],
            }),
            delegation_one: Box::new(urci_common::ISlasher::SignedDelegation {
                delegation: urci_common::ISlasher::Delegation {
                    proposer: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([60; 32]),
                        x_b: FixedBytes::from([61; 32]),
                        y_a: FixedBytes::from([62; 32]),
                        y_b: FixedBytes::from([63; 32]),
                    },
                    delegate: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([64; 32]),
                        x_b: FixedBytes::from([65; 32]),
                        y_a: FixedBytes::from([66; 32]),
                        y_b: FixedBytes::from([67; 32]),
                    },
                    committer: slasher_address,
                    slot: 100u64,
                    metadata: vec![1, 2, 3].into(),
                },
                signature: urci_common::BLS::G2Point {
                    x_c0_a: FixedBytes::from([70; 32]),
                    x_c0_b: FixedBytes::from([71; 32]),
                    x_c1_a: FixedBytes::from([72; 32]),
                    x_c1_b: FixedBytes::from([73; 32]),
                    y_c0_a: FixedBytes::from([74; 32]),
                    y_c0_b: FixedBytes::from([75; 32]),
                    y_c1_a: FixedBytes::from([76; 32]),
                    y_c1_b: FixedBytes::from([77; 32]),
                },
            }),
            delegation_two: Box::new(urci_common::ISlasher::SignedDelegation {
                delegation: urci_common::ISlasher::Delegation {
                    proposer: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([60; 32]), // Same proposer
                        x_b: FixedBytes::from([61; 32]),
                        y_a: FixedBytes::from([62; 32]),
                        y_b: FixedBytes::from([63; 32]),
                    },
                    delegate: urci_common::BLS::G1Point {
                        x_a: FixedBytes::from([80; 32]), // Different delegate (equivocation!)
                        x_b: FixedBytes::from([81; 32]),
                        y_a: FixedBytes::from([82; 32]),
                        y_b: FixedBytes::from([83; 32]),
                    },
                    committer: slasher_address,
                    slot: 100u64,
                    metadata: vec![1, 2, 3].into(),
                },
                signature: urci_common::BLS::G2Point {
                    x_c0_a: FixedBytes::from([90; 32]),
                    x_c0_b: FixedBytes::from([91; 32]),
                    x_c1_a: FixedBytes::from([92; 32]),
                    x_c1_b: FixedBytes::from([93; 32]),
                    y_c0_a: FixedBytes::from([94; 32]),
                    y_c0_b: FixedBytes::from([95; 32]),
                    y_c1_a: FixedBytes::from([96; 32]),
                    y_c1_b: FixedBytes::from([97; 32]),
                },
            }),
        }),
    );

    let urc_event = UrciEvent {
        caller_address: slasher_address,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12),
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}

fn create_unregistration_block(
    block_number: u64,
    parent_work_id: Option<B256>,
    registration_root: B256,
    owner: Address,
) -> UrciBlockUpdate {
    use alloy_primitives::{Log, LogData};

    let block_hash = B256::from([block_number as u8; 32]);
    let parent_hash = B256::from([(block_number - 1) as u8; 32]);
    let tx_hash = B256::from([100 + block_number as u8; 32]);
    let registry_address = alloy_primitives::address!("0000000000000000000000000000000000000001");

    let log = Log {
        address: registry_address,
        data: LogData::new_unchecked(vec![], Default::default()),
    };

    let event_kind = UrciEventKind::Unregistration(urci_common::OperatorUnregistered {
        registrationRoot: registration_root,
    });

    let urc_event = UrciEvent {
        caller_address: owner,
        call_depth: 0,
        call_index_at_depth: 0,
        input: None,
        log_index: block_number,
        event: event_kind,
    };

    let tx_event = UrciTxEvent {
        tx_sender_address: owner,
        tx_value: U256::ZERO,
        tx_type: 0,
        transaction_hash: tx_hash,
        transaction_index: 0,
        transaction_input: None,
        urc_logs: vec![(log, block_number)],
        urc_events: vec![urc_event],
        trace: vec![],
        ..Default::default()
    };

    UrciBlockUpdate {
        block_number,
        block_hash,
        parent_block_hash: parent_hash,
        timestamp: 1700000000 + (block_number * 12), // Realistic: ~12s per block from Nov 2023
        work_id: None,
        parent_work_id,
        events: vec![tx_event],
        reorged: false,
        system_error: None,
    }
}


pub async fn setup_test_prerequisites(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    chain_id: i64,
    event_type: &str,
    caller_address: Address,
    registry_address: Address,
) -> eyre::Result<i64> {
    use sqlx::Row;

    let block_hash = B256::from([1; 32]);
    let tx_hash = B256::from([2; 32]);

    // Insert event and return its ID
    let event_id = sqlx::query(
        "INSERT INTO events (chain_id, block_number, block_hash, tx_hash, tx_index, log_index, event_type, decoded_data)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id"
    )
    .bind(chain_id)
    .bind(1i64)
    .bind(block_hash.as_slice())
    .bind(tx_hash.as_slice())
    .bind(0i32)
    .bind(0i32)
    .bind(event_type)
    .bind(serde_json::json!({
        "caller_address": caller_address.to_string(),
        "registry_address": registry_address.to_string()
    }))
    .fetch_one(&mut **tx)
    .await?
    .get::<i64, _>("id");

    Ok(event_id)
}

/// Create test registry config with reasonable defaults
pub fn create_test_config() -> urci_common::IRegistry::Config {
    urci_common::IRegistry::Config {
        minCollateralWei: alloy_primitives::aliases::U80::from(1000000000000000000u128), // 1 ETH
        fraudProofWindow: 10u32,
        unregistrationDelay: 5u32,
        slashWindow: 20u32,
        optInDelay: 3u32,
    }
}
