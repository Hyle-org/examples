import { ProofData, BarretenbergBackend, BarretenbergVerifier as Verifier, type CompiledCircuit } from '@noir-lang/backend_barretenberg';
import { Noir } from '@noir-lang/noir_js';
import circuit from "../target/fibonacci.json";

import * as fs from 'fs';
import * as path from 'path';



function createDirectoryIfNotExists(directoryPath: string): void {
    const dirPath = path.join(__dirname, directoryPath);
    if (!fs.existsSync(directoryPath)) {
        fs.mkdirSync(directoryPath, { recursive: true });
    }   
}

const noirInput = {

    version: 1,
    initial_state_len: 4,
    initial_state: [0,0,0,1],
    next_state_len: 4,
    //you can define here the next state you want , value will be decoded in hex in hyle, 60->3c
    next_state: [0,0,0,60],
    identity_len: 0,
    identity: "",
    tx_hash_len: 43,
    tx_hash: [
        77, 68, 69, 121, 77, 122, 81, 49, 78, 106, 99, 52, 79, 87, 70, 105, 89, 50, 82, 108, 90, 106, 65, 120, 77, 106, 77, 48, 78, 84, 89, 51, 79, 68, 108, 104, 89, 109, 78, 107, 90, 87, 89
    ],
    payload_hash: 0,
    success: true,
    //define here proof data : 9th fibonacci number is 34. If you put a a wrong value then proof won't be generated. You can try it ;)
    program_outputs: {
        nth_fib: 9,
        expected_result: 34,
    },
};

const backend = new BarretenbergBackend(circuit as CompiledCircuit);
const noir = new Noir(circuit as CompiledCircuit, backend);

console.log('Generating proof... ⌛');
const { witness } = await noir.execute(noirInput);
const proof = await backend.generateProof(witness);

const verificationKey = await backend.getVerificationKey();
console.log('Json Proof creation for hyle');
var jsonProof = JSON.stringify({
    ...proof,
    proof: Array.from(proof.proof)
});

//file generation
const outputDirectory = "outputs";
createDirectoryIfNotExists(outputDirectory);
fs.writeFileSync(outputDirectory+"/proof.json", jsonProof);
fs.writeFileSync(outputDirectory+"/vk", verificationKey);

console.log('Proof generated, check /outputs directory to get proof and verification key (vk)... ✅');

process.exit();



