// console.log("Hello via Bun!");
import { BarretenbergBackend, BarretenbergVerifier as Verifier, type CompiledCircuit } from '@noir-lang/backend_barretenberg';


import * as fs from 'fs';

interface HyleOutput {
    version: number;
    initial_state: number[];
    next_state: number[];
    identity: string;
    tx_hash: number[];
    payload_hash: number[];
    success: boolean;
  }

function deserializePublicInputs<T>(publicInputs: string[]): HyleOutput {
    const version = parseInt(publicInputs.shift() as string);
    const initial_state = parseArray(publicInputs);
    const next_state = parseArray(publicInputs);
    const identity = parseString(publicInputs);
    const tx_hash = parseArray(publicInputs);
    const newLocal = publicInputs.shift();
    const payload_hash = bigintToBytesArray(BigInt(newLocal));
    const success = parseInt(publicInputs.shift()) === 1;
    // We don't parse the rest, which correspond to programOutputs
  
    return {
      version,
      initial_state,
      next_state,
      identity,
      tx_hash,
      payload_hash,
      success,
    };
  }

  function parseString(vector: string[]): string {
    let length = parseInt(vector.shift() as string);
    let resp = "";
    for (var i = 0; i < length; i += 1)
      resp += String.fromCharCode(parseInt(vector.shift() as string, 16));
    return resp;
  }
  
  function parseArray(vector: string[]): number[] {
    let length = parseInt(vector.shift() as string);
    let resp: number[] = [];
    for (var i = 0; i < length; i += 1)
      resp.push(parseInt(vector.shift() as string, 16));
    return resp;
  }
  
  function bigintToBytesArray(bigint: bigint): number[] {
    const byteArray: number[] = [];
    let tempBigInt = bigint;
  
    while (tempBigInt > 0n) {
      const byte = Number(tempBigInt & 0xffn);
      byteArray.push(byte);
      tempBigInt >>= 8n;
    }
  
    while (byteArray.length < 4) {
      byteArray.push(0);
    }
  
    if (byteArray.length === 0) {
      byteArray.push(0);
    }
  
    return byteArray.reverse();
  }

const proofFromJson = JSON.parse(
    //fs.readFileSync("/tmp/noir-proof.json", { encoding: "utf8" })
    fs.readFileSync("outputs/proof.json", { encoding: "utf8" })
);
//const vKey = fs.readFileSync("/tmp/noir-vkey");
const vKey = fs.readFileSync("outputs/vk");

const deserializedProofData: ProofData = {
    proof: Uint8Array.from(proofFromJson.proof),
    publicInputs: proofFromJson.publicInputs,
};

const verifier = new Verifier();
const isValid = await verifier.verifyProof(deserializedProofData, vKey);
if (isValid) {
    console.log('Proof validated..start deserializing output');
    const hyleOutput = deserializePublicInputs(
        deserializedProofData.publicInputs
      );
      // bigint in json serialization is a pain in the ass :cry:
  // Disgusting work around -> needs refacto.
  var stringified_output = JSON.stringify(hyleOutput, (_, v) =>
    typeof v === "bigint" ? "BIGINT_" + v.toString() + "_BIGINT" : v
  );
  stringified_output = stringified_output.replaceAll('"BIGINT_', "");
  stringified_output = stringified_output.replaceAll('_BIGINT"', "");

  console.log(stringified_output);
} else {
    console.log('Proof invalid');
}

process.exit();



