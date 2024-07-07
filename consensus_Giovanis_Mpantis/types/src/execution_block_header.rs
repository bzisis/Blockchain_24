// Copyright (c) 2022 Reth Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use crate::{Address, EthSpec, ExecutionPayloadRef, Hash256, Hash64, Uint256};
use metastruct::metastruct;

/// Execution block header used for RLP encoding and Keccak hashing.
///
/// This struct represents the header of an execution block. Fields are defined based on
/// specifications like EIP-3675 and EIP-4399.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[metastruct(mappings(map_execution_block_header_fields_base(exclude(
    withdrawals_root,
    blob_gas_used,
    excess_blob_gas,
    parent_beacon_block_root
)),))]
pub struct ExecutionBlockHeader {
    /// Parent block hash.
    pub parent_hash: Hash256,

    /// Hash of the ommers (uncles) list.
    pub ommers_hash: Hash256,

    /// Address that receives rewards for mining the block.
    pub beneficiary: Address,

    /// Root of the state trie after executing transactions.
    pub state_root: Hash256,

    /// Root hash of the transaction trie.
    pub transactions_root: Hash256,

    /// Root hash of the receipts trie.
    pub receipts_root: Hash256,

    /// Bloom filter for logs emitted by transactions.
    pub logs_bloom: Vec<u8>,

    /// Difficulty of the block.
    pub difficulty: Uint256,

    /// Block number.
    pub number: Uint256,

    /// Gas limit for the block.
    pub gas_limit: Uint256,

    /// Gas used by transactions in the block.
    pub gas_used: Uint256,

    /// Unix timestamp of the block.
    pub timestamp: u64,

    /// Extra data associated with the block.
    pub extra_data: Vec<u8>,

    /// Hash used for validator shuffling (EIP-4399).
    pub mix_hash: Hash256,

    /// Random value used for mining (nonce).
    pub nonce: Hash64,

    /// Base fee per gas (EIP-1559).
    pub base_fee_per_gas: Uint256,

    /// Root of withdrawals trie (optional).
    pub withdrawals_root: Option<Hash256>,

    /// Total gas used by the block blob (optional).
    pub blob_gas_used: Option<u64>,

    /// Excess gas beyond the block blob (optional).
    pub excess_blob_gas: Option<u64>,

    /// Root of the parent beacon block (optional).
    pub parent_beacon_block_root: Option<Hash256>,
}

impl ExecutionBlockHeader {
    /// Constructs an `ExecutionBlockHeader` from an `ExecutionPayloadRef`.
    ///
    /// This method maps fields from the `ExecutionPayloadRef` to construct an execution block header.
    /// - `payload`: The execution payload containing block data.
    /// - `rlp_empty_list_root`: Root hash of an empty RLP list.
    /// - `rlp_transactions_root`: Root hash of the transaction RLP.
    /// - `rlp_withdrawals_root`: Root hash of the withdrawals RLP (optional).
    /// - `rlp_blob_gas_used`: Total gas used by the block blob (optional).
    /// - `rlp_excess_blob_gas`: Excess gas beyond the block blob (optional).
    /// - `rlp_parent_beacon_block_root`: Root hash of the parent beacon block (optional).
    pub fn from_payload<E: EthSpec>(
        payload: ExecutionPayloadRef<E>,
        rlp_empty_list_root: Hash256,
        rlp_transactions_root: Hash256,
        rlp_withdrawals_root: Option<Hash256>,
        rlp_blob_gas_used: Option<u64>,
        rlp_excess_blob_gas: Option<u64>,
        rlp_parent_beacon_block_root: Option<Hash256>,
    ) -> Self {
        ExecutionBlockHeader {
            parent_hash: payload.parent_hash().into_root(),
            ommers_hash: rlp_empty_list_root,
            beneficiary: payload.fee_recipient(),
            state_root: payload.state_root(),
            transactions_root: rlp_transactions_root,
            receipts_root: payload.receipts_root(),
            logs_bloom: payload.logs_bloom().clone().into(),
            difficulty: Uint256::zero(),
            number: payload.block_number().into(),
            gas_limit: payload.gas_limit().into(),
            gas_used: payload.gas_used().into(),
            timestamp: payload.timestamp(),
            extra_data: payload.extra_data().clone().into(),
            mix_hash: payload.prev_randao(),
            nonce: Hash64::zero(),
            base_fee_per_gas: payload.base_fee_per_gas(),
            withdrawals_root: rlp_withdrawals_root,
            blob_gas_used: rlp_blob_gas_used,
            excess_blob_gas: rlp_excess_blob_gas,
            parent_beacon_block_root: rlp_parent_beacon_block_root,
        }
    }
}
