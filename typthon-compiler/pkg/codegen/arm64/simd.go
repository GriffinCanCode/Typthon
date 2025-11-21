// Package arm64 - NEON SIMD instruction generation
// Design: Vectorize arithmetic operations for Apple Silicon optimization
package arm64

import (
	"fmt"
	"io"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
)

// NeonOp represents a NEON vector operation
type NeonOp string

const (
	NeonAdd  NeonOp = "add"
	NeonSub  NeonOp = "sub"
	NeonMul  NeonOp = "mul"
	NeonFadd NeonOp = "fadd"
	NeonFsub NeonOp = "fsub"
	NeonFmul NeonOp = "fmul"
)

// VectorWidth represents NEON register width
type VectorWidth int

const (
	V128 VectorWidth = 128 // 128-bit vectors (4x32 or 2x64)
	V64  VectorWidth = 64  // 64-bit vectors (2x32 or 1x64)
)

// NeonGen generates NEON SIMD instructions
type NeonGen struct {
	w io.Writer
}

// NewNeonGen creates a NEON instruction generator
func NewNeonGen(w io.Writer) *NeonGen {
	return &NeonGen{w: w}
}

// EmitVectorOp emits a NEON vector operation
// Uses v registers (v0-v31) for SIMD operations
func (n *NeonGen) EmitVectorOp(op NeonOp, dest, src1, src2 string, width VectorWidth) {
	suffix := n.getSuffix(width)
	fmt.Fprintf(n.w, "\t%s %s.%s, %s.%s, %s.%s\n", op, dest, suffix, src1, suffix, src2, suffix)
	logger.Debug("Emitted NEON instruction", "op", op, "width", width)
}

// EmitVectorLoad loads data into NEON register
func (n *NeonGen) EmitVectorLoad(dest, srcAddr string, width VectorWidth) {
	inst := "ld1"
	suffix := n.getSuffix(width)
	fmt.Fprintf(n.w, "\t%s {%s.%s}, [%s]\n", inst, dest, suffix, srcAddr)
}

// EmitVectorStore stores NEON register to memory
func (n *NeonGen) EmitVectorStore(src, destAddr string, width VectorWidth) {
	inst := "st1"
	suffix := n.getSuffix(width)
	fmt.Fprintf(n.w, "\t%s {%s.%s}, [%s]\n", inst, src, suffix, destAddr)
}

// EmitVectorLoadMultiple loads multiple consecutive registers
func (n *NeonGen) EmitVectorLoadMultiple(regs []string, addr string, width VectorWidth) {
	if len(regs) == 0 {
		return
	}
	suffix := n.getSuffix(width)
	regList := n.formatRegList(regs, suffix)
	fmt.Fprintf(n.w, "\tld1 {%s}, [%s]\n", regList, addr)
}

// EmitVectorStoreMultiple stores multiple consecutive registers
func (n *NeonGen) EmitVectorStoreMultiple(regs []string, addr string, width VectorWidth) {
	if len(regs) == 0 {
		return
	}
	suffix := n.getSuffix(width)
	regList := n.formatRegList(regs, suffix)
	fmt.Fprintf(n.w, "\tst1 {%s}, [%s]\n", regList, addr)
}

// EmitVectorDup duplicates scalar to all lanes
func (n *NeonGen) EmitVectorDup(dest, scalar string, width VectorWidth) {
	suffix := n.getSuffix(width)
	fmt.Fprintf(n.w, "\tdup %s.%s, %s\n", dest, suffix, scalar)
}

// EmitVectorMLA emits multiply-accumulate (dest = dest + src1 * src2)
func (n *NeonGen) EmitVectorMLA(dest, src1, src2 string, width VectorWidth) {
	suffix := n.getSuffix(width)
	fmt.Fprintf(n.w, "\tmla %s.%s, %s.%s, %s.%s\n", dest, suffix, src1, suffix, src2, suffix)
}

// EmitVectorMLS emits multiply-subtract (dest = dest - src1 * src2)
func (n *NeonGen) EmitVectorMLS(dest, src1, src2 string, width VectorWidth) {
	suffix := n.getSuffix(width)
	fmt.Fprintf(n.w, "\tmls %s.%s, %s.%s, %s.%s\n", dest, suffix, src1, suffix, src2, suffix)
}

// EmitVectorCompare emits vector comparison
func (n *NeonGen) EmitVectorCompare(dest, src1, src2 string, cond CompareCondition, width VectorWidth) {
	suffix := n.getSuffix(width)
	inst := n.getCompareInst(cond)
	fmt.Fprintf(n.w, "\t%s %s.%s, %s.%s, %s.%s\n", inst, dest, suffix, src1, suffix, src2, suffix)
}

// CompareCondition represents NEON comparison types
type CompareCondition int

const (
	CmpEQ CompareCondition = iota // Equal
	CmpGT                         // Greater than
	CmpGE                         // Greater than or equal
	CmpLT                         // Less than
	CmpLE                         // Less than or equal
)

// TryVectorize attempts to vectorize a sequence of scalar operations
func TryVectorize(ops []*ir.BinOp) (vectorizable bool, vectorOps []*VectorOp) {
	if len(ops) < 4 {
		return false, nil // Need at least 4 ops for 128-bit vectorization
	}

	// Check if all ops are same type and vectorizable
	firstOp := ops[0].Op
	for _, op := range ops {
		if op.Op != firstOp || !isVectorizableOp(firstOp) {
			return false, nil
		}
	}

	// Group into vector lanes
	vectorOps = groupIntoVectors(ops, 4) // 4 operations per 128-bit vector
	return true, vectorOps
}

// VectorOp represents a vectorized operation
type VectorOp struct {
	Op      ir.Op
	Dests   []ir.Value
	Lefts   []ir.Value
	Rights  []ir.Value
	VecDest string
	VecL    string
	VecR    string
}

func isVectorizableOp(op ir.Op) bool {
	switch op {
	case ir.OpAdd, ir.OpSub, ir.OpMul:
		return true
	}
	return false
}

func groupIntoVectors(ops []*ir.BinOp, lanesPerVec int) []*VectorOp {
	result := make([]*VectorOp, 0)
	for i := 0; i+lanesPerVec <= len(ops); i += lanesPerVec {
		vop := &VectorOp{
			Op:     ops[i].Op,
			Dests:  make([]ir.Value, lanesPerVec),
			Lefts:  make([]ir.Value, lanesPerVec),
			Rights: make([]ir.Value, lanesPerVec),
		}
		for j := 0; j < lanesPerVec; j++ {
			vop.Dests[j] = ops[i+j].Dest
			vop.Lefts[j] = ops[i+j].L
			vop.Rights[j] = ops[i+j].R
		}
		result = append(result, vop)
	}
	return result
}

// getSuffix returns NEON instruction suffix for width
func (n *NeonGen) getSuffix(width VectorWidth) string {
	switch width {
	case V128:
		return "4s" // 4x 32-bit integers
	case V64:
		return "2s" // 2x 32-bit integers
	default:
		return "4s"
	}
}

// formatRegList formats register list for ld1/st1
func (n *NeonGen) formatRegList(regs []string, suffix string) string {
	result := ""
	for i, reg := range regs {
		if i > 0 {
			result += ", "
		}
		result += fmt.Sprintf("%s.%s", reg, suffix)
	}
	return result
}

// getCompareInst returns NEON comparison instruction
func (n *NeonGen) getCompareInst(cond CompareCondition) string {
	switch cond {
	case CmpEQ:
		return "cmeq"
	case CmpGT:
		return "cmgt"
	case CmpGE:
		return "cmge"
	case CmpLT:
		return "cmlt"
	case CmpLE:
		return "cmle"
	default:
		return "cmeq"
	}
}

// Convenience functions for common operations

// EmitVectorAddInt emits integer vector addition (4x 32-bit)
func (n *NeonGen) EmitVectorAddInt(dest, src1, src2 string) {
	n.EmitVectorOp(NeonAdd, dest, src1, src2, V128)
}

// EmitVectorSubInt emits integer vector subtraction (4x 32-bit)
func (n *NeonGen) EmitVectorSubInt(dest, src1, src2 string) {
	n.EmitVectorOp(NeonSub, dest, src1, src2, V128)
}

// EmitVectorMulInt emits integer vector multiplication (4x 32-bit)
func (n *NeonGen) EmitVectorMulInt(dest, src1, src2 string) {
	n.EmitVectorOp(NeonMul, dest, src1, src2, V128)
}

// EmitVectorAddFloat emits floating-point vector addition
func (n *NeonGen) EmitVectorAddFloat(dest, src1, src2 string) {
	n.EmitVectorOp(NeonFadd, dest, src1, src2, V128)
}

// EmitVectorSubFloat emits floating-point vector subtraction
func (n *NeonGen) EmitVectorSubFloat(dest, src1, src2 string) {
	n.EmitVectorOp(NeonFsub, dest, src1, src2, V128)
}

// EmitVectorMulFloat emits floating-point vector multiplication
func (n *NeonGen) EmitVectorMulFloat(dest, src1, src2 string) {
	n.EmitVectorOp(NeonFmul, dest, src1, src2, V128)
}
