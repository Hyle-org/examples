# TicketApp/SimpleToken composition example

Welcome to this TicketApp example, this contract shows an example of composition on Hylé.

## Goal

The goal of this example is to attribute a ticket to a user. To do so, we need a ticket contract (TicketApp) that will check using composition that a valid transfer happened from a user to the ticket account.

## Prerequisites

- [Install Rust](https://www.rust-lang.org/tools/install) (you'll need `rustup` and Cargo).
- For our example, [install RISC Zero](https://dev.risczero.com/api/zkvm/install).
- [Start a single-node devnet](https://docs.hyle.eu/developers/quickstart/devnet/). We recommend using [dev-mode](https://dev.risczero.com/api/generating-proofs/dev-mode) with `-e RISC0_DEV_MODE=1` for faster iterations during development.

## Quick Start

### User minting, Bob's origins

First let's create an identity contract to declare a user we will use to buy a ticket.

Go to `./simple-identity` folder and run:

```bash
cargo run -- --contract-name id register-contract
```

Now we have an identity contract called `id` we can use to declare a user. Let's declare one!

```bash
cargo run -- --contract-name id register-identity bob.id pass
cargo run -- --contract-name id register-identity alice.id pass
```

We now have a user called *bob* on the contract `id`. We can refer to it with `bob.id`. His password is `pass`. Same for *alice*.

Let's verify it quickly with:

```bash
cargo run -- --contract-name id verify-identity bob.id pass 0
```

`0` is the nonce. Every time we verify successfully *bob*'s identity, it increments. Now if we want to verify it again, we should use `1` as nonce.
And for *alice*:

```bash
cargo run -- --contract-name id verify-identity alice.id pass 0
```

### Filling Bob's and Alice's bag


Go to `./simple-token` folder and run:

```bash
cargo run -- --contract-name simple-token register 1000
```

On the node's logs, you will see:

> 📝 Registering new contract simple_token

You just registered a token contract named simple-token with an initial supply of 1000. Now let's transfer some tokens to our user *bob*.

To send `50` tokens to *bob* and `10` to *alice*

```bash
cargo run -- --contract-name simple-token transfer faucet.simple-token bob.id 50
cargo run -- --contract-name simple-token transfer faucet.simple-token alice.id 10
```

The node's log will show:

> INFO hyle::data_availability::node_state::verifiers: ✅ Risc0 proof verified.
>
> INFO hyle::data_availability::node_state::verifiers: 🔎 Program outputs: Transferred 50 to bob.id
> INFO hyle::data_availability::node_state::verifiers: 🔎 Program outputs: Transferred 10 to alice.id

Check onchain balance:

```bash
cargo run -- --contract-name simple-token balance faucet.simple-token

cargo run -- --contract-name simple-token balance bob.id
cargo run -- --contract-name simple-token balance alice.id
```

Now that *bob* has some tokens, let's buy him a ticket.

Register the ticket app by going to `./ticket-app` folder and running:

```bash
cargo run -- --contract-name ticket-app register simple-token 15
```

Our ticket app is called `ticket-app`, and sells a ticket for `15` simple-token.

Let's buy a ticket for *bob*:

```bash
cargo run -- --contract-name ticket-app --user bob.id --pass pass --nonce 1 buy-ticket
```

Check that *bob* has a ticket:

```bash
cargo run -- --contract-name ticket-app --user bob.id has-ticket
```

You can also check Bob's balance and see he now has `35` tokens.

Let's try with *alice*:

```bash
cargo run -- --contract-name ticket-app --user alice.id buy-ticket
```

You will get an error while executing the TicketApp program: `Execution failed ! Program output: Insufficient balance`. This is because Alice has a balance of 10 and the ticket costs 15.

### Executing the Project Locally in Development Mode

During development, faster iteration upon code changes can be achieved by leveraging [dev-mode], we strongly suggest activating it during your early development phase. Furthermore, you might want to get insights into the execution statistics of your project, and this can be achieved by specifying the environment variable `RUST_LOG="[executor]=info"` before running your project.

Put together, the command to run your project in development mode while getting execution statistics is:

```bash
RUST_LOG="[executor]=info" RISC0_DEV_MODE=1 cargo run
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
