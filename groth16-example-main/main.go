package main

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/hyle-org/hyle/x/zktx/keeper/gnark"
)

func Setup[T frontend.Circuit](circuit T) (constraint.ConstraintSystem, groth16.ProvingKey, groth16.VerifyingKey) {
	compiledCircuit, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, circuit)

	// groth16 zkSNARK: Setup
	provingKey, verifyingKey, _ := groth16.Setup(compiledCircuit)

	return compiledCircuit, provingKey, verifyingKey
}

func GenerateProof(ccs constraint.ConstraintSystem, witness witness.Witness, pk groth16.ProvingKey, vk groth16.VerifyingKey) (gnark.Groth16Proof, error) {
	// First generate the actual groth16 proof object
	proof, err := groth16.Prove(ccs, pk, witness)
	if err != nil {
		return gnark.Groth16Proof{}, err
	}

	// Then serialize all that in the format Hylé expects
	var proofBuf bytes.Buffer
	_, err = proof.WriteTo(&proofBuf)
	if err != nil {
		return gnark.Groth16Proof{}, err
	}

	var vkBuf bytes.Buffer
	_, err = vk.WriteTo(&vkBuf)
	if err != nil {
		return gnark.Groth16Proof{}, err
	}

	publicWitness, err := witness.Public()
	if err != nil {
		return gnark.Groth16Proof{}, err
	}

	var publicWitnessBuf bytes.Buffer
	_, err = publicWitness.WriteTo(&publicWitnessBuf)
	if err != nil {
		return gnark.Groth16Proof{}, err
	}

	return gnark.Groth16Proof{
		Proof:         proofBuf.Bytes(),
		VerifyingKey:  vkBuf.Bytes(),
		PublicWitness: publicWitnessBuf.Bytes(),
	}, nil
}

// This function exists to preserve private keys across runs
func loadCircuit(path string, circuit frontend.Circuit) (constraint.ConstraintSystem, groth16.ProvingKey, groth16.VerifyingKey, error) {
	// Read from file
	f, err := os.Open(path)
	if err != nil {
		// Create a new circuit
		css, pk, vk := Setup(circuit)
		// Serialize it as an example and so the pk/vk don't change
		var buf bytes.Buffer
		pk.WriteTo(&buf)
		vk.WriteTo(&buf)
		// There appears to be a bug in gnark 0.9 where readFrom reads too many bytes here, likely fixed in >= 0.11
		// So I'll put this last
		css.WriteTo(&buf)
		f, _ := os.Create(path)
		f.Write(buf.Bytes())
		return css, pk, vk, nil
	} else {
		pk := groth16.NewProvingKey(ecc.BN254)
		pk.ReadFrom(f)
		vk := groth16.NewVerifyingKey(ecc.BN254)
		vk.ReadFrom(f)
		css := groth16.NewCS(ecc.BN254)
		css.ReadFrom(f)
		return css, pk, vk, nil
	}
}

func simpleMain() {
	css, pk, vk, err := loadCircuit("simple_circuit.bin", &SimpleCircuit{})
	if err != nil {
		panic(err)
	}

	witness, _ := (&SimpleCircuit{}).CreateWitness(4)
	hyle_proof, _ := GenerateProof(css, witness, pk, vk)

	hyle_proof_marshalled, _ := json.Marshal(hyle_proof)
	f, _ := os.Create("simple_proof.json")
	f.Write(hyle_proof_marshalled)

	fmt.Println("Proof generated and saved to simple_proof.json")
	fmt.Println("Verifying key:", base64.StdEncoding.EncodeToString(hyle_proof.VerifyingKey))
	fmt.Println("Verify with hyled tx zktx verify")
}

func collatzMain() {
	css, pk, vk, err := loadCircuit("collatz_circuit.bin", &CollatzCircuit{
		HyleCircuit: gnark.HyleCircuit{
			// We need to specify the lengths of the arrays
			Input:  []frontend.Variable{1},
			Output: []frontend.Variable{1},
		},
	})
	if err != nil {
		panic(err)
	}

	witness, _ := (&CollatzCircuit{}).CreateResetWitness(4, "toto.collatz")
	hyle_proof, _ := GenerateProof(css, witness, pk, vk)

	hyle_proof_marshalled, _ := json.Marshal(hyle_proof)
	f, _ := os.Create("collatz_proof.json")
	f.Write(hyle_proof_marshalled)

	parsedData, _ := hyle_proof.ExtractData(witness)

	fmt.Println("Proof generated and saved to collatz_proof.json")
	fmt.Println("Initial state:", base64.StdEncoding.EncodeToString(parsedData.InitialState))
	fmt.Println("Verifying key:", base64.StdEncoding.EncodeToString(hyle_proof.VerifyingKey))
	fmt.Println("Register the contract with `hyled tx zktx register [your address] gnark-groth16-te-BN254 [verifying key] collatz [initial state]`")
	fmt.Println("Execute a state transition with `hyled tx zktx execute collatz [path to collatz_proof.json]`")
}

func main() {
	simpleMain()
	collatzMain()
}
