# Hyle Identity Registration Demo with MetaMask Snap

This document provides instructions for running a demo of Hyle identity registration using a MetaMask Snap.

## Prerequisites

Before starting the demo, ensure you have the following installed:

- **MetaMask Flask:** Follow the instructions to install MetaMask Flask: [https://docs.metamask.io/snaps/get-started/install-flask/](https://docs.metamask.io/snaps/get-started/install-flask/)
- **Hyle:** Clone the Hyle repository: `git clone https://github.com/Hyle-org/hyle.git`

## Running the Demo

1.  **Start Hyle Node:**

    - Navigate to the Hyle project directory.
    - Run the following command:

    ```bash
    HYLE_RUN_INDEXER=false HYLE_CONSENSUS__SLOT_DURATION=5000 RUST_LOG=debug cargo run --bin node
    ```

2.  **Register Contract:**

    - Navigate to the `/metamask-contract` directory within the Hyle project.
    - Run the following command:

    ```bash
    cargo run -- register contract
    ```

3.  **Start Proof Server:**

    - Still in the `/metamask-contract` directory, run:

    ```bash
    RISC0DEV_MODE=1 cargo run -- run-server
    ```

4.  **Start Hyle Snap Install Server:**

    - Navigate to the `/hyle-snap` directory.
    - Run the following command:

    ```bash
    yarn start
    ```

5.  **Install Hyle Snap:**

    - Open your browser and connect to `localhost:8000`.
    - Follow the instructions on the page to install the Hyle snap.

6.  **Sign and Register:**

    - Use the installed Hyle snap within MetaMask to sign and register your identity.

7.  **Signature Validation:**

    - The signature and account data are sent to the `/metamask-contract` component.
    - This component validates the signature using the k256 elliptic curve implementation from the `risc0/RustCrypto-elliptic-curves` repository (specifically the `risczero/k256` branch): [https://github.com/risc0/RustCrypto-elliptic-curves/tree/risczero/k256](https://github.com/risc0/RustCrypto-elliptic-curves/tree/risczero/k256)

8.  **Account Registration:**
    - If the signature is valid, the account is registered.

Documentation :

- [Hylé developer docs](https://docs.hyle.eu/)

![Alt Text](metamask-snap.gif)
