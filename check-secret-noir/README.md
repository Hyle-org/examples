# Check Secret Noir

This is a demonstration project that shows how to implement a secret checking system using Noir (a Domain Specific Language for SNARK proving systems) with a web frontend.

## Project Structure

- `/contract` - Contains the Noir smart contract code
  - `src/main.nr` - The main Noir contract implementing the secret checking logic
- `/frontend` - Web interface implementation

## Features

- Zero-knowledge proof based secret verification
- Web interface for submitting identity and password
- Real-time proof verification display
- Secure password handling through zero-knowledge proofs, settling on Hyle network.

## Setup and Running

1. Install dependencies:
```bash
# In the frontend directory
bun install
```

2. Run the frontend development server:
```bash
# In the frontend directory
bun run dev
```

## Security

This example demonstrates zero-knowledge proof concepts for password verification. The password is never directly exposed in the verification process, enhancing security through cryptographic proofs.

