# Check Secret Noir Library

A TypeScript library for securely handling secret verification using zero-knowledge proofs with Noir circuits to be settled on Hyle network.

## Main Functions

### `build_blob_transaction(identity: string, password: string): Promise<BlobTransaction>`

Creates a blob transaction containing a securely hashed secret. This function:

1. Takes an identity and password as input
2. Processes the secret to be handled by the circuit
3. Returns a blob transaction ready to be submitted

Example usage:
```typescript
import { NodeApiHttpClient } from "hyle";

const node = new NodeApiHttpClient("http://127.0.0.1:4321");
const blobTx = await build_blob_transaction("user123", "myPassword");
const txHash = await node.sendBlobTx(blobTx);
```

### `build_proof_transaction(identity: string, password: string, tx_hash: string, circuit?: CompiledCircuit): Promise<ProofTransaction>`

Generates a zero-knowledge proof transaction that demonstrates knowledge of a secret without revealing it. This function:

1. Takes an identity, password, and transaction hash as input
2. Creates a proof that shows:
   - The user knows the password
   - The password matches the stored hash
   - All without revealing the actual password
3. Returns a proof transaction that can be verified by others

Example usage:
```typescript
const proofTx = await build_proof_transaction(
  "user123",
  "myPassword",
  tx_hash 
);
await node.sendProofTx(proofTx);
```

## Installation

```bash
npm install check-secret-noir
# or
yarn add check-secret-noir
```

## Dependencies

- @noir-lang/noir_js
- @aztec/bb.js
- @noir-lang/types
- hyle

## Security

This library implements secure cryptographic operations to protect secrets:
- Zero-knowledge proofs ensure privacy
- No plain text secrets are ever stored or transmitted

