# Hylé example rust ZKVM smart contract

This repository provides an example smart contract for Hylé, implementing the Collatz Conjecture.
The smart contract is written in Rust, you can run it with several ZKVMs:

- RISC Zero
- SP1

## RISC Zero

To use RISC Zero, you'll need to compile with

```
cargo build --features risc0
```

The matching binary is `risc0-runner`.

### Installing RISC Zero

Please refer to [RiscZero's installation guide](https://dev.risczero.com/api/zkvm/install)

### Reproducible builds

RISC Zero provides reproducible builds using a docker setup. Simply run

```bash
cargo risczero build
```

to build the smart contract.

## SP1

To use SP1, you'll need to compile with

```
cargo build --features sp1
```

The matching binary is `sp1-runner`.

### Installing SP1

Please refer to [SP1's installation guide](https://docs.succinct.xyz/docs/getting-started/install)

### Reproducible builds

SP1 provides reproducible builds using a docker setup. Simply run

```bash
cargo prove build --docker
```

to build the smart contract.

## Running the smart contract

```bash
cargo run next X # Generate a proof of the transition from X to the next number in the collatz conjecture
# Or reproducibly
cargo run -- -r next X
```

```bash
cargo run reset X # Reset to X, assuming the current number is a 1
# Or reproducibly
cargo run -- -r reset X
```

### Verifying locally

Coming soon!
