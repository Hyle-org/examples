use dep::std;
use dep::std::println;

struct ProgramOutputs {
    nth_fib: u64,
    expected_result: u64
}

fn main(
    version: pub u32,
    initial_state_len: pub u32,
    initial_state: pub [u8; 4],
    next_state_len: pub u32,
    next_state: pub [u8; 4],
    identity_len: pub u8,
    identity: pub str<0>,
    tx_hash_len: pub u32,
    tx_hash: pub [u8; 43],
    payload_hash: pub Field,
    success: pub bool,
    program_outputs: pub ProgramOutputs
) {
    let result = fib(program_outputs.nth_fib);
    assert(result == program_outputs.expected_result);
}

unconstrained fn fib(mut n: u64) -> u64 {
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    for i in 0..n {
        let temp = b;
        b = a + b;
        a = temp;
    }
    a
}

#[test]
fn test_main_ok() {
    let version = 1;
    let initial_state_len = 4;
    let initial_state = [0, 0, 0, 1];
    let next_state_len = 4;
    let next_state = [0, 0, 0, 0];
    let identity_len = 0;
    let identity = "";
    let tx_hash_len = 43;
    let tx_hash = [
        77, 68, 69, 121, 77, 122, 81, 49, 78, 106, 99, 52, 79, 87, 70, 105, 89, 50, 82, 108, 90, 106, 65, 120, 77, 106, 77, 48, 78, 84, 89, 51, 79, 68, 108, 104, 89, 109, 78, 107, 90, 87, 89
    ];
    let payload_hash = 0x2cd4a07375bb18c67225f96dc94a459803676221c55d76cfbb28238e5279c86d as Field;
    let success = true;
    let program_outputs = ProgramOutputs { nth_fib: 9, expected_result: 34 };
    main(
        version,
        initial_state_len,
        initial_state,
        next_state_len,
        next_state,
        identity_len,
        identity,
        tx_hash_len,
        tx_hash,
        payload_hash,
        success,
        program_outputs
    );
}

#[test(should_fail)]
fn test_main_ko() {
    let version = 1;
    let initial_state_len = 4;
    let initial_state = [0, 0, 0, 1];
    let next_state_len = 4;
    let next_state = [0, 0, 0, 0];
    let identity_len = 0;
    let identity = "";
    let tx_hash_len = 43;
    let tx_hash = [
        77, 68, 69, 121, 77, 122, 81, 49, 78, 106, 99, 52, 79, 87, 70, 105, 89, 50, 82, 108, 90, 106, 65, 120, 77, 106, 77, 48, 78, 84, 89, 51, 79, 68, 108, 104, 89, 109, 78, 107, 90, 87, 89
    ];
    let payload_hash = 0x2cd4a07375bb18c67225f96dc94a459803676221c55d76cfbb28238e5279c86d as Field;
    let success = true;
    let program_outputs = ProgramOutputs { nth_fib: 10, expected_result: 56 };
    main(
        version,
        initial_state_len,
        initial_state,
        next_state_len,
        next_state,
        identity_len,
        identity,
        tx_hash_len,
        tx_hash,
        payload_hash,
        success,
        program_outputs
    );
}

