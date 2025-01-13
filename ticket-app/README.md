# TicketApp/SimpleToken composition example

Welcome to this TicketApp example, this contract shows an example of composition on Hylé.

## Goal

The goal of this example is to attribute a ticket to a user. To do so, we need a ticket contract (TicketApp) that will check using composition that a valid transfer happened from a user to the ticket account.

## Quick Start

First, make sure [rustup] is installed. The
[`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to
automatically install the correct version.

First let's go to `./simple-token` folder and run:

```bash
cargo run -- --contract-name simple-token register 1000
```

On the node's logs, you should see a line like

> 📝 Registering new contract simple_token

You just registered a token contract named simple-token with an initial supply of 1000! Now let's transfer some tokens to our user *bob*.

To send a blob & proof transactions to send 50 tokens to *bob* you can run:

```bash
cargo run -- -contract-name simple-token transfer faucet.simple-token bob.ticket-app 50
cargo run -- -contract-name simple-token transfer faucet.simple-token alice.ticket-app 10
```

On node's logs you should see:

> INFO hyle::data_availability::node_state::verifiers: ✅ Risc0 proof verified.
>
> INFO hyle::data_availability::node_state::verifiers: 🔎 Program outputs: Transferred 50 to bob.ticket_app
> INFO hyle::data_availability::node_state::verifiers: 🔎 Program outputs: Transferred 10 to alice.ticket_app

You can check onchain balance:

```bash
cargo run -- --contract-name simple-token balance faucet.simple-token

cargo run -- --contract-name simple-token balance bob.ticket-app
cargo run -- --contract-name simple-token balance alice.ticket-app
```

Note: The example does not compose with an identity contract, thus no identity verification is made.
This is the reason of the suffix ".simple-token" and ".ticket-app" on the "from" & "to" transfer fields. More info to come in the documentation.

Now *bob* has some tokens, let's buy a ticket.

Let's register the ticket app, to do so, go to `./ticket-app` folder and run:

```bash
cargo run -- --contract-name ticket-app register simple-token 15
```

Our ticket app is called ticket_app, and sells a ticket for 15 simple-token.

Let's buy a ticket to *bob*

```bash
cargo run -- --contract-name ticket-app --user bob.ticket-app buy-ticket
```

Now check *bob* has a ticket

```bash
cargo run -- --contract-name ticket-app --user bob.ticket-app has-ticket
```

You can check his balance and see it has been debited (should be 35 now).

Let's try with *alice*

```bash
cargo run -- --contract-name ticket-app --user alice.ticket-app buy-ticket
```

You should get an error while executing the TicketApp program `Execution failed ! Program output: Insufficient balance` since Alice has a balance of 10 and the ticket costs 15.

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
