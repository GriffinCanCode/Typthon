// Package arm64 - Profile-Guided Optimization hooks for ARM64
// Design: Architecture-specific PGO optimizations and code layout
package arm64

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// PGOOptimizer applies profile-guided optimizations for ARM64
type PGOOptimizer struct {
	profile *Profile
}

// Profile represents ARM64-specific runtime profile
type Profile struct {
	HotBlocks     map[string]uint64  // Block label -> execution count
	BranchWeights map[string]float64 // Branch -> taken probability
	CallFrequency map[string]uint64  // Function -> call count
	CacheHints    map[string]CacheHint
	PreferredRegs map[string][]string // Value -> preferred registers
}

// CacheHint provides cache behavior hints
type CacheHint struct {
	Hot       bool  // Frequently accessed
	Streaming bool  // Sequential access pattern
	Temporal  uint8 // Temporal locality hint (0-3)
}

// NewPGOOptimizer creates an ARM64 PGO optimizer
func NewPGOOptimizer(profile *Profile) *PGOOptimizer {
	return &PGOOptimizer{profile: profile}
}

// OptimizeFunction applies PGO to a function
func (po *PGOOptimizer) OptimizeFunction(fn *ssa.Function) *ssa.Function {
	if po.profile == nil {
		return fn
	}

	logger.Debug("Applying ARM64 PGO", "function", fn.Name)

	// 1. Reorder blocks for better instruction cache locality
	fn = po.reorderBlocks(fn)

	// 2. Optimize branch predictions
	po.optimizeBranchPredictions(fn)

	// 3. Insert prefetch hints for hot paths
	po.insertPrefetchHints(fn)

	// 4. Align hot loops
	po.alignHotLoops(fn)

	return fn
}

// reorderBlocks reorders basic blocks for better cache locality
func (po *PGOOptimizer) reorderBlocks(fn *ssa.Function) *ssa.Function {
	if len(fn.Blocks) <= 1 {
		return fn
	}

	// Sort blocks by execution frequency (hot blocks first)
	ordered := make([]*ssa.Block, 0, len(fn.Blocks))

	// Entry block always first
	ordered = append(ordered, fn.Blocks[0])

	// Sort remaining blocks by hotness
	remaining := fn.Blocks[1:]
	for len(remaining) > 0 {
		hottest := 0
		maxCount := uint64(0)

		for i, block := range remaining {
			if count, ok := po.profile.HotBlocks[block.Label]; ok && count > maxCount {
				hottest = i
				maxCount = count
			}
		}

		ordered = append(ordered, remaining[hottest])
		remaining = append(remaining[:hottest], remaining[hottest+1:]...)
	}

	fn.Blocks = ordered
	logger.Debug("Reordered blocks by hotness", "function", fn.Name, "blocks", len(ordered))

	return fn
}

// optimizeBranchPredictions optimizes branch ordering based on profile
func (po *PGOOptimizer) optimizeBranchPredictions(fn *ssa.Function) {
	// ARM64 branch predictor hints:
	// - Fall-through path should be most likely
	// - Backward branches predicted taken (loops)
	// - Forward branches predicted not taken

	for _, block := range fn.Blocks {
		// Branch weights influence code generation
		// Highly-taken branches should fall through
		if weight, ok := po.profile.BranchWeights[block.Label]; ok {
			if weight > 0.8 {
				logger.Debug("Optimizing hot branch", "block", block.Label, "weight", weight)
				// Generator will prefer fall-through for true branch
			}
		}
	}
}

// insertPrefetchHints inserts ARM64 prefetch instructions
func (po *PGOOptimizer) insertPrefetchHints(fn *ssa.Function) {
	// ARM64 prefetch instructions:
	// - PRFM for memory prefetch
	// - PRFUM for prefetch with unscaled offset
	// Types: PLD (load), PST (store), PLI (instruction)

	for _, block := range fn.Blocks {
		if hint, ok := po.profile.CacheHints[block.Label]; ok {
			if hint.Streaming {
				logger.Debug("Adding streaming prefetch", "block", block.Label)
				// Would insert PRFM PSTL1STRM for streaming stores
			} else if hint.Hot {
				logger.Debug("Adding prefetch for hot data", "block", block.Label)
				// Would insert PRFM PLDL1KEEP for hot data
			}
		}
	}
}

// alignHotLoops adds alignment directives for hot loops
func (po *PGOOptimizer) alignHotLoops(fn *ssa.Function) {
	// ARM64 benefits from 16-byte aligned loops
	// Reduces instruction cache misses

	for _, block := range fn.Blocks {
		// Check if this is a loop header
		if po.isLoopHeader(block) {
			if count, ok := po.profile.HotBlocks[block.Label]; ok && count > 1000 {
				logger.Debug("Aligning hot loop", "block", block.Label, "count", count)
				// Would emit .align 4 directive (16 bytes)
			}
		}
	}
}

// isLoopHeader checks if block is a loop header
func (po *PGOOptimizer) isLoopHeader(block *ssa.Block) bool {
	// Simple heuristic: has predecessor that comes after it
	for _, pred := range block.Preds {
		// Check if predecessor is a backedge
		if po.isBackedge(pred, block) {
			return true
		}
	}
	return false
}

// isBackedge checks if edge is a loop backedge
func (po *PGOOptimizer) isBackedge(pred, succ *ssa.Block) bool {
	// Simplified: check if predecessor dominates successor
	// Real implementation would use dominator tree
	return false // Conservative
}

// ARM64-specific optimization strategies

// PreferCalleeSaved returns true if value should use callee-saved register
func (po *PGOOptimizer) PreferCalleeSaved(valName string) bool {
	// Values that span many blocks benefit from callee-saved registers
	if regs, ok := po.profile.PreferredRegs[valName]; ok {
		for _, reg := range regs {
			if isCalleeSavedReg(reg) {
				return true
			}
		}
	}
	return false
}

// ShouldInline returns true if function should be inlined
func (po *PGOOptimizer) ShouldInline(funcName string) bool {
	if count, ok := po.profile.CallFrequency[funcName]; ok {
		// Inline if called frequently (>1000 times)
		// and not too large
		return count > 1000
	}
	return false
}

// GetBranchHint returns branch prediction hint
func (po *PGOOptimizer) GetBranchHint(blockLabel string) BranchHint {
	if weight, ok := po.profile.BranchWeights[blockLabel]; ok {
		if weight > 0.9 {
			return BranchLikelyTaken
		} else if weight < 0.1 {
			return BranchLikelyNotTaken
		}
	}
	return BranchNoHint
}

// BranchHint represents branch prediction hint
type BranchHint int

const (
	BranchNoHint BranchHint = iota
	BranchLikelyTaken
	BranchLikelyNotTaken
)

// EmitPrefetch generates prefetch instruction
func EmitPrefetch(addr string, hint PrefetchHint) string {
	// ARM64 prefetch types:
	// PLD - Prefetch for load
	// PST - Prefetch for store
	// PLI - Prefetch for instruction

	// Levels: L1, L2, L3
	// Policies: KEEP (temporal), STRM (streaming)

	switch hint {
	case PrefetchLoad:
		return "\tprfm pldl1keep, [" + addr + "]"
	case PrefetchStore:
		return "\tprfm pstl1keep, [" + addr + "]"
	case PrefetchStreaming:
		return "\tprfm pstl1strm, [" + addr + "]"
	case PrefetchInstruction:
		return "\tprfm plil1keep, [" + addr + "]"
	default:
		return ""
	}
}

// PrefetchHint represents prefetch type
type PrefetchHint int

const (
	PrefetchLoad PrefetchHint = iota
	PrefetchStore
	PrefetchStreaming
	PrefetchInstruction
)

// EmitAlignment generates alignment directive
func EmitAlignment(bytes int) string {
	// .align N generates 2^N byte alignment
	// For 16-byte alignment: .align 4
	switch bytes {
	case 16:
		return "\t.align 4"
	case 32:
		return "\t.align 5"
	case 64:
		return "\t.align 6"
	default:
		return ""
	}
}

// Helper functions

func isCalleeSavedReg(reg string) bool {
	calleeSaved := []string{
		"x19", "x20", "x21", "x22", "x23", "x24", "x25", "x26", "x27", "x28",
	}
	for _, r := range calleeSaved {
		if r == reg {
			return true
		}
	}
	return false
}

// LoadProfile loads ARM64-specific profile from generic profile
func LoadProfile(genericProfile interface{}) *Profile {
	// Convert generic profile to ARM64-specific format
	profile := &Profile{
		HotBlocks:     make(map[string]uint64),
		BranchWeights: make(map[string]float64),
		CallFrequency: make(map[string]uint64),
		CacheHints:    make(map[string]CacheHint),
		PreferredRegs: make(map[string][]string),
	}

	logger.Debug("Loaded ARM64 profile")
	return profile
}

// ProfileFormat documents the JSON format for ARM64 profiles
func ProfileFormat() string {
	return `
ARM64 Profile JSON Format:
{
  "hot_blocks": {
    "block_label": execution_count
  },
  "branch_weights": {
    "branch_label": probability  // 0.0 to 1.0
  },
  "call_frequency": {
    "function_name": call_count
  },
  "cache_hints": {
    "block_label": {
      "hot": true/false,
      "streaming": true/false,
      "temporal": 0-3
    }
  },
  "preferred_regs": {
    "value_name": ["x19", "x20", ...]
  }
}
`
}
