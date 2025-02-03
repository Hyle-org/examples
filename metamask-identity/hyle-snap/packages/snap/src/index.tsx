import type {
  OnUserInputHandler,
  OnHomePageHandler,
  OnInstallHandler,
  OnSignatureHandler,
} from '@metamask/snaps-sdk';
import { UserInputEventType } from '@metamask/snaps-sdk';
import { Box, Heading, Text, Divider, Button } from '@metamask/snaps-sdk/jsx';

import { registerIdentity } from './hyle';

async function getAccount() {
  // Retrieve stored account
  let state = await snap.request({
    method: 'snap_manageState',
    params: { operation: 'get' },
  });

  // If no stored account, request from MetaMask
  if (!state || !state.account) {
    const accounts = await ethereum.request({ method: 'eth_requestAccounts' });

    // If accounts exist, store the first one
    if (accounts.length > 0) {
      state = { account: accounts[0] };

      // Save account in Snap state
      await snap.request({
        method: 'snap_manageState',
        params: { operation: 'update', newState: state },
      });
    } else {
      state = { account: 'No account found' };
    }
  }

  return state.account;
}

// Sign message using personal_sign
async function signMessage() {
  const message = 'hyle registration';
  const hexMessage = toHexMessage(message); // Convert message to hex
  console.log(hexMessage);
  const ethAddr = await ethereum.request({
    method: 'eth_requestAccounts',
  });
  console.log(ethAddr[0]);

  try {
    const signature = await ethereum.request({
      method: 'personal_sign',
      params: [hexMessage, ethAddr[0]],
    });

    return signature;
  } catch (error) {
    console.log(error);
    await snap.request({
      method: 'snap_notify',
      params: {
        type: 'inApp',
        message: `Signing failed: ${error.message} with account ${account} and message ${message}`,
      },
    });
    return 'Signing failed';
  }
}

// Convert message to hex format
function toHexMessage(message: string): string {
  return `0x${Buffer.from(message, 'utf8').toString('hex')}`;
}

// Store account on Snap install
export const onInstall: OnInstallHandler = async () => {
  const account = await getAccount();
  await snap.request({
    method: 'snap_dialog',
    params: {
      type: 'alert',
      content: (
        <Box>
          <Text>Connected Account: {account}</Text>
        </Box>
      ),
    },
  });
};

// Use stored account in Home Page
export const onHomePage: OnHomePageHandler = async () => {
  const account = await getAccount();
  return {
    content: (
      <Box>
        <Heading>Identity Registration</Heading>
        <Text>Connected Account: {account}</Text>
        <Divider />
        <Button name="register-button">Sign & register</Button>
      </Box>
    ),
  };
};

// Handle user actions
export const onUserInput: OnUserInputHandler = async ({ id, event }) => {
  if (event.type === UserInputEventType.ButtonClickEvent) {
    switch (event.name) {
      case 'register-button': {
        const signature = await signMessage();
        const ethAddr = await ethereum.request({
          method: 'eth_requestAccounts',
        });
        const generatedProof = await registerIdentity(signature, ethAddr[0]);

        await snap.request({
          method: 'snap_dialog',
          params: {
            type: 'alert',
            content: (
              <Box>
                <Text>Registration Completed !</Text>
                <Divider />
                <Text>generatedProof tx:</Text>
                <Text>{generatedProof}</Text>
              </Box>
            ),
          },
        });
        break;
      }
    }
  }
};

export const onSignature: OnSignatureHandler = async ({
  signature,
  signatureOrigin,
}) => {
  return {
    content: (
      <Box>
        <Heading>My Signature Insights</Heading>
        <Text>Here are the insights {signature}</Text>
        <Text>{signatureOrigin}</Text>
      </Box>
    ),
    severity: SeverityLevel.Critical,
  };
};
