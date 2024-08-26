package main

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/hyle-org/hyle/x/zktx/keeper/gnark"
)

// A more complex example circuit.
type CollatzCircuit struct {
	gnark.HyleCircuit
}

// This circuit verifies the Collatz conjecture between intput/output
func (circuit *CollatzCircuit) Define(api frontend.API) error {
	api.AssertIsEqual(circuit.Version, 1)
	api.AssertIsEqual(circuit.InputLen, 1)
	if circuit.Input[0] == 1 {
		// Cannot reset to 0
		api.AssertIsDifferent(circuit.Output[0], 0)
		api.AssertIsEqual(circuit.OutputLen, 1)
		return nil
	}
	isEven := api.Xor(circuit.Input[0], 1)
	if isEven == 1 {
		api.AssertIsEqual(circuit.Output[0], api.Div(circuit.Input[0], 2))
	} else {
		api.AssertIsEqual(circuit.Output[0], api.Add(api.Mul(circuit.Input[0], 3), 1))
	}
	api.AssertIsEqual(circuit.OutputLen, 1)
	return nil
}

func (circuit *CollatzCircuit) CreateResetWitness(resetTo int, origin string) (witness.Witness, error) {
	assignment := CollatzCircuit{
		HyleCircuit: gnark.HyleCircuit{
			Version:     1,
			InputLen:    1,
			Input:       []frontend.Variable{1},
			OutputLen:   1,
			Output:      []frontend.Variable{resetTo},
			IdentityLen: len(origin),
			Identity:    gnark.ToArray256([]byte(origin)),
			TxHash:      gnark.ToArray64([]byte("TODO")),
		},
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}
	return w, nil
}

func (circuit *CollatzCircuit) CreateNextWitness(current int, origin string) (witness.Witness, error) {
	next := 0
	if current == 1 {
		next = 1
	}
	isEven := current%2 == 0
	if isEven {
		next = current / 2
	} else {
		next = current*3 + 1
	}
	assignment := CollatzCircuit{
		HyleCircuit: gnark.HyleCircuit{
			Version:     1,
			InputLen:    1,
			Input:       []frontend.Variable{current},
			OutputLen:   1,
			Output:      []frontend.Variable{next},
			IdentityLen: len(origin),
			Identity:    gnark.ToArray256([]byte(origin)),
			TxHash:      gnark.ToArray64([]byte("TODO")),
		},
	}

	w, err := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}
	return w, nil
}
