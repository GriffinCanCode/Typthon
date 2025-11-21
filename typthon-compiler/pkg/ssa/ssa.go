// Package ssa implements Static Single Assignment form construction.
//
// Design: Minimal SSA - phi nodes at block joins, dominance-based.
// For Phase 1: simplified since we only have straight-line code
package ssa

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// Convert converts IR to SSA form
// Phase 1: IR is already in SSA form (single block, no phi nodes needed)
// Future: Full SSA with dominance frontiers for control flow
func Convert(prog *ir.Program) *Program {
	logger.Debug("Converting IR to SSA", "functions", len(prog.Functions))
	ssaProg := &Program{}

	for _, irFn := range prog.Functions {
		logger.Debug("Converting function to SSA", "name", irFn.Name, "blocks", len(irFn.Blocks))
		ssaFn := &Function{
			Name:   irFn.Name,
			Params: irFn.Params,
		}

		// Convert each block
		for _, irBlock := range irFn.Blocks {
			ssaBlock := &Block{
				Label: irBlock.Label,
				Insts: irBlock.Insts,
				Term:  irBlock.Term,
			}
			ssaFn.Blocks = append(ssaFn.Blocks, ssaBlock)
		}

		// Build CFG edges
		buildCFG(ssaFn)
		logger.LogSSAGeneration(irFn.Name, len(ssaFn.Blocks))

		ssaProg.Functions = append(ssaProg.Functions, ssaFn)
	}

	logger.Info("SSA conversion complete", "functions", len(ssaProg.Functions))
	return ssaProg
}

// buildCFG constructs control flow graph edges
func buildCFG(fn *Function) {
	blockMap := make(map[string]*Block)
	for _, block := range fn.Blocks {
		blockMap[block.Label] = block
	}

	// Connect blocks based on terminators
	for _, block := range fn.Blocks {
		switch term := block.Term.(type) {
		case *ir.Branch:
			if target, ok := blockMap[term.Target]; ok {
				block.Succs = append(block.Succs, target)
				target.Preds = append(target.Preds, block)
			}
		case *ir.CondBranch:
			if trueBlock, ok := blockMap[term.TrueBlock]; ok {
				block.Succs = append(block.Succs, trueBlock)
				trueBlock.Preds = append(trueBlock.Preds, block)
			}
			if falseBlock, ok := blockMap[term.FalseBlock]; ok {
				block.Succs = append(block.Succs, falseBlock)
				falseBlock.Preds = append(falseBlock.Preds, block)
			}
		}
	}
}

// Program in SSA form
type Program struct {
	Functions []*Function
}

type Function struct {
	Name   string
	Params []*ir.Param
	Blocks []*Block
}

type Block struct {
	Label string
	Phis  []*Phi
	Insts []ir.Inst
	Term  ir.Terminator
	Preds []*Block // Predecessors
	Succs []*Block // Successors
}

// Phi represents a φ node for SSA
type Phi struct {
	Dest   ir.Value
	Values []PhiValue
}

type PhiValue struct {
	Value ir.Value
	Block *Block
}

// Dominators computes the dominator tree using simple algorithm
// For Phase 1: entry block dominates everything (single block)
func (f *Function) Dominators() map[*Block]*Block {
	doms := make(map[*Block]*Block)
	if len(f.Blocks) == 0 {
		return doms
	}

	// Entry block dominates itself
	entry := f.Blocks[0]
	doms[entry] = entry

	// All other blocks dominated by entry
	for i := 1; i < len(f.Blocks); i++ {
		doms[f.Blocks[i]] = entry
	}

	return doms
}

// DominanceFrontiers computes dominance frontiers
// For Phase 1: no control flow, so frontiers are empty
func (f *Function) DominanceFrontiers() map[*Block][]*Block {
	frontiers := make(map[*Block][]*Block)

	for _, block := range f.Blocks {
		if len(block.Preds) >= 2 {
			// Join point - compute frontier
			for _, pred := range block.Preds {
				frontiers[pred] = append(frontiers[pred], block)
			}
		}
	}

	return frontiers
}
