use core::clone::Clone;
use core::traits::Into;
use core::option::OptionTrait;
use core::traits::TryInto;
use core::array::ArrayTrait;
use core::serde::Serde;
use core::pedersen::PedersenTrait;
use core::hash::{HashStateTrait, HashStateExTrait};
use starknet::{
    EthAddress,
    secp256_trait::{
        Secp256Trait, Secp256PointTrait, recover_public_key, is_signature_entry_valid, Signature
    },
    secp256k1::Secp256k1Point, SyscallResult, SyscallResultTrait
};
use core::keccak::keccak_u256s_be_inputs;

#[derive(Serde, Drop, Clone, Debug)]
struct HyleOutput {
    version: u32,
    initial_state: felt252,
    next_state: felt252,
    identity: ByteArray,
    tx_hash: felt252,
    payload_hash: felt252,
    success: bool,
    program_outputs: Array<felt252>
}

fn main(input: Array<felt252>) -> Array<felt252> {
    let mut input = input.span();
    
    // Deserialize input parameters
    let msg_hash: felt252 = Serde::deserialize(ref input).unwrap();
    let signature_r: felt252 = Serde::deserialize(ref input).unwrap();
    let signature_s: felt252 = Serde::deserialize(ref input).unwrap();
    let eth_address: felt252 = Serde::deserialize(ref input).unwrap();

    // Construct the Signature object
    let signature = Signature {
        r: signature_r,
        s: signature_s
    };

    // Initial state compute (could be a previous tx hash, state hash, etc.)
    let initial_state = 1;

    // Verify Ethereum Signature
    let result = is_eth_signature_valid(msg_hash, signature, eth_address.into());

    // Next state compute (could be updated based on the result)
    let next_state = match result {
        Result::Ok(_) => 1,
        Result::Err(_) => 0,
    };

    // Identity - here, just an example as empty, but in real use, would be the caller's identity
    let identity: ByteArray = "";

    // Success flag based on the signature verification
    let success = result.is_ok();

    // Create payload for the output
    let mut payload = array![msg_hash, signature_r, signature_s, eth_address];

    processHyleOutput(1, initial_state.clone(), next_state.clone(), identity, 0, payload, success, array![next_state.into()])
}

/// Asserts that an Ethereum signature is valid w.r.t. a given Eth address.
/// Returns a Result with an error string if the signature is invalid.
pub fn is_eth_signature_valid(
    msg_hash: felt252, signature: Signature, eth_address: EthAddress
) -> Result<(), felt252> {
    if !is_signature_entry_valid::<Secp256k1Point>(signature.r) {
        return Result::Err('Signature r value out of range');
    }
    if !is_signature_entry_valid::<Secp256k1Point>(signature.s) {
        return Result::Err('Signature s value out of range');
    }

    let public_key_point = recover_public_key::<Secp256k1Point>(msg_hash, signature).unwrap();
    let calculated_eth_address = public_key_point_to_eth_address(public_key_point);

    if eth_address != calculated_eth_address {
        return Result::Err('Invalid signature');
    }
    Result::Ok(())
}

/// Converts a public key point to the corresponding Ethereum address.
pub fn public_key_point_to_eth_address(
    public_key_point: Secp256k1Point
) -> EthAddress {
    let (x, y) = public_key_point.get_coordinates().unwrap_syscall();

    // Keccak output is little endian.
    let point_hash_le = keccak_u256s_be_inputs([x, y].span());
    let point_hash = felt252 {
        low: core::integer::u128_byte_reverse(point_hash_le.high),
        high: core::integer::u128_byte_reverse(point_hash_le.low)
    };

    point_hash.into()
}

fn processHyleOutput(
    version: u32,
    initial_state: felt252,
    next_state: felt252,
    identity: ByteArray,
    tx_hash: felt252,
    payload: Array<felt252>,
    success: bool,
    program_output: Array<felt252>
) -> Array<felt252> {
    // Hashing payload
    let payload_span = payload.span();
    let payload_hash = compute_hash_on_elements(payload_span);

    // HyleOutput
    let hyle_output = HyleOutput {
        version: version,
        initial_state: initial_state,
        next_state: next_state,
        identity: identity,
        tx_hash: tx_hash,
        payload_hash: payload_hash,
        success: success,
        program_outputs: program_output,
    };

    let mut output = array![];
    hyle_output.serialize(ref output);
    output
}

/// Creates a Pedersen hash chain with the elements of `data` and returns the finalized hash.
fn compute_hash_on_elements(mut data: Span<felt252>) -> felt252 {
    let data_len = data.len();
    let mut state = PedersenTrait::new(0);
    let mut hash = 0;
    loop {
        match data.pop_front() {
            Option::Some(elem) => { state = state.update_with(*elem); },
            Option::None => {
                hash = state.update_with(data_len).finalize();
                break;
            },
        };
    };
    hash
}

