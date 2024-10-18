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
   # To start from a fresh new inital state use -i argument
   cargo run -- -r -i mint bob 100
   # Or to fetch current onchain state of contract :
   cargo run -- -r mint bob 100
   ```

- Non-reproducibly:

   ```sh
   cargo run transfer alice bob 100

   ```

This will hyled commands you can use to send your transaction on hyle:

```sh
You can register the contract by running:
hyled contract default risc0 9bbfa2ef3131b9aab7db20ce72349ef3edf536d08d7932fa6dd1bf8f25d08d7a erc20_rust 0106666175636574fc40420f00
You can send the blob tx:
hyled blob IDENTITY erc20_rust 0005616c69636503626f6205
You can send the proof tx:
hyled proof BLOB_TX_HASH 0 erc20_rust erc20.risc0.proof
```

Key information includes:
- **Method ID**: `9bbfa2ef3131b9aab7db20ce72349ef3edf536d08d7932fa6dd1bf8f25d08d7a`, which will be used later to register the contract on Hylé.
- **Initial State Transition**: `0106666175636574fc40420f00`, which will be set when registering the contract on Hylé. This is an encoded version of the `Balances` object.
- **Blob payload**: ` 0005616c69636503626f6205`, which will be sequenced on the chain

### Register Contract on Hylé

You can install the hyled tool :


```sh
cargo install --git https://github.com/Hyle-org/hyle.git --bin hyled
```

And then run:

```sh
hyled contract default risc0 9bbfa2ef3131b9aab7db20ce72349ef3edf536d08d7932fa6dd1bf8f25d08d7a erc20_rust 0106666175636574fc40420f00
```

The contract will be deployed with an inital state with a faucet account 


### Publish Payload on Hylé

Run:

```sh
hyled blob "bob" erc20_rust <payload>
```


Once executed, get the transaction hash, which will be used for proof verification.  
Example: `DB38EC67872788B3B325ED52D3E487BBD1BFCBE98B2EDD63918DBB080E7BDDD0`.



### Prove on Hylé

Run by replacing `[transaction_hash]` with the one used to settle the payload: `DB38EC67872788B3B325ED52D3E487BBD1BFCBE98B2EDD63918DBB080E7BDDD0`.

```sh
hyled proof [transaction_hash] 0 erc20_rust erc20.risc0.proof
```



