# Simple Identity risc0 example

On Hylé, any smart contract can serve as proof of identity. This flexibility allows you to register your preferred identity source as a smart contract for account identification. Hylé also ships [a native `hydentity` contract](https://github.com/Hyle-org/hyle/tree/main/contracts/hydentity) for simplicity.

This is a Risc0 example called simple_identity.

## Prerequisites

- [Install Rust](https://www.rust-lang.org/tools/install) (you'll need `rustup` and Cargo).
- For our example, [install RISC Zero](https://dev.risczero.com/api/zkvm/install).
- [Start a single-node devnet](https://docs.hyle.eu/developers/quickstart/devnet/). We recommend using [dev-mode](https://dev.risczero.com/api/generating-proofs/dev-mode) with `-e RISC0_DEV_MODE=1` for faster iterations during development.

## Quickstart

### Build and register the identity contract

To build all methods and register the smart contract on the local node [from the source](https://github.com/Hyle-org/examples/blob/simple_erc20/simple-token/host/src/main.rs), run:

```bash
cargo run -- register-contract
```

The expected output is `📝 Registering new contract simple_identity`.

### Register an account / Sign up

To register an account with a username (`alice`) and password (`abc123`), execute:

```sh
cargo run -- register-identity alice.simple_identity abc123
```

The node's logs will display:

```bash
INFO hyle_verifiers: ✅ Risc0 proof verified.
```

### Verify identity / Login

To verify `alice`'s identity:

```bash
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

```bash
INFO hyle_verifiers: ✅ Risc0 proof verified.
```

### Executing the Project Locally in Development Mode

During development, faster iteration upon code changes can be achieved by leveraging [dev-mode], we strongly suggest activating it during your early development phase.

```bash
RISC0_DEV_MODE=1 cargo run
```

<!--### Running Proofs Remotely on Bonsai-->
<!---->
<!--_Note: The Bonsai proving service is still in early Alpha; an API key is-->
<!--required for access. [Click here to request access][bonsai access]._-->
<!---->
<!--If you have access to the URL and API key to Bonsai you can run your proofs-->
<!--remotely. To prove in Bonsai mode, invoke `cargo run` with two additional-->
<!--environment variables:-->
<!---->
<!--```bash-->
<!--BONSAI_API_KEY="YOUR_API_KEY" BONSAI_API_URL="BONSAI_URL" cargo run-->
<!--```-->

## How to create a project based on this example

- The [RISC Zero Developer Docs][dev-docs] is a great place to get started.
- Example projects are available in the [examples folder][examples] of
  [`risc0`][risc0-repo] repository.
- Reference documentation is available at [https://docs.rs][docs.rs], including
  [`risc0-zkvm`][risc0-zkvm], [`cargo-risczero`][cargo-risczero],
  [`risc0-build`][risc0-build], and [others][crates].

## Directory Structure

It is possible to organize the files for these components in various ways.
However, in this starter template we use a standard directory structure for zkVM
applications, which we think is a good starting point for your applications.

```text
project_name
├── Cargo.toml
├── contract 
│   ├── Cargo.toml
│   └── src
│       └── lib.rs         <-- [Contract code goes here, common to host & guest]
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs        <-- [Host code goes here]
└── methods
    ├── Cargo.toml
    ├── build.rs
    ├── guest
    │   ├── Cargo.toml
    │   └── src
    │       └── main.rs    <-- [Guest code goes here]
    └── src
        └── lib.rs
```

<!--[bonsai access]: https://bonsai.xyz/apply-->
[cargo-risczero]: https://docs.rs/cargo-risczero
[crates]: https://github.com/risc0/risc0/blob/main/README.md#rust-binaries
[dev-docs]: https://dev.risczero.com
[dev-mode]: https://dev.risczero.com/api/generating-proofs/dev-mode
[docs.rs]: https://docs.rs/releases/search?query=risc0
[examples]: https://github.com/risc0/risc0/tree/main/examples
[risc0-build]: https://docs.rs/risc0-build
[risc0-repo]: https://www.github.com/risc0/risc0
[risc0-zkvm]: https://docs.rs/risc0-zkvm
[rust-toolchain]: rust-toolchain.toml
[rustup]: https://rustup.rs
[zkvm-overview]: https://dev.risczero.com/zkvm
