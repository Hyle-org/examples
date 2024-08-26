import { BarretenbergBackend  } from '@noir-lang/backend_barretenberg';
import { InputMap, Noir } from '@noir-lang/noir_js';
import * as fs from 'fs';
import * as dotenv from "dotenv";
dotenv.config({ path: '../Prover.toml' });

// Loading circuit
import circuit from "../target/noir_collatz_conjecture.json";

// Circuit tools setup
const backend = new BarretenbergBackend(circuit);
const verificationKey = await backend.getVerificationKey();

/////// LOCAL PROOF CREATION /////////
// Proving
const input = {
    version: process.env.version,
    initial_state_len: process.env.initial_state_len,
    initial_state: JSON.parse(process.env.initial_state as string),
    next_state_len: process.env.next_state_len,
    next_state: JSON.parse(process.env.next_state as string),
    identity_len: process.env.identity_len,
    identity: process.env.identity,
    tx_hash_len: process.env.tx_hash_len,
    tx_hash: JSON.parse(process.env.tx_hash as string),
    program_outputs: process.env.program_outputs
} as InputMap;
const noir = new Noir(circuit, backend);
const proof = await noir.generateProof(input);
var jsonProof = JSON.stringify({
    ...proof,
    proof: Array.from(proof.proof)
});
fs.writeFileSync('../target/proof.json', jsonProof);
fs.writeFileSync('../target/vkey', verificationKey);

process.exit();
