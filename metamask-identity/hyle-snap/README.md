# MetaMask Snap: Hylé Identity Registration

## Overview

This MetaMask Snap enables users to generate a signature and register their connected account with a Hylé node. The Snap facilitates retrieving the user's Ethereum account, signing a message, and sending the signed identity to Hylé for registration.

## Features

- Retrieves and stores the user's Ethereum account.
- Signs a predefined message using `personal_sign`.
- Registers the signed identity with a Hylé node.
- Displays the registration result within the MetaMask Snap UI.

## Installation

To install and use this Snap, follow these steps:

1. Ensure you have MetaMask Flask installed. Flask is the developer-focused distribution of MetaMask required to run Snaps.
2. Clone this repository and install dependencies:
   ```sh
   git clone <repository_url>
   cd <project_directory>
   yarn install
   ```
3. Build and serve the Snap:
   ```sh
   yarn start
   ```
4. Open your browser and navigate to `http://localhost:8000` to install the Snap in MetaMask Flask.

## Usage

### Connecting an Account

On installation, the Snap retrieves and stores the user's Ethereum account. If no account is found, it prompts the user to connect an account through MetaMask.

### Signing and Registering Identity

Make sure Hyle node is running (localhost:4321) as well as metamask-identity server (localhost:3000) before starting.

1. Open the Snap's home page.
2. Click the `Register & Sign` button.
3. The Snap will sign a message (`hyle registration`) using the connected Ethereum account.
4. The signed identity is sent to the Hylé node for registration.
5. A confirmation message with the transaction details is displayed.

## API Details

### `getAccount()`

Retrieves the user's Ethereum account from storage or requests it from MetaMask if not already stored.

### `signMessage()`

Signs the predefined message `hyle registration` using `personal_sign` and returns the generated signature.

### `registerIdentity(signature, ethAddr)`

Sends the signed identity to the Hylé node for registration.

### Event Handlers

- `onInstall`: Retrieves and displays the connected account during Snap installation.
- `onHomePage`: Renders the Snap UI, displaying the connected account and a registration button.
- `onUserInput`: Handles user actions, triggers signing and identity registration upon button click.
- `onSignature`: Displays signature insights when an event related to signing occurs.
