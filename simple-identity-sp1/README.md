# SP1 Project Template

This is a simple example of a token transfer contract working with SP1 prover.
The logic is the very similar to the `simple_token` example (for risc0)

## Requirements

- [Rust](https://rustup.rs/)
- [SP1 4.0.0-rc1](https://docs.succinct.xyz/getting-started/install.html)
- [A running hyle devnet](https://docs.hyle.eu/developers/quickstart/devnet/)

## Running the Project

### Build the Program

To build the program, run the following command:

```sh
cd program
cargo prove build
```

### Build and register the identity contract

To build all methods and register the smart contract on the local node from the source, run:

```sh
cd script
cargo run -- register-contract
```
The expected output is `📝 Registering new contract simple_identity`.

### Register an account / Sign up

To register an account with a username (alice) and password (abc123), execute:

```sh
cargo run -- register-identity alice.simple_identity abc123
```
The node's logs will display:

```sh
INFO hyle_verifiers: ✅ SP1 proof verified.

```
### Verify identity / Login

To verify alice's identity:

```sh
cargo run -- verify-identity alice.simple_identity abc123 0
```
This command will:

1. Send a blob transaction to verify `alice`'s identity.
1. Generate a ZK proof of that identity. It will only be valid once, thus the inclusion of a nonce.
1. Send the proof to the devnet.

Upon reception of the proof, the node will:

1. Verify the proof.
1. Settle the blob transaction.
1. Update the contract's state.

The node's logs will display:

