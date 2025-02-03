// lib/hyle.ts

// Basic types from SDK
export type TxHash = string;
export type BlockHeight = number;
export type ContractName = string;
export type StateDigest = string;
export type Identity = string;
export type ValidatorPublicKey = string; // Assuming this is a string-based key
export type ProgramId = string;
export type Verifier = string;

export interface HyleOutput {
  version: number;
  initial_state: StateDigest;
  next_state: StateDigest;
  identity: Identity;
  tx_hash: TxHash;
  index: number;
  blobs: number[];
  success: boolean;
  program_outputs: number[];
}

// lib/myFile.ts
export interface BlobData {
  data: Uint8Array;
}

// Core interfaces matching Rust types
export interface Blob {
  contract_name: ContractName;
  data: number[];
}

export interface BlobTransaction {
  identity: Identity;
  blobs: Blob[];
}

export interface Proof {
  tx_hash: TxHash;
  contract_name: ContractName;
  identity: Identity;
  signature: string;
}

export interface ProofTransaction {
  contract_name: ContractName;
  proof: number[];
}

export async function registerIdentity(signature: string, ethAddr: string) {
  const HYLE_NODE_URL = 'http://localhost:4321';
  const HYLE_PROVER_URL = 'http://localhost:3000';
  const contract_name: Identity = 'metamask_identity';

  console.log('HYLE_NODE_URL', HYLE_NODE_URL);
  console.log(contract_name);

  const identity = ethAddr + '.' + contract_name;

  const action: IdentityAction = {
    type: 'RegisterIdentity',
    account: identity,
  };

  // Create the blob
  const blob: Blob = {
    contract_name: contract_name,
    data: [...new TextEncoder().encode(JSON.stringify(action))],
  };

  // Create the blob transaction
  const blobTx: BlobTransaction = {
    identity: identity,
    blobs: [blob],
  };

  console.log('blobTx generated');

  // Send blob transaction
  const response = await fetch(`${HYLE_NODE_URL}/v1/tx/send/blob`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(blobTx),
  });

  console.log('blobTx sent');

  const txHash = await response.text();

  // Create proof
  const proof: Proof = {
    tx_hash: txHash,
    contract_name: contract_name,
    identity: identity,
    signature: signature,
  };

  // Send proof transaction
  const responseProof = await fetch(`${HYLE_PROVER_URL}/prove`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(proof),
  });

  const generatedProof = await responseProof.text();

  return generatedProof;
}
