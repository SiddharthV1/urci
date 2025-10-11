use alloy_primitives::{Address, BlockHash, BlockNumber, FixedBytes, B256};

pub type WorkId = B256;
pub type BlockNumberWithHash = (BlockNumber, BlockHash);
pub type Owner = Address;
pub type Slasher = Address;
pub type Commiter = Address;
pub type RegistrationRoot = FixedBytes<32>;
pub type SlashAmountWei = alloy_primitives::U256;
