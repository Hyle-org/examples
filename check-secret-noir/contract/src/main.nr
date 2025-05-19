// Simple Token Contract implementation in Noir

fn main(
    // The version of the HyleOutput. This is unchecked for now.
    version: pub u32,
    // The initial state of the contract. This is the state before the transaction is executed.
    initial_state_len: pub u32,
    initial_state: pub [u8; 4],
    // The state of the contract after the transaction is executed.
    next_state_len: pub u32,
    next_state: pub [u8; 4],
    // The identity used to execute the transaction. This is the same as the one used in the
    // BlobTransaction.
    identity_len: pub u8,
    identity: pub str<256>,
    // TxHash of the BlobTransaction.
    tx_hash: pub str<64>,
    // ------ Blobs ------
    // The index of the blob being executed.
    index: pub u32,
    // Number of blobs used by the contract
    // If blob_number is > 1, then you need multiple occurence of parameters group "Blob"
    blob_number: pub u32,
    // --- Blob
    // At least one blob_index must match the index parameter
    blob_index: pub u32,
    blob_contract_name_len: pub u8,
    blob_contract_name: pub str<256>,
    blob_capacity: pub u32,
    blob_len: pub u32,
    blob: pub [u8; 32],
    // ---
    // Number of blobs in the transaction. tx_blob_count >= blob_number
    tx_blob_count: pub u32,
    // -------------------
    // Weither the execution was successful or not. If false, the BlobTransaction will be
    // settled as failed.
    success: pub bool,
    // ------ Private inputs ------
    password: [u8; 32],
) {
    let mut id: BoundedVec<u8, 256> = BoundedVec::from_parts(identity.as_bytes(), identity_len as u32);
    id.extend_from_slice(":".as_bytes());
    id.extend_from_slice(password);

    println(f"Identity: {identity}");
    println(f"Password: {password}");
    println(f"extended id: {id}");

    // Hash the combined input using SHA-256
    let input_hash = sha256::sha256_var(id.storage(), identity_len as u64 + 1 + 32);
    println(f"Input Hash: {input_hash}");

    assert(input_hash == blob);

    assert(success == true);
    assert(version == 1);
    assert(initial_state_len == 4);
    assert(next_state_len == 4);
    assert(initial_state == [0; 4]);
    assert(next_state == [0; 4]);
    assert(blob_capacity == 32);
    assert(blob_len == 32);
    assert(blob_number == 1);
    assert(tx_blob_count >= 1);
}
