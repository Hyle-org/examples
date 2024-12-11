# Hylé example rust ZKVM smart contract

This repository provides an example smart contract for Hylé, implementing the Collatz Conjecture.
The smart contract is written in Rust, you can run it with several ZKVMs:

- RISC Zero
- SP1

## RISC Zero

### Installing RISC Zero

Please refer to [RiscZero's installation guide](https://dev.risczero.com/api/zkvm/install)

### Reproducible builds

RISC Zero provides using a docker setup. Simply run

```bash
cargo risczero build --manifest-path methods/guest/Cargo.toml
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

Install the [Hylé RISC Zero verifier](https://github.com/Hyle-org/verifiers-for-hyle).
You can then verify proofs using:

```sh
# The verifier currently expects no `0x` prefix. Pass data as base64 values.
cargo run -p risc0-verifier 14ecbb1bc768586be869bb32be7a337fc3c0c67d51ebfe9a4baf6570446b9e12 [path_to_proof] [initial_state] [final_state]
```

If the proof is malformed, or doesn't respect the rules of the smart contract, the verifier will return an error.

## Verifying on Hylé

Once you [installed the CLI](https://docs.hyle.eu/developers/using-the-cli/hyled-install-instructions/) and got [connected to devnet](https://docs.hyle.eu/developers/using-the-cli/connect-to-devnet/), you should be able to [_register_ and _execute_ for your contract](https://docs.hyle.eu/developers/using-the-cli/your-first-smart-contract/).
