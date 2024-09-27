use core::clone::Clone;
use core::traits::Into;
use core::option::OptionTrait;
use core::traits::TryInto;
use core::array::ArrayTrait;
use core::serde::Serde;
use core::pedersen::PedersenTrait;
use core::hash::{HashStateTrait, HashStateExTrait};


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
    // get datas from args file
    let fibo_number: felt252 = Serde::deserialize(ref input).unwrap();
    // Initial state compute
    let initial_state = 1;
    //cal fib number
    let res = fib(fibo_number.into());
    // Next state compute
    let next_state = res;
    let success = true;
    let mut payload = array![fibo_number];
    let identity: ByteArray = "";
    processHyleOutput(1, initial_state.clone(), next_state.clone(), identity, 0, payload, success, array![res.into()])
}

fn fib(mut n: felt252) -> felt252 {
    let mut a: u32 = 0;
    let mut b: u32 = 1;
    while n != 0 {
        n = n - 1;
        let temp = b;
        b = a + b;
        a = temp;
    };
    a.into()
}


fn compute_state_pedersen_hash<T, +Serde<T>>(state: @T) -> felt252 {
    let mut serialiazed_state: Array<felt252> = ArrayTrait::new();
    state.serialize(ref serialiazed_state);

    compute_hash_on_elements(serialiazed_state.span())
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

#[cfg(test)]
mod tests {
    use super::fib;

    #[test]
    fn it_works() {
        let input = array![10 //fibo nth to calculate
        ];
        let output = super::main(input);

        assert!(
            output == array![
                1,
                1,
                55,
                0,
                0,
                0,
                0,
                3225999730476793779736563625099842789687658654485121285797139585172270104279,
                1,
                1,
                55
            ],
            "output mismatch"
        );
    }
}
