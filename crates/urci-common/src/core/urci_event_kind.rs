//! URCI event kinds extracted from logs

use alloy_primitives::{Bytes, Log};
use alloy_sol_types::{SolCall, SolEventInterface};
use serde::{Deserialize, Serialize};

use super::errors::SystemError;
use super::slashing::SlashingCall;
use super::types::{Owner, SlashAmountWei};
use super::validation_result::RegistrationValidationResult;

use crate::{
    CollateralAdded, CollateralClaimed, OperatorOptedIn, OperatorOptedOut, OperatorRegistered,
    OperatorSlashed, OperatorUnregistered, RegistryEvents, Verifier,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrciEventKind {
    Registration(
        Owner,
        OperatorRegistered,
        std::result::Result<RegistrationValidationResult, String>,
    ),
    Unregistration(OperatorUnregistered),
    OptIn(OperatorOptedIn),
    OptOut(OperatorOptedOut),
    CollateralClamed(CollateralClaimed),
    CollateralAdded(CollateralAdded),
    Slashing(OperatorSlashed, SlashAmountWei, Box<SlashingCall>),
}

impl UrciEventKind {
    // Extract UrciEventKind from call input and log
    // Returns SystemError if there's a fatal error (decode failure, EVM issue)
    // Returns Ok with validation_error field if BLS validation fails (indexable)
    pub async fn from_log_and_input(
        log: &Log,
        call_input: &Bytes,
        verifier: &impl Verifier,
    ) -> Result<Option<Self>, SystemError> {
        // Decode the event from log
        let decoded_event =
            RegistryEvents::decode_raw_log(log.topics(), &log.data.data).map_err(|e| {
                SystemError::EventParseFailed(format!("Failed to decode registry event: {}", e))
            })?;

        // Create UrciEventKind based on event type
        let event_kind = match decoded_event {
            RegistryEvents::OperatorRegistered(data) => {
                // Try to decode calldata - this is a SYSTEM ERROR if it fails
                let register_call = crate::registerCall::abi_decode(call_input).map_err(|e| {
                    SystemError::CalldataDecodeFailed(format!(
                        "Failed to decode register() calldata: {}",
                        e
                    ))
                })?;

                // For registration, verify BLS signatures using the verifier (async)
                // EVM execution errors are SYSTEM ERRORS (propagate up)
                // Validation failures (fraud, invalid sigs) are stored in validation_error field
                let validation_result = match verifier
                    .verify_registration(register_call.registrations, register_call.owner)
                    .await
                {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        let err_str = e.to_string();
                        // Check if this is an EVM execution error (SYSTEM ERROR)
                        if err_str.contains("unlinked")
                            || err_str.contains("execution")
                            || err_str.contains("out of gas")
                            || err_str.contains("revert")
                        {
                            return Err(SystemError::EvmExecutionFailed(err_str));
                        }
                        // Otherwise it's a validation issue - store in validation_error field
                        Err(err_str)
                    }
                };

                UrciEventKind::Registration(data.owner, data, validation_result)
            }
            RegistryEvents::OperatorUnregistered(data) => UrciEventKind::Unregistration(data),
            RegistryEvents::OperatorOptedIn(data) => UrciEventKind::OptIn(data),
            RegistryEvents::OperatorOptedOut(data) => UrciEventKind::OptOut(data),
            RegistryEvents::CollateralClaimed(data) => UrciEventKind::CollateralClamed(data),
            RegistryEvents::CollateralAdded(data) => UrciEventKind::CollateralAdded(data),
            RegistryEvents::OperatorSlashed(data) => {
                // Decode the slashing call type from input by checking function selector
                if call_input.is_empty() || call_input.len() < 4 {
                    return Err(SystemError::CalldataDecodeFailed(
                        "OperatorSlashed event without transaction input".to_string(),
                    ));
                }

                let selector = &call_input[0..4];

                // Try each slashing type based on selector
                let slashing_call = if selector == crate::slashRegistrationCall::SELECTOR.as_slice()
                {
                    let call =
                        crate::slashRegistrationCall::abi_decode(call_input).map_err(|e| {
                            SystemError::CalldataDecodeFailed(format!(
                                "Failed to decode slashRegistration call: {}",
                                e
                            ))
                        })?;
                    SlashingCall::Fraud {
                        registration_proof: Box::new(call.proof),
                    }
                } else if selector == crate::slashEquivocationCall::SELECTOR.as_slice() {
                    let call =
                        crate::slashEquivocationCall::abi_decode(call_input).map_err(|e| {
                            SystemError::CalldataDecodeFailed(format!(
                                "Failed to decode slashEquivocation call: {}",
                                e
                            ))
                        })?;
                    SlashingCall::Equivocation {
                        registration_proof: Box::new(call.proof),
                        delegation_one: Box::new(call.delegationOne),
                        delegation_two: Box::new(call.delegationTwo),
                    }
                } else if selector == crate::slashCommitment_1Call::SELECTOR.as_slice() {
                    let call =
                        crate::slashCommitment_1Call::abi_decode(call_input).map_err(|e| {
                            SystemError::CalldataDecodeFailed(format!(
                                "Failed to decode slashCommitment call: {}",
                                e
                            ))
                        })?;
                    SlashingCall::Commitment {
                        registration_proof: Box::new(call.proof),
                        delegation: Box::new(call.delegation),
                        commitment: Box::new(call.commitment),
                    }
                } else if selector == crate::slashCommitment_0Call::SELECTOR.as_slice() {
                    let call =
                        crate::slashCommitment_0Call::abi_decode(call_input).map_err(|e| {
                            SystemError::CalldataDecodeFailed(format!(
                                "Failed to decode slashCommitment call: {}",
                                e
                            ))
                        })?;
                    SlashingCall::SlasherCommitment {
                        registration_root: call.registrationRoot,
                        commitment: Box::new(call.commitment),
                        evidence: call.evidence.to_vec(),
                    }
                } else {
                    return Err(SystemError::CalldataDecodeFailed(format!(
                        "Unknown slashing selector: {:?}",
                        hex::encode(selector)
                    )));
                };

                let slash_amount = data.slashAmountWei;
                UrciEventKind::Slashing(data, slash_amount, Box::new(slashing_call))
            }
        };

        Ok(Some(event_kind))
    }
}
