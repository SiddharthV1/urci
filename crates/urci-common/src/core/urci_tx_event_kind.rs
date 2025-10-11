//! Transaction-level URCI event kind - wraps UrciTxEvent with trace requirements

use alloy_consensus::{
    transaction::SignerRecoverable, EthereumTxEnvelope, Transaction, TxEip4844, Typed2718,
};
use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

use super::errors::SystemError;
use super::urci_event::UrciEvent;
use super::urci_event_kind::UrciEventKind;
use super::urci_tx_event::{UrciIndexedLogs, UrciTxEvent};
use crate::Verifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrciTxEventKind {
    UrciEvent(UrciTxEvent),
    TraceRequest(UrciTxEvent),
}

impl UrciTxEventKind {
    // Create UrciTxEventKind from transaction data
    // Returns SystemError if calldata decode or EVM execution fails (should flip indexer to failed state)
    pub async fn new(
        urc_address: &Address,
        tx: &EthereumTxEnvelope<TxEip4844>,
        transaction_index: u64,
        logs: UrciIndexedLogs,
        verifier: &impl Verifier,
    ) -> Result<Self, SystemError> {
        let to = tx.to();
        let sender = SignerRecoverable::recover_signer(tx).unwrap();
        let value = tx.value();
        let input = tx.input();
        let transaction_hash = *tx.hash();

        let mut needs_trace = true;
        let mut urc_events = Vec::new();

        // Special case: single log from direct call to URC - no trace needed
        if to == Some(*urc_address) && logs.len() == 1 {
            needs_trace = false;

            // Extract the single log and its index
            let (log, log_index) = &logs[0];

            let event_kind = UrciEventKind::from_log_and_input(log, input, verifier)
                .await?
                .ok_or_else(|| {
                    SystemError::EventParseFailed("Event decode returned None".to_string())
                })?;
            // Create UrciEvent with direct call info (no trace needed)
            let urc_event = UrciEvent {
                caller_address: sender,     // Direct caller is tx sender
                call_depth: 0,              // Direct call
                call_index_at_depth: 0,     // First call
                input: Some(input.clone()), // Include tx input
                log_index: *log_index,
                event: event_kind,
            };

            urc_events.push(urc_event);

            Ok(UrciTxEventKind::UrciEvent(UrciTxEvent {
                tx_sender_address: sender,
                tx_to_address: to,
                tx_value: value,
                tx_type: tx.ty() as u64,
                tx_gas: tx.gas_limit(),
                tx_gas_price: tx.max_fee_per_gas(),
                tx_nonce: tx.nonce(),
                transaction_hash,
                transaction_index,
                transaction_input: if input.is_empty() {
                    None
                } else {
                    Some(input.clone())
                },
                urc_logs: logs,
                urc_events,
                trace: Vec::new(),
                tracer: needs_trace,
            }))
        } else {
            Ok(UrciTxEventKind::TraceRequest(UrciTxEvent {
                tx_sender_address: sender,
                tx_to_address: to,
                tx_value: value,
                tx_type: tx.ty() as u64,
                tx_gas: tx.gas_limit(),
                tx_gas_price: tx.max_fee_per_gas(),
                tx_nonce: tx.nonce(),
                transaction_hash,
                transaction_index,
                transaction_input: if input.is_empty() {
                    None
                } else {
                    Some(input.clone())
                },
                urc_logs: logs,
                urc_events,
                trace: Vec::new(),
                tracer: needs_trace,
            }))
        }
    }

    pub fn needs_trace(&self) -> bool {
        matches!(self, UrciTxEventKind::TraceRequest(_))
    }

    pub fn set_trace(&mut self, trace: Vec<super::urci_event::CallTrace>) {
        if let UrciTxEventKind::TraceRequest(mut tx_event) =
            std::mem::replace(self, UrciTxEventKind::UrciEvent(UrciTxEvent::default()))
        {
            tx_event.trace = trace;
            tx_event.tracer = false; // Trace has been set, no longer needs tracing
            *self = UrciTxEventKind::UrciEvent(tx_event);
        }
    }
}
