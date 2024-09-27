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
   cargo run -- -r transfer alice bob 100
   ```

- Non-reproducibly:

   ```sh
   cargo run transfer alice bob 100
   ```


This will output:

```sh
Method ID: Digest(e200c46f3df94f8b8fa556e67dc484e9d3b15379c2d51d1973dbc821e7564c34) (hex)
proof.json written, transition from AAAAAQ== (1) to AAAAAg== (2)
Program outputted "Transferred 100 from alice to bob. Sender new balance: 900, Receiver new balance: 600"
```

Key information includes:
- **Method ID**: `e200c46f3df94f8b8fa556e67dc484e9d3b15379c2d51d1973dbc821e7564c34`, which will be used later to register the contract on Hylé.
- **Initial State Transition**: `AAAAAQ== (1)`, which will be set when registering the contract on Hylé.
- **Next State Transition**: `AAAAAg== (2)`, which will be visible on Hylé once the proof is validated.

### Proof Verification - Locally

Install the [Hylé RISC Zero verifier](https://github.com/Hyle-org/verifiers-for-hyle).

You can then verify proofs in **risc0-verifier/**, run:

```sh
cargo run -p risc0-verifier e200c46f3df94f8b8fa556e67dc484e9d3b15379c2d51d1973dbc821e7564c34 ../../../examples/erc20/rust/proof.json
```

Expected result should look similar to:

```sh
{"version":1,"initial_state":[0,0,0,1],"next_state":[0,0,0,2],"identity":"alice","tx_hash":[1],"program_outputs":null}
```

### Register Contract on Hylé

- RISC Zero smart contracts are identified by their image ID. Two identical programs will have identical image IDs.
- State digest is our initial value: `AAAAAQ== (1)`.

Run:

```sh
./hyled tx zktx register default risczero e200c46f3df94f8b8fa556e67dc484e9d3b15379c2d51d1973dbc821e7564c34 erc20_rust AAAAAQ==
```

The contract will be deployed with the state_digest value = 1 (AAAAAQ==).


### Publish Payload on Hylé

Run:

```sh
./hyled tx zktx publish "" erc20_rust AAAAAg==
```


Once executed, get the transaction hash, which will be used for proof verification.  
Example: `DB38EC67872788B3B325ED52D3E487BBD1BFCBE98B2EDD63918DBB080E7BDDD0`.



### Prove on Hylé

Run by replacing `[transaction_hash]` with the one used to settle the payload: `DB38EC67872788B3B325ED52D3E487BBD1BFCBE98B2EDD63918DBB080E7BDDD0`.

```sh
./hyled tx zktx prove [transaction_hash] 0 erc20_rust ../examples/erc20/rust/proof.json
```



