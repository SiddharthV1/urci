//! Inspector trait implementation - all EVM hooks in one place

use alloy_primitives::{Address, Log};
use revm::context_interface::ContextTr;
use revm::inspector::Inspector;
use revm::interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CallScheme, CreateInputs, CreateOutcome,
    Interpreter,
};

use super::inspector::UrciInspector;
use super::call_frame::CallFrame;
use crate::inspectors::types::{CallEdge, CallStatus, CallType};

impl<CTX> Inspector<CTX, EthInterpreter> for UrciInspector
where
    CTX: ContextTr,
{
    fn call(&mut self, context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        // Increment depth counter
        if self.depth_counters.len() <= self.call_stack.len() {
            self.depth_counters.push(0);
        }
        self.depth_counters[self.call_stack.len()] += 1;

        // Create call edge
        let trace_address = self.current_trace_address();

        let input_data = inputs.input.bytes(context);
        let input_len = input_data.len();

        let edge = CallEdge {
            id: None,
            block_number: self.block_number,
            block_hash: self.block_hash,
            tx_hash: self.tx_hash,
            tx_index: self.tx_index,
            trace_address: trace_address.clone(),
            depth: self.call_stack.len() as u32,
            call_type: CallType::from(inputs.scheme),
            from_addr: inputs.caller,
            to_addr: inputs.target_address,
            code_addr: if inputs.scheme == CallScheme::DelegateCall {
                Some(inputs.bytecode_address)
            } else {
                None
            },
            value_wei: inputs.value.get(),
            gas_in: inputs.gas_limit,
            gas_used: 0,                 // Will be updated in call_end
            status: CallStatus::Success, // Will be updated in call_end
            selector: Self::extract_selector(&input_data),
            input_data: if input_len > 0 {
                Some(input_data)
            } else {
                None
            },
            output_data: None, // Will be updated in call_end
            input_len,
            output_len: 0,   // Will be updated in call_end
            error_msg: None, // Will be updated in call_end
        };

        let edge_index = self.edges.len();
        self.edges.push(edge);

        // Push frame for call graph tracking
        self.call_stack.push(CallFrame {
            edge_index,
            trace_address,
            gas_in: inputs.gas_limit,
        });

        // If this is a call to the Registry, store the input for later use when log is emitted
        if inputs.target_address == self.registry_address {
            let input_bytes = inputs.input.bytes(context);
            self.pending_registry_call_input = Some(input_bytes);
        }

        None // Don't override the call
    }

    fn call_end(&mut self, _context: &mut CTX, _inputs: &CallInputs, outcome: &mut CallOutcome) {
        if let Some(frame) = self.call_stack.pop() {
            // Update edge with results
            if let Some(edge) = self.edges.get_mut(frame.edge_index) {
                edge.gas_used = frame.gas_in.saturating_sub(outcome.result.gas.remaining());

                let output = outcome.result.output.clone();
                edge.output_len = output.len();

                if self.store_full_data && output.len() <= self.max_data_size {
                    edge.output_data = Some(output.clone());
                }

                edge.status = if outcome.result.result.is_ok() {
                    CallStatus::Success
                } else {
                    CallStatus::Revert
                };

                // Try to decode revert reason
                if !outcome.result.result.is_ok()
                    && output.len() >= 68
                    && output[0..4] == [0x08, 0xc3, 0x79, 0xa0]
                {
                    // Standard Error(string) revert
                    if let Ok(reason) = std::str::from_utf8(&output[68..]) {
                        edge.error_msg = Some(reason.trim_end_matches('\0').to_string());
                    }
                }
            }
        }

        // Pop depth counter if needed
        if self.call_stack.len() < self.depth_counters.len() {
            self.depth_counters.truncate(self.call_stack.len() + 1);
        }
    }

    fn create(&mut self, _context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        // Capture contract creation in call graph
        if self.depth_counters.len() <= self.call_stack.len() {
            self.depth_counters.push(0);
        }
        self.depth_counters[self.call_stack.len()] += 1;

        let trace_address = self.current_trace_address();

        let edge = CallEdge {
            id: None,
            block_number: self.block_number,
            block_hash: self.block_hash,
            tx_hash: self.tx_hash,
            tx_index: self.tx_index,
            trace_address: trace_address.clone(),
            depth: self.call_stack.len() as u32,
            call_type: if inputs.scheme == revm::interpreter::CreateScheme::Create {
                CallType::Create
            } else {
                CallType::Create2
            },
            from_addr: inputs.caller,
            to_addr: Address::ZERO, // Will be updated in create_end with actual address
            code_addr: None,
            value_wei: inputs.value,
            gas_in: inputs.gas_limit,
            gas_used: 0,
            status: CallStatus::Success,
            selector: None,
            input_data: if self.store_full_data && inputs.init_code.len() <= self.max_data_size {
                Some(inputs.init_code.clone())
            } else {
                None
            },
            output_data: None,
            input_len: inputs.init_code.len(),
            output_len: 0,
            error_msg: None,
        };

        let edge_index = self.edges.len();
        self.edges.push(edge);

        self.call_stack.push(CallFrame {
            edge_index,
            trace_address,
            gas_in: inputs.gas_limit,
        });

        None
    }

    fn create_end(
        &mut self,
        _context: &mut CTX,
        _inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        if let Some(frame) = self.call_stack.pop() {
            if let Some(edge) = self.edges.get_mut(frame.edge_index) {
                edge.gas_used = frame.gas_in.saturating_sub(outcome.result.gas.remaining());

                let output = outcome.result.output.clone();
                edge.output_len = output.len();
                if self.store_full_data && output.len() <= self.max_data_size {
                    edge.output_data = Some(output.clone());
                }

                edge.status = if outcome.result.result.is_ok() {
                    CallStatus::Success
                } else {
                    CallStatus::Revert
                };
            }
        }

        if self.call_stack.len() < self.depth_counters.len() {
            self.depth_counters.truncate(self.call_stack.len() + 1);
        }
    }

    /// Called when a log is emitted
    /// This is where we check if a valid URC event was emitted
    fn log(&mut self, _interp: &mut Interpreter<EthInterpreter>, _context: &mut CTX, log: Log) {
        // Track the call depth and index for this log
        // The log is emitted by the currently active call (top of call stack)
        if let Some(_frame) = self.call_stack.last() {
            let call_depth = self.call_stack.len() as u32 - 1; // 0-indexed depth
            let call_index_at_depth = self.depth_counters[call_depth as usize] - 1; // We already incremented it

            // Store mapping: log_index -> (call_depth, call_index)
            // Note: log_index will be determined later from receipt, for now we track all logs
            let log_index = self.log_to_call_map.len() as u64; // Temporary index
            self.log_to_call_map
                .push((log_index, call_depth, call_index_at_depth));
        }

        // Check if this log is from the Registry contract
        if log.address == self.registry_address {
            self.process_registry_log(&log);
        }
    }
}
