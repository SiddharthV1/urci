/// SQL query constants to reduce duplication
/// All frequently used SQL queries are defined here as constants
// ============================================================================
// ADDRESS TABLE
// ============================================================================
pub const INSERT_ADDRESS: &str =
    "INSERT INTO address (address, chain_id, is_contract)
     VALUES ($1, $2, $3)
     ON CONFLICT (address, chain_id) DO NOTHING";

// Removed - use INSERT_ADDRESS instead

// ============================================================================
// EVENTS TABLE
// ============================================================================

pub const INSERT_EVENT_RETURNING_ID: &str =
    "INSERT INTO events (
        chain_id, block_number, block_hash, tx_hash, tx_index, log_index,
        event_type, decoded_data, writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
     RETURNING id";

// ============================================================================
// TRANSACTIONS TABLE
// ============================================================================

pub const INSERT_TRANSACTION_IGNORE: &str =
    "INSERT INTO transactions (
        chain_id, hash, block_hash, from_address, to_address,
        value, gas, gas_price, nonce, input, tx_index
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
     ON CONFLICT (chain_id, hash) DO NOTHING";

// ============================================================================
// BLOCKS TABLE
// ============================================================================

pub const INSERT_BLOCK_UPSERT: &str =
    "INSERT INTO blocks (
        chain_id, number, hash, parent_hash, parent_work_id,
        canonical, finalized, timestamp, active_event_id, session_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
     ON CONFLICT (chain_id, hash) DO UPDATE
     SET canonical = EXCLUDED.canonical,
         finalized = EXCLUDED.finalized,
         active_event_id = EXCLUDED.active_event_id,
         session_id = EXCLUDED.session_id";

/// Test-specific block insert that marks blocks as root to bypass parent_work_id constraint
/// Uses parent_work_id = parent_work_id trick to keep existing value on conflict
pub const INSERT_BLOCK_AS_ROOT: &str =
    "INSERT INTO blocks (
        chain_id, number, hash, parent_hash, parent_work_id,
        canonical, finalized, timestamp, active_event_id, session_id, is_root
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, TRUE)
     ON CONFLICT (chain_id, hash) DO UPDATE
     SET canonical = EXCLUDED.canonical,
         finalized = EXCLUDED.finalized,
         active_event_id = EXCLUDED.active_event_id,
         session_id = EXCLUDED.session_id,
         parent_work_id = blocks.parent_work_id,
         is_root = TRUE";

pub const INSERT_BLOCK_RETURNING_WORK_ID: &str =
    "INSERT INTO blocks (
        chain_id, number, hash, parent_hash, parent_work_id,
        timestamp, canonical, finalized, session_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
     RETURNING work_id";

pub const SELECT_BLOCK_WORK_ID: &str =
    "SELECT work_id FROM blocks WHERE chain_id = $1 AND number = $2";

// ============================================================================
// OPERATORS TABLE
// ============================================================================

pub const INSERT_OPERATOR_IGNORE: &str =
    "INSERT INTO operators (
        registration_root, chain_id, owner_address, num_keys,
        registration_processed, event_id, writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7)
     ON CONFLICT (chain_id, registration_root) DO NOTHING";

pub const UPDATE_OPERATOR_UNREGISTER: &str =
    "UPDATE operators
     SET registration_processed = FALSE, updated_at = NOW()
     WHERE registration_root = $1 AND chain_id = $2";

// ============================================================================
// OPERATOR_DATA TABLE
// ============================================================================

pub const INSERT_OPERATOR_DATA: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, event_id, writer_id
     ) VALUES ($1, $2, $3::operator_event_type, $4, $5, $6)";

pub const INSERT_OPERATOR_DATA_REGISTERED: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, registered_at, event_id, writer_id
     ) VALUES ($1, $2, 'OperatorRegistered'::operator_event_type, $3, $4, $5, $6)";

pub const INSERT_OPERATOR_DATA_UNREGISTERED: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, deleted, event_id, writer_id
     ) VALUES ($1, $2, 'OperatorUnregistered'::operator_event_type, $3, TRUE, $4, $5)";

// ============================================================================
// OPERATOR_COLLATERAL TABLE
// ============================================================================

pub const INSERT_OPERATOR_COLLATERAL_DELTA: &str =
    "INSERT INTO operator_collateral (
        registration_root, chain_id, collateral_wei_delta, event_id, writer_id
     ) VALUES ($1, $2, $3::NUMERIC, $4, $5)";

pub const INSERT_OPERATOR_COLLATERAL_FULL: &str =
    "INSERT INTO operator_collateral (
        registration_root, chain_id, collateral_wei_delta, collateral_wei_total, event_id, writer_id
     ) VALUES ($1, $2, $3::NUMERIC, $3::NUMERIC, $4, $5)";

// ============================================================================
// BLS SIGNATURE/PUBKEY TABLES
// ============================================================================

pub const INSERT_BLS_SIGNATURE_G2_RETURNING_ID: &str =
    "INSERT INTO bls_signature_g2_point (
        json,
        bls_signature_g2_point_c0_x_a, bls_signature_g2_point_c0_x_b,
        bls_signature_g2_point_c0_y_a, bls_signature_g2_point_c0_y_b,
        bls_signature_g2_point_c1_x_a, bls_signature_g2_point_c1_x_b,
        bls_signature_g2_point_c1_y_a, bls_signature_g2_point_c1_y_b,
        writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
     RETURNING id";

pub const INSERT_BLS_PUBKEY_G1_RETURNING_ID: &str =
    "INSERT INTO bls_pubkey_g1_point (
        json,
        bls_pubkey_g1_point_x_a, bls_pubkey_g1_point_x_b,
        bls_pubkey_g1_point_y_a, bls_pubkey_g1_point_y_b,
        writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6)
     RETURNING id";

// ============================================================================
// SLASHER/COMMITTER TABLES
// ============================================================================

pub const INSERT_SLASHER_RETURNING_ID: &str =
    "INSERT INTO slasher (chain_id, address, event_id, writer_id)
     VALUES ($1, $2, $3, $4)
     RETURNING id";

pub const INSERT_COMMITTER_RETURNING_ID: &str =
    "INSERT INTO committer (chain_id, address, event_id, writer_id)
     VALUES ($1, $2, $3, $4)
     RETURNING id";

// ============================================================================
// OPERATOR_SLASHER_COMMITMENT TABLE
// ============================================================================

pub const INSERT_OPERATOR_SLASHER_COMMITMENT_OPTIN: &str =
    "INSERT INTO operator_slasher_commitment (
        registration_root, chain_id, slasher_id, committer_id, opted_in_at, writer_id
     ) VALUES ($1, $2, $3, $4, NOW(), $5)";

pub const INSERT_OPERATOR_SLASHER_COMMITMENT_OPTOUT_SELECT: &str =
    "INSERT INTO operator_slasher_commitment (
        registration_root, chain_id, slasher_id, committer_id, opted_out_at, writer_id
     ) SELECT
        $1, $2, slasher_id, committer_id, NOW(), $3
     FROM operator_slasher_commitment
     WHERE registration_root = $1
       AND chain_id = $2
       AND slasher_id = (SELECT id FROM slasher WHERE address = $4 AND chain_id = $2 LIMIT 1)
     ORDER BY created_at DESC
     LIMIT 1";

// ============================================================================
// WRITER STATUS TABLE
// ============================================================================

pub const UPDATE_WRITER_STATUS: &str =
    "UPDATE writer_status
     SET seen_block_to = $1, chain_tip = $1, last_indexed_event_id = $2, updated_at = NOW()
     WHERE writer_id = $3 AND chain_id = $4";

// ============================================================================
// MERKLE TREE TABLE
// ============================================================================

pub const INSERT_MERKLE_TREE: &str =
    "INSERT INTO merkle_tree (
        root, leafs, leaf_count
     ) VALUES ($1, $2, $3)
     ON CONFLICT (root) DO NOTHING";

pub const SELECT_MERKLE_TREE_LEAF_COUNT: &str =
    "SELECT leaf_count FROM merkle_tree WHERE root = $1";

// ============================================================================
// MERKLE INCLUSION PROOF TABLE
// ============================================================================

pub const INSERT_MERKLE_PROOF_RETURNING_ID: &str =
    "INSERT INTO merkle_inclusion_proof (
        merkle_index, merkle_inclusion_proof, bytes_32, writer_id
     ) VALUES ($1, $2, $3, $4)
     RETURNING id";

// ============================================================================
// SIGNED REGISTRATION TABLE
// ============================================================================

pub const INSERT_SIGNED_REGISTRATION: &str =
    "INSERT INTO signed_registration (
        owner_address, chain_id, bls_signature_id, bls_pub_key_id,
        merkle_tree_id, merkle_inclusion_proof_id,
        verification_status, validation_error, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6, $7::verification_status, $8, $9)";

// ============================================================================
// SLASH TABLES
// ============================================================================

pub const INSERT_SLASH_REGISTRATION: &str =
    "INSERT INTO slash_registration (
        event_id, registration_root, chain_id, operator_data_id, signed_commitment_id,
        sender_address, sender_reward_wei, burned_wei, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6, $7::NUMERIC, $8::NUMERIC, $9)";

pub const INSERT_SLASH_OFFCHAIN_DELEGATION_COMMITMENT: &str =
    "INSERT INTO slash_offchain_delegation_commitment (
        chain_id, operator_data_id, signed_delegation_id, signed_commitment_id,
        committer_id, sender_address, sender_reward_wei, burned_wei, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6, $7::NUMERIC, $8::NUMERIC, $9)";

pub const INSERT_SLASH_COMMITMENT: &str =
    "INSERT INTO slash_commitment (
        event_id, chain_id, operator_data_id, signed_commitment_id,
        sender_address, sender_reward_wei, burned_wei, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6::NUMERIC, $7::NUMERIC, $8)";

pub const INSERT_SLASH_EQUIVOCATION: &str =
    "INSERT INTO slash_equivocation (
        event_id, chain_id, operator_data_id, signed_commitment_id,
        sender_address, sender_reward_wei, burned_wei,
        committer_id, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6::NUMERIC, $7::NUMERIC, $8, $9)";

// ============================================================================
// OPERATOR COLLATERAL DELTA (WITHOUT TOTAL)
// ============================================================================

pub const INSERT_OPERATOR_COLLATERAL_NEGATIVE: &str =
    "INSERT INTO operator_collateral (
        registration_root, chain_id, collateral_wei_delta, writer_id
    ) VALUES ($1, $2, $3::NUMERIC, $4)";

// ============================================================================
// CHAIN ROOTS TABLE
// ============================================================================

pub const INSERT_CHAIN_ROOTS_IGNORE: &str =
    "INSERT INTO chain_roots (chain_id, start_height)
     VALUES ($1, $2)
     ON CONFLICT (chain_id) DO NOTHING";

// ============================================================================
// WRITERS TABLE
// ============================================================================

pub const INSERT_WRITER: &str =
    "INSERT INTO writers (id, name, session)
     VALUES ($1, $2, $3)
     ON CONFLICT (id) DO UPDATE
     SET session = EXCLUDED.session";

// ============================================================================
// WRITER STATUS INSERT
// ============================================================================

pub const INSERT_WRITER_STATUS: &str =
    "INSERT INTO writer_status (
        writer_id, session_id, chain_id, seen_block_from,
        seen_block_to, chain_tip, updated_at
    ) VALUES ($1, $2, $3, $4, $5, $6, NOW())";

// ============================================================================
// BLOCKS INSERT WITH UPSERT FOR CANONICAL/FINALIZED
// ============================================================================

pub const INSERT_BLOCK_WITH_PARENT: &str =
    "INSERT INTO blocks (
        chain_id, number, hash, parent_hash, parent_work_id,
        canonical, finalized, timestamp, session_id
    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ON CONFLICT (chain_id, hash) DO UPDATE
    SET canonical = EXCLUDED.canonical,
        finalized = EXCLUDED.finalized,
        session_id = EXCLUDED.session_id";

// ============================================================================
// OPERATOR DATA INSERT TYPES
// ============================================================================

pub const INSERT_OPERATOR_DATA_COLLATERAL_ADDED: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, event_id, writer_id
     ) VALUES ($1, $2, 'CollateralAdded'::operator_event_type, $3, $4, $5)";

pub const INSERT_OPERATOR_DATA_COLLATERAL_CLAIMED: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, event_id, writer_id
     ) VALUES ($1, $2, 'CollateralClaimed'::operator_event_type, $3, $4, $5)";

pub const INSERT_OPERATOR_DATA_OPTED_IN: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, event_id, writer_id
     ) VALUES ($1, $2, 'OperatorOptedIn'::operator_event_type, $3, $4, $5)";

pub const INSERT_OPERATOR_DATA_OPTED_OUT: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, event_id, writer_id
     ) VALUES ($1, $2, 'OperatorOptedOut'::operator_event_type, $3, $4, $5)";

pub const INSERT_OPERATOR_DATA_SLASHED: &str =
    "INSERT INTO operator_data (
        registration_root, chain_id, event_type, event, slashed_at,
        equivocated, event_id, writer_id
    ) VALUES ($1, $2, 'OperatorSlashed'::operator_event_type, $3, $4, $5, $6, $7)
    RETURNING id";

// ============================================================================
// BLOCKS TABLE QUERIES
// ============================================================================

/// Get canonical head work_id after reorg
pub const SELECT_CANONICAL_HEAD_WORK_ID: &str =
    "SELECT work_id FROM blocks WHERE chain_id = $1 AND canonical = true ORDER BY number DESC LIMIT 1";

/// Get last finalized block work_id
pub const SELECT_LAST_FINALIZED_WORK_ID: &str =
    "SELECT work_id FROM blocks WHERE chain_id = $1 AND finalized = true ORDER BY number DESC LIMIT 1";

pub const UPDATE_BLOCKS_CANONICAL_FALSE: &str =
    "UPDATE blocks SET canonical = false WHERE number >= $1 AND number <= $2";

pub const UPDATE_BLOCKS_FINALIZED_TRUE: &str =
    "UPDATE blocks SET finalized = true
     WHERE number <= $1 AND canonical = true AND finalized = false";

pub const CALL_FINALIZE_SESSION_BLOCKS: &str =
    "SELECT finalize_session_blocks($1, $2)";

pub const SELECT_COUNT_BLOCKS_BY_CHAIN: &str =
    "SELECT COUNT(*) FROM blocks WHERE chain_id = $1";

pub const SELECT_MAX_CANONICAL_BLOCK: &str =
    "SELECT MAX(number) FROM blocks WHERE canonical = true";

pub const SELECT_LATEST_FINALIZED_BLOCK: &str =
    "SELECT number, work_id FROM blocks
     WHERE finalized = true
     ORDER BY number DESC
     LIMIT 1";

pub const SELECT_LATEST_CANONICAL_BLOCK: &str =
    "SELECT number, work_id FROM blocks
     WHERE canonical = true
     ORDER BY number DESC
     LIMIT 1";

pub const SELECT_BLOCK_BY_NUMBER: &str =
    "SELECT b.number, b.hash, b.work_id, b.timestamp
     FROM blocks b
     WHERE b.number = $1";

pub const SELECT_BLOCK_RANGE: &str =
    "SELECT b.number, b.hash, b.work_id, b.timestamp
     FROM blocks b
     WHERE b.number >= $1 AND b.number <= $2
     ORDER BY b.number ASC";

pub const SELECT_CANONICAL_BLOCK: &str =
    "SELECT number, hash, work_id FROM blocks
     WHERE canonical = true
     ORDER BY number DESC
     LIMIT 1";

pub const SELECT_FINALIZED_BLOCK: &str =
    "SELECT number, hash, work_id FROM blocks
     WHERE finalized = true AND chain_id = $1
     ORDER BY number DESC
     LIMIT 1";

pub const SELECT_BLOCK_BY_NUMBER_CANONICAL: &str =
    "SELECT number, hash, work_id FROM blocks
     WHERE number = $1 AND canonical = true";

pub const SELECT_BLOCK_BY_HASH_CANONICAL: &str =
    "SELECT number, hash, work_id FROM blocks
     WHERE hash = $1 AND canonical = true";

pub const SELECT_CANONICAL_BLOCK_DETAILED: &str =
    "SELECT b.number, b.hash, b.work_id, b.timestamp
     FROM blocks b
     WHERE b.canonical = true AND b.chain_id = $1
     ORDER BY b.number DESC
     LIMIT 1";

pub const SELECT_FINALIZED_BLOCK_DETAILED: &str =
    "SELECT b.number, b.hash, b.work_id, b.timestamp
     FROM blocks b
     WHERE b.finalized = true AND b.chain_id = $1
     ORDER BY b.number DESC
     LIMIT 1";

// ============================================================================
// API BLOCK QUERIES (with events)
// ============================================================================

pub const SELECT_BLOCK_WITH_EVENTS_BY_NUMBER: &str =
    "SELECT
        b.chain_id,
        b.number,
        b.hash,
        b.parent_hash,
        b.work_id,
        b.parent_work_id,
        b.timestamp,
        b.canonical,
        b.finalized,
        b.active_event_id,
        (SELECT COUNT(*) FROM events e WHERE e.block_hash = b.hash AND e.chain_id = b.chain_id) as event_count
     FROM blocks b
     WHERE b.chain_id = $1 AND b.number = $2 AND b.canonical = true";

pub const SELECT_BLOCK_WITH_EVENTS_BY_HASH: &str =
    "SELECT
        b.chain_id,
        b.number,
        b.hash,
        b.parent_hash,
        b.work_id,
        b.parent_work_id,
        b.timestamp,
        b.canonical,
        b.finalized,
        b.active_event_id,
        (SELECT COUNT(*) FROM events e WHERE e.block_hash = b.hash AND e.chain_id = b.chain_id) as event_count
     FROM blocks b
     WHERE b.chain_id = $1 AND b.hash = $2 AND b.canonical = true";

pub const SELECT_BLOCK_HEAD_WITH_EVENTS: &str =
    "SELECT
        b.chain_id,
        b.number,
        b.hash,
        b.parent_hash,
        b.work_id,
        b.parent_work_id,
        b.timestamp,
        b.canonical,
        b.finalized,
        b.active_event_id,
        (SELECT COUNT(*) FROM events e WHERE e.block_hash = b.hash AND e.chain_id = b.chain_id) as event_count
     FROM blocks b
     WHERE b.chain_id = $1 AND b.canonical = true
     ORDER BY b.number DESC
     LIMIT 1";

pub const SELECT_BLOCK_FINALIZED_WITH_EVENTS: &str =
    "SELECT
        b.chain_id,
        b.number,
        b.hash,
        b.parent_hash,
        b.work_id,
        b.parent_work_id,
        b.timestamp,
        b.canonical,
        b.finalized,
        b.active_event_id,
        (SELECT COUNT(*) FROM events e WHERE e.block_hash = b.hash AND e.chain_id = b.chain_id) as event_count
     FROM blocks b
     WHERE b.chain_id = $1 AND b.finalized = true
     ORDER BY b.number DESC
     LIMIT 1";

pub const SELECT_EVENTS_BY_BLOCK: &str =
    "SELECT
        e.id,
        e.chain_id,
        e.block_number,
        e.block_hash,
        e.tx_hash,
        e.tx_index,
        e.log_index,
        e.event_type,
        e.decoded_data,
        e.writer_id
     FROM events e
     WHERE e.chain_id = $1 AND e.block_hash = $2
     ORDER BY e.log_index";

// ============================================================================
// CHAIN ROOTS QUERIES
// ============================================================================

pub const SELECT_CHAIN_ROOT_START_HEIGHT: &str =
    "SELECT start_height FROM chain_roots WHERE chain_id = $1";

// ============================================================================
// SYSTEM QUERIES
// ============================================================================

pub const CHECK_TABLES_EXIST: &str =
    "SELECT EXISTS(SELECT 1 FROM information_schema.tables
     WHERE table_schema = 'public' AND table_name = 'blocks')";

pub const TRY_ADVISORY_LOCK: &str =
    "SELECT pg_advisory_xact_lock($1)";

// ============================================================================
// ADDRESS INSERT WITHOUT CONTRACT FLAG
// ============================================================================

// Removed - use INSERT_ADDRESS with proper is_contract parameter

// ============================================================================
// SLASH COMMITMENT VARIANT (WITHOUT EVENT_ID)
// ============================================================================

pub const INSERT_SLASH_COMMITMENT_VARIANT: &str =
    "INSERT INTO slash_commitment (
        chain_id, operator_data_id, signed_commitment_id, committer_id,
        sender_address, evidence, sender_reward_wei, burned_wei, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6, $7::NUMERIC, $8::NUMERIC, $9)";

pub const INSERT_SLASH_EQUIVOCATION_VARIANT: &str =
    "INSERT INTO slash_equivocation (
        chain_id, operator_data_id, signed_delegation_id_1, signed_delegation_id_2,
        sender_address, sender_reward_wei, burned_wei, writer_id
    ) VALUES ($1, $2, $3, $4, $5, $6::NUMERIC, $7::NUMERIC, $8)";

pub const INSERT_COMMITTER_VARIANT: &str =
    "INSERT INTO committer (address, chain_id, writer_id)
     VALUES ($1, $2, $3)
     RETURNING id";

pub const INSERT_SIGNED_COMMITMENT: &str =
    "INSERT INTO signed_commitment (
        commitment_type, payload, slasher_address, chain_id,
        commitment, event_id, writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7)
     RETURNING id";

pub const INSERT_SIGNED_DELEGATION: &str =
    "INSERT INTO signed_delegation (
        chain_id, proposer, delegate, committer, slot, metadata,
        message, signature, event_id, writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
     RETURNING id";

pub const INSERT_OPERATOR_COLLATERAL_NEGATIVE_VARIANT: &str =
    "INSERT INTO operator_collateral (
        registration_root, chain_id, collateral_wei_delta, writer_id
    ) VALUES ($1, $2, $3::NUMERIC, $4)";

pub const INSERT_WRITER_STATUS_UPSERT: &str =
    "INSERT INTO writer_status (writer_id, session_id, chain_id, seen_block_from, seen_block_to, chain_tip, last_indexed_event_id, updated_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
     ON CONFLICT (writer_id, session_id, chain_id)
     DO UPDATE SET
        seen_block_from = LEAST(writer_status.seen_block_from, EXCLUDED.seen_block_from),
        seen_block_to = GREATEST(writer_status.seen_block_to, EXCLUDED.seen_block_to),
        chain_tip = EXCLUDED.chain_tip,
        last_indexed_event_id = EXCLUDED.last_indexed_event_id,
        updated_at = NOW()";

pub const INSERT_WRITERS_VARIANT: &str =
    "INSERT INTO writers (id, name, session, config)
     VALUES ($1, $2, $3, $4)
     ON CONFLICT (session) DO UPDATE
     SET updated_at = NOW()";

pub const SELECT_DERIVE_WRITER_ID: &str =
    "SELECT derive_writer_id($1::jsonb)";

pub const INSERT_CHAIN: &str =
    "INSERT INTO chain (id, name) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING";

pub const UPDATE_WRITER_CONFIG: &str =
    "UPDATE writers SET config = $1 WHERE id = $2";

pub const INSERT_CONFIG: &str =
    "INSERT INTO config (
        chain_id, contract_address, min_collateral_wei,
        fraud_proof_window, unregistration_delay, slash_window, opt_in_delay, writer_id
     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
     ON CONFLICT (chain_id, contract_address) DO UPDATE
     SET min_collateral_wei = EXCLUDED.min_collateral_wei,
         fraud_proof_window = EXCLUDED.fraud_proof_window,
         unregistration_delay = EXCLUDED.unregistration_delay,
         slash_window = EXCLUDED.slash_window,
         opt_in_delay = EXCLUDED.opt_in_delay,
         writer_id = EXCLUDED.writer_id,
         updated_at = NOW()";

// ============================================================================
// READER QUERIES (for urc-api)
// ============================================================================

pub const SELECT_CONTRACT_CONFIG: &str =
    "SELECT contract_address,
            min_collateral_wei::TEXT as min_collateral_wei,
            fraud_proof_window,
            unregistration_delay, slash_window, opt_in_delay
     FROM config
     WHERE chain_id = $1
     LIMIT 1";

pub const SELECT_OPERATOR_KEYS: &str =
    "SELECT
        mip.merkle_index,
        mip.merkle_inclusion_proof,
        mip.bytes_32
    FROM merkle_inclusion_proof mip
    WHERE mip.registration_root = $1
        AND mip.chain_id = $2
    ORDER BY mip.merkle_index";

pub const SELECT_OPERATORS_BY_PUBKEY: &str =
    "SELECT
        mip.registration_root,
        mip.chain_id::bigint as chain_id,
        mip.merkle_index,
        o.owner_address
    FROM merkle_inclusion_proof mip
    JOIN operators o ON o.registration_root = mip.registration_root
        AND o.chain_id = mip.chain_id
    WHERE mip.bytes_32 = $1
        AND mip.chain_id = $2
    ORDER BY mip.merkle_index";

pub const SELECT_OPERATOR_COLLATERAL_HISTORY: &str =
    "SELECT
        collateral_wei_total,
        collateral_wei_delta,
        event_id,
        created_at
    FROM operator_collateral
    WHERE registration_root = $1
        AND chain_id = $2
    ORDER BY created_at ASC";

pub const SELECT_OPERATOR_COMMITMENTS: &str =
    "SELECT
        s.address as slasher_address,
        c.address as committer_address,
        osc.opted_in_at,
        osc.opted_out_at,
        osc.slashed
    FROM operator_slasher_commitment osc
    LEFT JOIN slasher s ON s.id = osc.slasher_id
    LEFT JOIN committer c ON c.id = osc.committer_id
    WHERE osc.registration_root = $1
        AND osc.chain_id = $2::integer
    ORDER BY osc.created_at DESC
    LIMIT $3";

pub const SELECT_OPERATOR_SLASHING: &str =
    "WITH all_slashes AS (
        SELECT 'SlashRegistration' as slash_type,
               sr.event_id,
               b.number as block_number,
               sr.sender_address,
               sr.sender_reward_wei,
               sr.burned_wei,
               sr.created_at
        FROM slash_registration sr
        JOIN operator_data od ON od.id = sr.operator_data_id
        JOIN events e ON e.id = od.event_id
        JOIN blocks b ON b.number = e.block_number AND b.chain_id = e.chain_id
        WHERE sr.registration_root = $1 AND sr.chain_id = $2

        UNION ALL

        SELECT 'SlashCommitment' as slash_type,
               e.id as event_id,
               b.number as block_number,
               sc.sender_address,
               sc.sender_reward_wei,
               sc.burned_wei,
               sc.created_at
        FROM slash_commitment sc
        JOIN operator_data od ON od.id = sc.operator_data_id
        JOIN events e ON e.id = od.event_id
        JOIN blocks b ON b.number = e.block_number AND b.chain_id = e.chain_id
        JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
        WHERE o.registration_root = $1 AND sc.chain_id = $2

        UNION ALL

        SELECT 'SlashOffchainDelegationCommitment' as slash_type,
               e.id as event_id,
               b.number as block_number,
               sodc.sender_address,
               sodc.sender_reward_wei,
               sodc.burned_wei,
               sodc.created_at
        FROM slash_offchain_delegation_commitment sodc
        JOIN operator_data od ON od.id = sodc.operator_data_id
        JOIN events e ON e.id = od.event_id
        JOIN blocks b ON b.number = e.block_number AND b.chain_id = e.chain_id
        JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
        WHERE o.registration_root = $1 AND sodc.chain_id = $2

        UNION ALL

        SELECT 'SlashEquivocation' as slash_type,
               e.id as event_id,
               b.number as block_number,
               se.sender_address,
               se.sender_reward_wei,
               se.burned_wei,
               se.created_at
        FROM slash_equivocation se
        JOIN operator_data od ON od.id = se.operator_data_id
        JOIN events e ON e.id = od.event_id
        JOIN blocks b ON b.number = e.block_number AND b.chain_id = e.chain_id
        JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
        WHERE o.registration_root = $1 AND se.chain_id = $2
    )
    SELECT * FROM all_slashes
    ORDER BY block_number DESC
    LIMIT $3";

// =============================================================================
// FORK DETECTION QUERIES
// =============================================================================

/// Get fork detection status for a chain
/// Detects when multiple writers have different blocks at the same height
/// $1 = chain_id
pub const SELECT_FORK_STATUS: &str = "
WITH writer_stats AS (
    SELECT
        COUNT(DISTINCT writer_id) as total_writers,
        COUNT(DISTINCT CASE WHEN synced THEN writer_id END) as synced_writers,
        MIN(chain_tip) as min_chain_tip
    FROM view_writer_status
    WHERE chain_id = $1
),
fork_heads AS (
    SELECT
        b.number as block_number,
        b.hash as block_hash,
        b.active_event_id as head_event_id,
        b.canonical,
        COUNT(DISTINCT ws.writer_id) as writer_count
    FROM blocks b
    JOIN writer_status ws ON ws.chain_id = b.chain_id
        AND b.number <= ws.chain_tip
        AND b.number >= ws.seen_block_from
    WHERE b.chain_id = $1
        AND b.number = (SELECT MAX(number) FROM blocks WHERE chain_id = $1 AND canonical = true)
    GROUP BY b.number, b.hash, b.active_event_id, b.canonical
)
SELECT
    $1::BIGINT as chain_id,
    COALESCE((SELECT MAX(number) FROM blocks WHERE chain_id = $1 AND canonical = true), 0) as highest_common_block,
    COALESCE((SELECT COUNT(*) > 1 FROM fork_heads), false) as forks_detected,
    COALESCE((SELECT total_writers FROM writer_stats), 0)::INT as total_writers,
    COALESCE((SELECT synced_writers FROM writer_stats), 0)::INT as synced_writers
";

/// Get fork head details for a specific block number
/// $1 = chain_id
/// $2 = block_number
pub const SELECT_FORK_HEADS: &str = "
SELECT
    b.number as block_number,
    b.hash as block_hash,
    b.active_event_id as head_event_id,
    b.canonical,
    COUNT(DISTINCT ws.writer_id) as writer_count
FROM blocks b
LEFT JOIN writer_status ws ON ws.chain_id = b.chain_id
    AND b.number <= ws.chain_tip
    AND b.number >= ws.seen_block_from
WHERE b.chain_id = $1
    AND b.number = $2
GROUP BY b.number, b.hash, b.active_event_id, b.canonical
ORDER BY b.canonical DESC, writer_count DESC
";

// =============================================================================
// COLLATERAL QUERIES
// =============================================================================

/// Get total collateral statistics for a chain
/// $1 = chain_id
pub const SELECT_TOTAL_COLLATERAL: &str = "
SELECT
    $1::BIGINT as chain_id,
    COALESCE(SUM(oc.collateral_wei_total::NUMERIC), 0) as total_collateral_wei,
    COUNT(*) as total_operators,
    COUNT(CASE WHEN COALESCE(oc.collateral_wei_total::NUMERIC, 0) > 0 THEN 1 END) as operators_with_collateral,
    COALESCE(AVG(CASE WHEN COALESCE(oc.collateral_wei_total::NUMERIC, 0) > 0 THEN oc.collateral_wei_total::NUMERIC END), 0) as average_collateral_wei
FROM operators o
LEFT JOIN blocks b ON b.hash = (SELECT e.block_hash FROM events e WHERE e.id = o.event_id)
    AND b.chain_id = o.chain_id
LEFT JOIN LATERAL (
    SELECT
        COALESCE(SUM(collateral_wei_delta::NUMERIC), 0) + COALESCE(MAX(CASE WHEN collateral_wei_total IS NOT NULL THEN collateral_wei_total::NUMERIC ELSE 0 END), 0) as collateral_wei_total
    FROM operator_collateral
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
) oc ON true
LEFT JOIN LATERAL (
    SELECT deleted
    FROM operator_data
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
    ORDER BY created_at DESC
    LIMIT 1
) od ON true
WHERE o.chain_id = $1
    AND COALESCE(od.deleted, false) = false
    AND COALESCE(b.canonical, true) = true
";

/// Get operators filtered by collateral range
/// $1 = chain_id
/// $2 = min_collateral (optional, use 0 for no minimum)
/// $3 = max_collateral (optional, use NULL for no maximum)
/// $4 = limit
pub const SELECT_OPERATORS_BY_COLLATERAL: &str = "
SELECT
    o.registration_root,
    o.chain_id::bigint as chain_id,
    o.owner_address,
    o.num_keys,
    o.registration_processed,
    e.block_number,
    oc.collateral_wei_total,
    od.event_type::text as last_event_type,
    od.deleted,
    od.equivocated,
    b.canonical,
    b.finalized,
    CASE
        WHEN b.finalized THEN 'finalized'
        WHEN b.canonical THEN 'canonical'
        ELSE 'pending'
    END as sync_status
FROM operators o
LEFT JOIN events e ON e.id = o.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
LEFT JOIN LATERAL (
    SELECT
        COALESCE(SUM(collateral_wei_delta::NUMERIC), 0) + COALESCE(MAX(CASE WHEN collateral_wei_total IS NOT NULL THEN collateral_wei_total::NUMERIC ELSE 0 END), 0) as collateral_wei_total
    FROM operator_collateral
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
) oc ON true
LEFT JOIN LATERAL (
    SELECT event_type, deleted, equivocated
    FROM operator_data
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
    ORDER BY created_at DESC
    LIMIT 1
) od ON true
WHERE o.chain_id = $1
    AND COALESCE(od.deleted, false) = false
    AND COALESCE(b.canonical, true) = true
    AND COALESCE(oc.collateral_wei_total, 0) >= $2::NUMERIC
    AND ($3::NUMERIC IS NULL OR COALESCE(oc.collateral_wei_total, 0) <= $3::NUMERIC)
ORDER BY COALESCE(oc.collateral_wei_total, 0) DESC, o.registration_root
LIMIT $4
";

/// Get top operators by collateral
/// $1 = chain_id
/// $2 = limit
pub const SELECT_TOP_OPERATORS_BY_COLLATERAL: &str = "
SELECT
    o.registration_root,
    o.chain_id::bigint as chain_id,
    o.owner_address,
    o.num_keys,
    o.registration_processed,
    e.block_number,
    oc.collateral_wei_total,
    od.event_type::text as last_event_type,
    od.deleted,
    od.equivocated,
    b.canonical,
    b.finalized,
    CASE
        WHEN b.finalized THEN 'finalized'
        WHEN b.canonical THEN 'canonical'
        ELSE 'pending'
    END as sync_status
FROM operators o
LEFT JOIN events e ON e.id = o.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
LEFT JOIN LATERAL (
    SELECT
        COALESCE(SUM(collateral_wei_delta::NUMERIC), 0) + COALESCE(MAX(CASE WHEN collateral_wei_total IS NOT NULL THEN collateral_wei_total::NUMERIC ELSE 0 END), 0) as collateral_wei_total
    FROM operator_collateral
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
) oc ON true
LEFT JOIN LATERAL (
    SELECT event_type, deleted, equivocated
    FROM operator_data
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
    ORDER BY created_at DESC
    LIMIT 1
) od ON true
WHERE o.chain_id = $1
    AND COALESCE(od.deleted, false) = false
    AND COALESCE(b.canonical, true) = true
    AND COALESCE(oc.collateral_wei_total, 0) > 0
ORDER BY COALESCE(oc.collateral_wei_total, 0) DESC, o.registration_root
LIMIT $2
";

// =============================================================================
// SLASHER QUERIES
// =============================================================================

/// List all slashers on a chain
/// $1 = chain_id
/// $2 = limit
pub const SELECT_SLASHERS: &str = "
SELECT
    s.address,
    s.chain_id,
    s.event_id,
    s.created_at
FROM slasher s
WHERE s.chain_id = $1
ORDER BY s.created_at DESC
LIMIT $2
";

/// Get operators committed to a specific slasher
/// $1 = chain_id
/// $2 = slasher_address (bytes)
/// $3 = limit
pub const SELECT_SLASHER_OPERATORS: &str = "
SELECT
    o.registration_root,
    o.chain_id::bigint as chain_id,
    o.owner_address,
    o.num_keys,
    o.registration_processed,
    e.block_number,
    oc.collateral_wei_total,
    od.event_type::text as last_event_type,
    od.deleted,
    od.equivocated,
    b.canonical,
    b.finalized,
    CASE
        WHEN b.finalized THEN 'finalized'
        WHEN b.canonical THEN 'canonical'
        ELSE 'pending'
    END as sync_status,
    -- Commitment fields
    osc.opted_in_at,
    osc.opted_out_at,
    osc.slashed,
    c.address as committer_address,
    CASE
        WHEN osc.opted_out_at IS NULL THEN 'active'
        ELSE 'opted_out'
    END as commitment_status
FROM operator_slasher_commitment osc
JOIN operators o ON o.registration_root = osc.registration_root AND o.chain_id = osc.chain_id
JOIN slasher s ON s.id = osc.slasher_id
LEFT JOIN committer c ON c.id = osc.committer_id
LEFT JOIN events e ON e.id = o.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
LEFT JOIN LATERAL (
    SELECT
        COALESCE(SUM(collateral_wei_delta::NUMERIC), 0) + COALESCE(MAX(CASE WHEN collateral_wei_total IS NOT NULL THEN collateral_wei_total::NUMERIC ELSE 0 END), 0) as collateral_wei_total
    FROM operator_collateral
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
) oc ON true
LEFT JOIN LATERAL (
    SELECT event_type, deleted, equivocated
    FROM operator_data
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
    ORDER BY created_at DESC
    LIMIT 1
) od ON true
WHERE osc.chain_id = $1
    AND s.address = $2
ORDER BY osc.opted_in_at DESC
LIMIT $3
";

/// Get slasher statistics
/// $1 = chain_id
/// $2 = slasher_address (bytes)
pub const SELECT_SLASHER_STATS: &str = "
WITH slasher_info AS (
    SELECT id, address, chain_id
    FROM slasher
    WHERE chain_id = $1 AND address = $2
),
operator_counts AS (
    SELECT
        COUNT(*) as total_operators,
        COUNT(CASE WHEN osc.opted_out_at IS NULL THEN 1 END) as active_operators
    FROM operator_slasher_commitment osc
    JOIN slasher_info si ON si.id = osc.slasher_id
    WHERE osc.chain_id = $1
),
slash_counts AS (
    SELECT
        COALESCE(COUNT(*), 0) as registration_slashes,
        COALESCE(SUM(sr.sender_reward_wei::NUMERIC), 0) as registration_rewards,
        COALESCE(SUM(sr.burned_wei::NUMERIC), 0) as registration_burned
    FROM slash_registration sr
    WHERE sr.chain_id = $1
),
commitment_counts AS (
    SELECT
        COALESCE(COUNT(*), 0) as commitment_slashes,
        COALESCE(SUM(sc.sender_reward_wei::NUMERIC), 0) as commitment_rewards,
        COALESCE(SUM(sc.burned_wei::NUMERIC), 0) as commitment_burned
    FROM slash_commitment sc
    WHERE sc.chain_id = $1
),
offchain_counts AS (
    SELECT
        COALESCE(COUNT(*), 0) as offchain_slashes,
        COALESCE(SUM(sodc.sender_reward_wei::NUMERIC), 0) as offchain_rewards,
        COALESCE(SUM(sodc.burned_wei::NUMERIC), 0) as offchain_burned
    FROM slash_offchain_delegation_commitment sodc
    WHERE sodc.chain_id = $1
),
equivocation_counts AS (
    SELECT
        COALESCE(COUNT(*), 0) as equivocation_slashes,
        COALESCE(SUM(se.sender_reward_wei::NUMERIC), 0) as equivocation_rewards,
        COALESCE(SUM(se.burned_wei::NUMERIC), 0) as equivocation_burned
    FROM slash_equivocation se
    WHERE se.chain_id = $1
)
SELECT
    si.address as slasher_address,
    si.chain_id,
    COALESCE(oc.total_operators, 0) as total_operators,
    COALESCE(oc.active_operators, 0) as active_operators,
    COALESCE(sc.registration_slashes, 0) + COALESCE(cc.commitment_slashes, 0) +
        COALESCE(ofc.offchain_slashes, 0) + COALESCE(ec.equivocation_slashes, 0) as total_slashes,
    COALESCE(sc.registration_slashes, 0) as registration_slashes,
    COALESCE(cc.commitment_slashes, 0) as commitment_slashes,
    COALESCE(ofc.offchain_slashes, 0) as offchain_slashes,
    COALESCE(ec.equivocation_slashes, 0) as equivocation_slashes,
    COALESCE(sc.registration_rewards, 0) + COALESCE(cc.commitment_rewards, 0) +
        COALESCE(ofc.offchain_rewards, 0) + COALESCE(ec.equivocation_rewards, 0) as total_rewards,
    COALESCE(sc.registration_burned, 0) + COALESCE(cc.commitment_burned, 0) +
        COALESCE(ofc.offchain_burned, 0) + COALESCE(ec.equivocation_burned, 0) as total_burned
FROM slasher_info si
LEFT JOIN operator_counts oc ON true
LEFT JOIN slash_counts sc ON true
LEFT JOIN commitment_counts cc ON true
LEFT JOIN offchain_counts ofc ON true
LEFT JOIN equivocation_counts ec ON true
";

/// Get all slashing events related to a slasher
/// $1 = chain_id
/// $2 = slasher_address (bytes)
/// $3 = limit
pub const SELECT_SLASHER_SLASHING_EVENTS: &str = "
WITH slasher_info AS (
    SELECT id FROM slasher WHERE chain_id = $1 AND address = $2
)
-- Registration slashes
SELECT
    'registration' as slash_type,
    sr.event_id,
    e.block_number,
    sr.registration_root,
    sr.sender_address,
    sr.sender_reward_wei,
    sr.burned_wei,
    b.timestamp,
    NULL::JSONB as details
FROM slash_registration sr
JOIN events e ON e.id = sr.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
WHERE sr.chain_id = $1

UNION ALL

-- Commitment slashes (filtered by slasher via signed_commitment)
SELECT
    'commitment' as slash_type,
    e.id as event_id,
    e.block_number,
    o.registration_root,
    sc.sender_address,
    sc.sender_reward_wei,
    sc.burned_wei,
    b.timestamp,
    jsonb_build_object('evidence', encode(sc.evidence, 'hex')) as details
FROM slash_commitment sc
JOIN operator_data od ON od.id = sc.operator_data_id
JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
JOIN events e ON e.id = od.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
LEFT JOIN signed_commitment scom ON scom.id = sc.signed_commitment_id
LEFT JOIN slasher_info si ON si.id IS NOT NULL
WHERE sc.chain_id = $1

UNION ALL

-- Offchain delegation slashes
SELECT
    'offchain_delegation' as slash_type,
    e.id as event_id,
    e.block_number,
    o.registration_root,
    sodc.sender_address,
    sodc.sender_reward_wei,
    sodc.burned_wei,
    b.timestamp,
    jsonb_build_object(
        'evidence', encode(sodc.evidence, 'hex'),
        'slashing_digest', encode(sodc.slashing_digest, 'hex')
    ) as details
FROM slash_offchain_delegation_commitment sodc
JOIN operator_data od ON od.id = sodc.operator_data_id
JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
JOIN events e ON e.id = od.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
WHERE sodc.chain_id = $1

UNION ALL

-- Equivocation slashes
SELECT
    'equivocation' as slash_type,
    e.id as event_id,
    e.block_number,
    o.registration_root,
    se.sender_address,
    se.sender_reward_wei,
    se.burned_wei,
    b.timestamp,
    NULL::JSONB as details
FROM slash_equivocation se
JOIN operator_data od ON od.id = se.operator_data_id
JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
JOIN events e ON e.id = od.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
WHERE se.chain_id = $1

ORDER BY block_number DESC
LIMIT $3
";

// =============================================================================
// COMMITTER QUERIES
// =============================================================================

/// List all committers on a chain
/// $1 = chain_id
/// $2 = limit
pub const SELECT_COMMITTERS: &str = "
SELECT
    c.address,
    c.chain_id,
    c.event_id,
    c.created_at
FROM committer c
WHERE c.chain_id = $1
ORDER BY c.created_at DESC
LIMIT $2
";

/// Get operators using a specific committer
/// $1 = chain_id
/// $2 = committer_address (bytes)
/// $3 = limit
pub const SELECT_COMMITTER_OPERATORS: &str = "
SELECT
    o.registration_root,
    o.chain_id::bigint as chain_id,
    o.owner_address,
    o.num_keys,
    o.registration_processed,
    e.block_number,
    oc.collateral_wei_total,
    od.event_type::text as last_event_type,
    od.deleted,
    od.equivocated,
    b.canonical,
    b.finalized,
    CASE
        WHEN b.finalized THEN 'finalized'
        WHEN b.canonical THEN 'canonical'
        ELSE 'pending'
    END as sync_status,
    -- Commitment fields
    osc.opted_in_at,
    osc.opted_out_at,
    osc.slashed,
    s.address as slasher_address,
    CASE
        WHEN osc.opted_out_at IS NULL THEN 'active'
        ELSE 'opted_out'
    END as commitment_status
FROM operator_slasher_commitment osc
JOIN operators o ON o.registration_root = osc.registration_root AND o.chain_id = osc.chain_id
JOIN committer c ON c.id = osc.committer_id
LEFT JOIN slasher s ON s.id = osc.slasher_id
LEFT JOIN events e ON e.id = o.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
LEFT JOIN LATERAL (
    SELECT
        COALESCE(SUM(collateral_wei_delta::NUMERIC), 0) + COALESCE(MAX(CASE WHEN collateral_wei_total IS NOT NULL THEN collateral_wei_total::NUMERIC ELSE 0 END), 0) as collateral_wei_total
    FROM operator_collateral
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
) oc ON true
LEFT JOIN LATERAL (
    SELECT event_type, deleted, equivocated
    FROM operator_data
    WHERE registration_root = o.registration_root
        AND chain_id = o.chain_id
    ORDER BY created_at DESC
    LIMIT 1
) od ON true
WHERE osc.chain_id = $1
    AND c.address = $2
ORDER BY osc.opted_in_at DESC
LIMIT $3
";

/// Get slashing events initiated by a committer
/// $1 = chain_id
/// $2 = committer_address (bytes)
/// $3 = limit
pub const SELECT_COMMITTER_SLASHING_EVENTS: &str = "
WITH committer_info AS (
    SELECT id, address FROM committer WHERE chain_id = $1 AND address = $2
)
-- Commitment slashes
SELECT
    'commitment' as slash_type,
    e.id as event_id,
    e.block_number,
    o.registration_root,
    sc.sender_address,
    ci.address as committer_address,
    sc.sender_reward_wei,
    sc.burned_wei,
    b.timestamp,
    jsonb_build_object('evidence', encode(sc.evidence, 'hex')) as details
FROM slash_commitment sc
JOIN committer_info ci ON ci.id = sc.committer_id
JOIN operator_data od ON od.id = sc.operator_data_id
JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
JOIN events e ON e.id = od.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
WHERE sc.chain_id = $1

UNION ALL

-- Offchain delegation slashes
SELECT
    'offchain_delegation' as slash_type,
    e.id as event_id,
    e.block_number,
    o.registration_root,
    sodc.sender_address,
    ci.address as committer_address,
    sodc.sender_reward_wei,
    sodc.burned_wei,
    b.timestamp,
    jsonb_build_object(
        'evidence', encode(sodc.evidence, 'hex'),
        'slashing_digest', encode(sodc.slashing_digest, 'hex')
    ) as details
FROM slash_offchain_delegation_commitment sodc
JOIN committer_info ci ON ci.id = sodc.committer_id
JOIN operator_data od ON od.id = sodc.operator_data_id
JOIN operators o ON o.registration_root = od.registration_root AND o.chain_id = od.chain_id
JOIN events e ON e.id = od.event_id
LEFT JOIN blocks b ON b.hash = e.block_hash AND b.chain_id = e.chain_id
WHERE sodc.chain_id = $1

ORDER BY block_number DESC
LIMIT $3
";