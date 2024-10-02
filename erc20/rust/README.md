# ERC20 Contract Example with RISC Zero

This project demonstrates an example smart contract for Hylé, implementing an ERC20 smart contract.

## Tooling

- Rust installation: https://www.rust-lang.org/tools/install
- Risc Zero installation: https://dev.risczero.com/api/zkvm/install


## Reproducible builds

To build the smart contract run :
```sh
./build.sh
```

### Proof Generation

To generate a proof of an ERC20 token transfer from one account to another, run:

- With reproducible ELF binary:

   ```sh
   cargo run -- -r mint bob 100
   ```

- Non-reproducibly:

   ```sh
   cargo run transfer bob alice 100
   ```


This will output:

```sh
Method ID: Digest(2a7db06061796667484e04092ae6db09afab019a0c4e25dc8c868498e6c08b31) (hex)
proof.json written, transition from "ed286b3c39b2f86f9ce86bbc35455fe7e8f7b3f9683ba76e0bcc637eb5602f3d" to "f5cbdc50df4fbea14ff33c28b496cff6021bf3d9977380bc116f4f5698e30b38"
HyleOutput { version: 1, initial_state: [237, ..., 61], next_state: [245, ..., 56], identity: "bob", tx_hash: [1], index: 0, payloads: [1, ..., 0], success: true, program_outputs: "Minted 100 to bob" }
```

Key information includes:
- **Method ID**: `2a7db06061796667484e04092ae6db09afab019a0c4e25dc8c868498e6c08b31`, which will be used later to register the contract on Hylé.
- **Initial State Transition**: `ed286b3c39b2f86f9ce86bbc35455fe7e8f7b3f9683ba76e0bcc637eb5602f3d`, which will be set when registering the contract on Hylé.
- **Next State Transition**: `f5cbdc50df4fbea14ff33c28b496cff6021bf3d9977380bc116f4f5698e30b38`, which will be visible on Hylé once the proof is validated.

### Proof Verification - Locally

Install the [Hylé RISC Zero verifier](https://github.com/Hyle-org/verifiers-for-hyle).

You can then verify proofs in **risc0-verifier/**, run:

```sh
cargo run -p risc0-verifier 2a7db06061796667484e04092ae6db09afab019a0c4e25dc8c868498e6c08b31 ../../../examples/erc20/rust/proof.json
```

Expected result should look similar to:

```sh
{ version: 1, initial_state: [237, ..., 61], next_state: [245, ..., 56], identity: "bob", tx_hash: [1], index: 0, payloads: [1, ..., 0], success: true, program_outputs: "Minted 100 to bob" }
```

### Register Contract on Hylé

- RISC Zero smart contracts are identified by their image ID. Two identical programs will have identical image IDs.
- State digest is our initial value: `ed286b3c39b2f86f9ce86bbc35455fe7e8f7b3f9683ba76e0bcc637eb5602f3d`.

Run:

```sh
./hyled tx zktx register default risczero 2a7db06061796667484e04092ae6db09afab019a0c4e25dc8c868498e6c08b31 erc20_rust ed286b3c39b2f86f9ce86bbc35455fe7e8f7b3f9683ba76e0bcc637eb5602f3d
```

The contract will be deployed with the state_digest value = 1 (AAAAAQ==).


### Publish Payload on Hylé

Run:

```sh
./hyled tx zktx publish "bob" erc20_rust <payload>
```


Once executed, get the transaction hash, which will be used for proof verification.  
Example: `DB38EC67872788B3B325ED52D3E487BBD1BFCBE98B2EDD63918DBB080E7BDDD0`.



### Prove on Hylé

Run by replacing `[transaction_hash]` with the one used to settle the payload: `DB38EC67872788B3B325ED52D3E487BBD1BFCBE98B2EDD63918DBB080E7BDDD0`.

```sh
./hyled tx zktx prove [transaction_hash] 0 erc20_rust ../examples/erc20/rust/proof.json
```



