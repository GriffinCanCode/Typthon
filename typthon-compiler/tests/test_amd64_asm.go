// Quick test to generate amd64 assembly
package main

import (
	"os"

	"github.com/GriffinCanCode/typthon-compiler/pkg/codegen/amd64"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

func main() {
	// Create simple add function IR
	param_a := &ir.Param{Name: "a", Type: ir.IntType{}}
	param_b := &ir.Param{Name: "b", Type: ir.IntType{}}

	temp := &ir.Temp{ID: 0, Type: ir.IntType{}}

	fn := &ir.Function{
		Name:       "add",
		Params:     []*ir.Param{param_a, param_b},
		ReturnType: ir.IntType{},
		Blocks: []*ir.Block{
			{
				Label: "entry",
				Insts: []ir.Inst{
					&ir.BinOp{
						Dest: temp,
						Op:   ir.OpAdd,
						L:    param_a,
						R:    param_b,
					},
				},
				Term: &ir.Return{Value: temp},
			},
		},
	}

	prog := &ir.Program{
		Functions: []*ir.Function{fn},
	}

	// Convert to SSA
	ssaProg := ssa.Convert(prog)

	// Generate amd64 assembly
	gen := amd64.NewGenerator(os.Stdout)
	if err := gen.Generate(ssaProg); err != nil {
		panic(err)
	}
}
