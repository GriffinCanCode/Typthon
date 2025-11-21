// Package arm64 - ARM64-specific peephole optimizations
// Design: Architecture-aware pattern matching and optimization
package arm64

import (
	"strings"

	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// PeepholeOptimizer performs ARM64-specific peephole optimizations
type PeepholeOptimizer struct {
	patterns []Pattern
}

// Pattern represents an optimization pattern
type Pattern struct {
	Name    string
	Match   func([]string) bool
	Replace func([]string) []string
}

// NewPeepholeOptimizer creates an ARM64 peephole optimizer
func NewPeepholeOptimizer() *PeepholeOptimizer {
	po := &PeepholeOptimizer{
		patterns: make([]Pattern, 0),
	}
	po.registerPatterns()
	return po
}

// Optimize applies peephole optimizations to assembly code
func (po *PeepholeOptimizer) Optimize(assembly string) string {
	lines := strings.Split(assembly, "\n")
	optimized := po.optimizeLines(lines)
	return strings.Join(optimized, "\n")
}

// optimizeLines applies patterns to assembly lines
func (po *PeepholeOptimizer) optimizeLines(lines []string) []string {
	result := make([]string, 0, len(lines))
	i := 0

	for i < len(lines) {
		// Try to match patterns
		matched := false
		for _, pattern := range po.patterns {
			// Check if we have enough lines for pattern
			window := min(5, len(lines)-i)
			if window < 2 {
				break
			}

			windowLines := lines[i : i+window]
			if pattern.Match(windowLines) {
				replaced := pattern.Replace(windowLines)
				result = append(result, replaced...)
				i += window
				matched = true
				logger.Debug("Applied peephole optimization", "pattern", pattern.Name)
				break
			}
		}

		if !matched {
			result = append(result, lines[i])
			i++
		}
	}

	return result
}

// registerPatterns registers all optimization patterns
func (po *PeepholeOptimizer) registerPatterns() {
	// Pattern 1: Redundant move elimination
	po.patterns = append(po.patterns, Pattern{
		Name: "redundant_mov",
		Match: func(lines []string) bool {
			if len(lines) < 2 {
				return false
			}
			// mov x0, x1 followed by mov x1, x0
			return isMov(lines[0]) && isMov(lines[1]) &&
				getMovDest(lines[0]) == getMovSrc(lines[1]) &&
				getMovSrc(lines[0]) == getMovDest(lines[1])
		},
		Replace: func(lines []string) []string {
			// Keep only first move
			return []string{lines[0]}
		},
	})

	// Pattern 2: Load-store forwarding
	po.patterns = append(po.patterns, Pattern{
		Name: "load_store_forward",
		Match: func(lines []string) bool {
			if len(lines) < 2 {
				return false
			}
			// ldr x0, [x1] followed by str x0, [x1]
			return isLdr(lines[0]) && isStr(lines[1]) &&
				getLdrDest(lines[0]) == getStrSrc(lines[1]) &&
				getLdrAddr(lines[0]) == getStrAddr(lines[1])
		},
		Replace: func(lines []string) []string {
			// Remove redundant store
			return []string{lines[0]}
		},
	})

	// Pattern 3: Add-sub cancellation
	po.patterns = append(po.patterns, Pattern{
		Name: "add_sub_cancel",
		Match: func(lines []string) bool {
			if len(lines) < 2 {
				return false
			}
			// add x0, x0, #N followed by sub x0, x0, #N
			return isAdd(lines[0]) && isSub(lines[1]) &&
				getArithDest(lines[0]) == getArithDest(lines[1]) &&
				getArithSrc1(lines[0]) == getArithDest(lines[0]) &&
				getArithSrc1(lines[1]) == getArithDest(lines[1]) &&
				getArithImm(lines[0]) == getArithImm(lines[1])
		},
		Replace: func(lines []string) []string {
			// Remove both instructions
			return []string{}
		},
	})

	// Pattern 4: Strength reduction: mul by power of 2
	po.patterns = append(po.patterns, Pattern{
		Name: "mul_to_shift",
		Match: func(lines []string) bool {
			if len(lines) < 1 {
				return false
			}
			// mul with immediate power of 2
			return isMul(lines[0]) && hasPowerOf2Imm(lines[0])
		},
		Replace: func(lines []string) []string {
			// Replace with lsl (shift)
			return []string{convertMulToShift(lines[0])}
		},
	})

	// Pattern 5: Combine stp/ldp pairs
	po.patterns = append(po.patterns, Pattern{
		Name: "combine_stores",
		Match: func(lines []string) bool {
			if len(lines) < 2 {
				return false
			}
			// Two consecutive str that can be combined into stp
			return isStr(lines[0]) && isStr(lines[1]) &&
				canCombineToStp(lines[0], lines[1])
		},
		Replace: func(lines []string) []string {
			return []string{combineToStp(lines[0], lines[1])}
		},
	})

	// Pattern 6: Branch optimization
	po.patterns = append(po.patterns, Pattern{
		Name: "branch_to_next",
		Match: func(lines []string) bool {
			if len(lines) < 2 {
				return false
			}
			// Branch to immediately following label
			return isBranch(lines[0]) && isLabel(lines[1]) &&
				getBranchTarget(lines[0]) == getLabel(lines[1])
		},
		Replace: func(lines []string) []string {
			// Remove branch, keep label
			return []string{lines[1]}
		},
	})

	// Pattern 7: Comparison simplification
	po.patterns = append(po.patterns, Pattern{
		Name: "cmp_immediate_zero",
		Match: func(lines []string) bool {
			if len(lines) < 1 {
				return false
			}
			// cmp reg, #0 -> tst reg, reg (faster)
			return isCmp(lines[0]) && hasCmpImmediateZero(lines[0])
		},
		Replace: func(lines []string) []string {
			return []string{convertCmpToTst(lines[0])}
		},
	})

	// Pattern 8: Madd fusion
	po.patterns = append(po.patterns, Pattern{
		Name: "madd_fusion",
		Match: func(lines []string) bool {
			if len(lines) < 2 {
				return false
			}
			// mul x0, x1, x2 followed by add x0, x0, x3 -> madd
			return isMul(lines[0]) && isAdd(lines[1]) &&
				getMulDest(lines[0]) == getArithSrc1(lines[1]) &&
				getArithDest(lines[1]) == getArithSrc1(lines[1])
		},
		Replace: func(lines []string) []string {
			return []string{convertToMadd(lines[0], lines[1])}
		},
	})
}

// Helper functions for pattern matching

func isMov(line string) bool {
	return strings.Contains(strings.TrimSpace(line), "mov ")
}

func isLdr(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "ldr ")
}

func isStr(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "str ")
}

func isAdd(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "add ")
}

func isSub(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "sub ")
}

func isMul(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "mul ")
}

func isBranch(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "b ")
}

func isLabel(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasSuffix(trimmed, ":")
}

func isCmp(line string) bool {
	trimmed := strings.TrimSpace(line)
	return strings.HasPrefix(trimmed, "cmp ")
}

func getMovDest(line string) string {
	parts := strings.Fields(strings.TrimSpace(line))
	if len(parts) >= 2 {
		return strings.TrimSuffix(parts[1], ",")
	}
	return ""
}

func getMovSrc(line string) string {
	parts := strings.Fields(strings.TrimSpace(line))
	if len(parts) >= 3 {
		return parts[2]
	}
	return ""
}

func getLdrDest(line string) string {
	return getMovDest(line)
}

func getLdrAddr(line string) string {
	// Extract [xN] or [xN, #offset]
	start := strings.Index(line, "[")
	end := strings.Index(line, "]")
	if start >= 0 && end > start {
		return line[start : end+1]
	}
	return ""
}

func getStrSrc(line string) string {
	return getMovDest(line)
}

func getStrAddr(line string) string {
	return getLdrAddr(line)
}

func getArithDest(line string) string {
	return getMovDest(line)
}

func getArithSrc1(line string) string {
	parts := strings.Fields(strings.TrimSpace(line))
	if len(parts) >= 3 {
		return strings.TrimSuffix(parts[2], ",")
	}
	return ""
}

func getArithImm(line string) string {
	parts := strings.Fields(strings.TrimSpace(line))
	if len(parts) >= 4 {
		return parts[3]
	}
	return ""
}

func getBranchTarget(line string) string {
	parts := strings.Fields(strings.TrimSpace(line))
	if len(parts) >= 2 {
		return strings.TrimPrefix(parts[1], ".")
	}
	return ""
}

func getLabel(line string) string {
	return strings.TrimSuffix(strings.TrimSpace(line), ":")
}

func getMulDest(line string) string {
	return getArithDest(line)
}

func hasPowerOf2Imm(line string) bool {
	imm := getArithImm(line)
	// Check if immediate is power of 2
	if strings.HasPrefix(imm, "#") {
		// Simplified check - real implementation would parse and check
		return false // Conservative
	}
	return false
}

func hasCmpImmediateZero(line string) bool {
	return strings.Contains(line, "#0")
}

func canCombineToStp(line1, line2 string) bool {
	// Check if consecutive memory locations and registers
	// Simplified - real implementation would parse addresses
	return false // Conservative
}

func convertMulToShift(line string) string {
	// Convert mul to lsl (left shift)
	// Simplified placeholder
	return line
}

func combineToStp(line1, line2 string) string {
	// Combine two str into one stp
	// Simplified placeholder
	return line1
}

func convertCmpToTst(line string) string {
	// Convert cmp reg, #0 to tst reg, reg
	parts := strings.Fields(strings.TrimSpace(line))
	if len(parts) >= 2 {
		reg := strings.TrimSuffix(parts[1], ",")
		return "\ttst " + reg + ", " + reg
	}
	return line
}

func convertToMadd(mulLine, addLine string) string {
	// Convert mul+add to madd (multiply-add)
	// Simplified placeholder
	return mulLine
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
