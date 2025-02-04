# SP1 Project Template

This is a simple example of a token transfer contract working with SP1 prover.
The logic is very similar to the `simple_token` example (for risc0)

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

### Register the Program on hyle

To register the program (i.e. contract) on hyle with an initial supply of 1000 run:

```sh
cd script
cargo run --release -- register 1000
```

### Send a blob transaction and its proof for a transfer

```sh
cd script
cargo run --release -- transfer faucet.simple_token bob.simple_token 100
```

This will send the transactions to transfer 100 token from faucet to bob. The suffix `.simple_token` is for identity management.
It is the default name of this contract when it was registered. See [hyle documentation](https://docs.hyle.eu/developers/general-doc/identity/) for further details.
