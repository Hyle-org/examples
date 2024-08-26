fn main(
    version: pub u32,
    initial_state_len: pub u32,
    initial_state: pub [u8; 4],
    next_state_len: pub u32,
    next_state: pub [u8; 4],
    identity_len: pub u8,
    identity: pub str<0>,
    tx_hash_len: pub u32,
    tx_hash: pub [u8; 4],
    program_outputs: pub str<17>
) {
    let collatz_response = compute_collatz(initial_state);
    if (initial_state == [0, 0, 0, 1]){
        assert(true);
    }
    else {
        assert(next_state == collatz_response);
    }
    assert(version == 1);
    assert(initial_state_len == 4);
    assert(next_state_len == 4);
    assert(identity_len == 0);
    assert(identity == "");
    assert(tx_hash_len == 4);
    assert(tx_hash == [0, 0, 0, 0]);
    assert(program_outputs == "Any output heehee");
}

fn compute_collatz(initial_state: [u8; 4]) -> [u8; 4] {
    let mut next_state_num: u32 = 0;
    let mut initial_state_num: u32  = from_be_bytes(initial_state); 
    if (initial_state_num%2 == 0) {
		next_state_num = initial_state_num / 2;
	} else {
		next_state_num = initial_state_num*3 + 1;
	}
    to_be_bytes(next_state_num)
}


fn from_be_bytes(bytes: [u8; 4]) -> u32{
    let num = ((bytes[0] as u32) << 24) |
    ((bytes[1] as u32) << 16) |
    ((bytes[2] as u32) << 8) |
    (bytes[3] as u32);

    num
}

fn to_be_bytes(num: u32) -> [u8; 4] {
    let bytes = [
        ((num >> 24) & 0xff) as u8,
        ((num >> 16) & 0xff) as u8,
        ((num >> 8) & 0xff) as u8,
        (num & 0xff) as u8,
    ];
    bytes
}