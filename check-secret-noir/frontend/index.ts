import { Noir } from "@noir-lang/noir_js";
import { reconstructHonkProof, UltraHonkBackend } from "@aztec/bb.js";
import circuit from "../contract/target/check_secret.json";
import { CompiledCircuit, InputMap } from "@noir-lang/types";
import {
  BlobTransaction,
  NodeApiHttpClient,
  Blob,
  ProofTransaction,
} from "hyle";
import { register as registerBlob } from "./model";

const node = new NodeApiHttpClient("http://127.0.0.1:4321");

const show = (id: string, content: string): void => {
  const container = document.getElementById(id);
  if (container) {
    container.appendChild(document.createTextNode(content));
    container.appendChild(document.createElement("br"));
  }
};

const sha256 = async (data: Uint8Array): Promise<Uint8Array> => {
  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  return new Uint8Array(hashBuffer);
};

const stringToBytes = (input: string): Uint8Array => {
  return new TextEncoder().encode(input);
};

const encodeToHex = (data: Uint8Array): string => {
  return Array.from(data)
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
};

export const assert = (condition: boolean, message: string): void => {
  if (!condition) {
    throw new Error(message);
  }
};

const generateProverData = (
  id: string,
  pwd: Uint8Array,
  stored_hash: Uint8Array,
  tx: string,
): InputMap => {
  const version = 1;
  const initial_state = [0, 0, 0, 0];
  const initial_state_len = initial_state.length;
  const next_state = [0, 0, 0, 0];
  const next_state_len = next_state.length;
  const identity_len = id.length;
  const identity = id.padEnd(64, "0");
  const tx_hash = tx.padEnd(64, "0");
  const tx_hash_len = tx_hash.length;
  const index = 0;
  const blob_number = 1;
  const blob_index = 0;
  const blob_contract_name_len = "check_secret".length;
  const blob_contract_name = "check_secret".padEnd(64, "0");
  const blob_len = 32;
  const blob: number[] = Array.from(stored_hash);
  const tx_blob_count = 1;
  const success = 1;
  const password: number[] = Array.from(pwd);
  assert(password.length == 32, "Password length is not 32 bytes");
  assert(blob.length == blob_len, "Blob length is not 32 bytes");

  return {
    version,
    initial_state,
    initial_state_len,
    next_state,
    next_state_len,
    identity,
    identity_len,
    tx_hash,
    tx_hash_len,
    index,
    blob_number,
    blob_index,
    blob_contract_name_len,
    blob_contract_name,
    blob_len,
    blob,
    tx_blob_count,
    success,
    password,
  };
};

const register = async (identity: string, storedHash: Uint8Array) => {
  const idBlob = registerBlob(identity);

  const secretBlob: Blob = {
    contract_name: "check_secret",
    data: Array.from(storedHash),
  };

  return await sendBlobTx(identity, [/*idBlob,*/ secretBlob]);
};

const sendProoof = async (proof: Uint8Array) => {
  const proofTx: ProofTransaction = {
    contract_name: "check_secret",
    proof: Array.from(proof),
  };
  return await node.sendProofTx(proofTx);
};
const sendBlobTx = async (identity: string, blobs: Blob[]) => {
  const blobTx: BlobTransaction = {
    identity: identity,
    blobs,
  };
  return await node.sendBlobTx(blobTx);
};

document.getElementById("submit")?.addEventListener("click", async () => {
  try {
    const noir = new Noir(circuit as CompiledCircuit);
    const backend = new UltraHonkBackend(circuit.bytecode);

    // Prepare inputs
    const identity = (document.getElementById("identity") as HTMLInputElement)
      ?.value;
    const password = (document.getElementById("password") as HTMLInputElement)
      ?.value;

    if (!identity || !password) {
      show("logs", "Identity and password are required.");
      return;
    }

    const hashed_password_bytes = await sha256(stringToBytes(password));
    console.log(`hashed_password_bytes: ${hashed_password_bytes}`);

    let encoder = new TextEncoder();
    let id_prefix = encoder.encode(`${identity.padEnd(64, "0")}:`);
    let extended_id = new Uint8Array([...id_prefix, ...hashed_password_bytes]);
    console.log(`extended_id: ${extended_id}`);

    const stored_hash = await sha256(extended_id);
    console.log(`Hashed Id: ${stored_hash}`);
    console.log(`Hashed Id (hex): ${encodeToHex(stored_hash)}`);

    show("logs", `Built account id hash: ${stored_hash}... ✅`);

    const tx_hash = await register(identity, stored_hash);

    show("logs", "Register transaction sent... ✅");

    show("logs", "Generating witness... ⏳");
    const { witness } = await noir.execute(
      generateProverData(identity, hashed_password_bytes, stored_hash, tx_hash),
    );
    show("logs", "Generated witness... ✅");
    console.log("Witness: ", witness);

    show("logs", "Generating proof... ⏳");
    const before = Date.now();
    const proof = await backend.generateProof(witness);
    const after = Date.now();
    const time = after - before;
    show("logs", `Generated proof in ${(time / 1000).toFixed(2)}sec... ✅`);
    show("results", proof.proof.toString());

    await sendProoof(
      reconstructHonkProof(
        flattenFieldsAsArray(proof.publicInputs),
        proof.proof,
      ),
    );
    show("logs", "Proof sent... ✅");

    show("logs", "Verifying proof... ⌛");
    const isValid = await backend.verifyProof(proof);
    show("logs", `Proof is ${isValid ? "valid" : "invalid"}... ✅`);
  } catch (err: unknown) {
    console.error(err);
    const errorMessage =
      err instanceof Error ? err.message : "Unknown error occurred";
    show("logs", "Error: " + errorMessage);
  }
});

export function flattenFieldsAsArray(fields: string[]): Uint8Array {
  const flattenedPublicInputs = fields.map(hexToUint8Array);
  return flattenUint8Arrays(flattenedPublicInputs);
}

function flattenUint8Arrays(arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((acc, val) => acc + val.length, 0);
  const result = new Uint8Array(totalLength);

  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }

  return result;
}

function hexToUint8Array(hex: string): Uint8Array {
  const sanitisedHex = BigInt(hex).toString(16).padStart(64, "0");

  const len = sanitisedHex.length / 2;
  const u8 = new Uint8Array(len);

  let i = 0;
  let j = 0;
  while (i < len) {
    u8[i] = parseInt(sanitisedHex.slice(j, j + 2), 16);
    i += 1;
    j += 2;
  }

  return u8;
}
