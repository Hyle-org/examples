package main

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
)

// A simple sample groth16 circuit
type SimpleCircuit struct {
	X frontend.Variable `gnark:"x"`       // x  --> secret visibility (default)
	Y frontend.Variable `gnark:",public"` // Y  --> public visibility
}

// Define declares the circuit logic. The compiler then produces a list of constraints
// which must be satisfied (valid witness) in order to create a valid zk-SNARK
func (circuit *SimpleCircuit) Define(api frontend.API) error {
	// compute x**3 and store it in the local variable x3.
	x3 := api.Mul(circuit.X, circuit.X, circuit.X)

	// compute x**3 + x + 5 and store it in the local variable res
	res := api.Add(x3, circuit.X, 5)

	// assert that the statement x**3 + x + 5 == y is true.
	api.AssertIsEqual(circuit.Y, res)
	return nil
}

// This is a method for convenience
func (circuit *SimpleCircuit) CreateWitness(x int) (witness.Witness, error) {
	assignment := SimpleCircuit{
		X: frontend.Variable(x),
		Y: x*x*x + x + 5, // Can also be assigned directly
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}
	return w, nil
}
