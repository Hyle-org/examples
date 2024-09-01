# Noir collatz conjecture

Implementation of the collatz conjecture in Noir, similar to the RISC Zero implementation at [hyle_org/collatz_conjecture](https://github.com/Hyle-org/collatz-conjecture).

## Usage

Install Nargo v0.30, as other versions won't be compatible for now.

```sh
nargo build
cd prover
bun update
bun hyle_custom_prover.ts
```
