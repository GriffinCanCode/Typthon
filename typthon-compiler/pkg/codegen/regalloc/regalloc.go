// Package regalloc implements linear scan register allocation with liveness intervals.
//
// Design: Fast linear scan algorithm with proper liveness analysis and spilling support.
// Based on Poletto & Sarkar's linear scan algorithm with improvements from Wimmer & Mössenböck.
package regalloc

import (
	"fmt"
	"sort"

	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// Interval represents the live range of a value
type Interval struct {
	Value ir.Value
	Start int // First instruction where value is defined
	End   int // Last instruction where value is used
	Reg   string
	Spill int // Stack offset if spilled (-1 if not spilled)
}

// Allocator performs linear scan register allocation
type Allocator struct {
	fn             *ssa.Function
	intervals      []*Interval
	active         []*Interval
	free           []string
	regMap         map[ir.Value]string
	spillMap       map[ir.Value]int
	nextSpillSlot  int
	instPositions  map[ir.Inst]int
	valuePositions map[ir.Value]int
	cfg            *Config
	callSites      []int // Positions of call instructions
}

// Config holds register allocation configuration for an architecture
type Config struct {
	Available   []string // Available registers for allocation
	Reserved    []string // Reserved registers (args, return, etc.)
	CalleeSaved []string // Callee-saved registers
	CallerSaved []string // Caller-saved registers
}

// isCallerSaved checks if a register is caller-saved
func (c *Config) isCallerSaved(reg string) bool {
	for _, r := range c.CallerSaved {
		if r == reg {
			return true
		}
	}
	return false
}

// isCalleeSaved checks if a register is callee-saved
func (c *Config) isCalleeSaved(reg string) bool {
	for _, r := range c.CalleeSaved {
		if r == reg {
			return true
		}
	}
	return false
}

// NewAllocator creates a new register allocator
func NewAllocator(fn *ssa.Function, cfg *Config) *Allocator {
	return &Allocator{
		fn:             fn,
		intervals:      make([]*Interval, 0),
		active:         make([]*Interval, 0),
		free:           append([]string{}, cfg.Available...),
		regMap:         make(map[ir.Value]string),
		spillMap:       make(map[ir.Value]int),
		nextSpillSlot:  0,
		instPositions:  make(map[ir.Inst]int),
		valuePositions: make(map[ir.Value]int),
		cfg:            cfg,
		callSites:      make([]int, 0),
	}
}

// Allocate performs register allocation
func (a *Allocator) Allocate() error {
	logger.Debug("Starting register allocation", "function", a.fn.Name)

	// Step 1: Number instructions
	a.numberInstructions()

	// Step 2: Compute liveness intervals
	if err := a.computeLiveness(); err != nil {
		return err
	}

	// Step 3: Sort intervals by start position
	sort.Slice(a.intervals, func(i, j int) bool {
		return a.intervals[i].Start < a.intervals[j].Start
	})

	logger.Debug("Computed liveness intervals", "count", len(a.intervals))

	// Step 4: Linear scan allocation
	for _, interval := range a.intervals {
		if err := a.allocateInterval(interval); err != nil {
			return err
		}
	}

	logger.Debug("Register allocation complete",
		"allocated", len(a.regMap),
		"spilled", len(a.spillMap))

	return nil
}

// numberInstructions assigns position numbers to all instructions
func (a *Allocator) numberInstructions() {
	pos := 0
	for _, block := range a.fn.Blocks {
		// Phi nodes come first in each block
		for range block.Phis {
			pos += 2 // Even numbers for definitions
		}
		for _, inst := range block.Insts {
			a.instPositions[inst] = pos
			// Track call sites
			if _, isCall := inst.(*ir.Call); isCall {
				a.callSites = append(a.callSites, pos)
			}
			pos += 2
		}
		// Terminator gets a position too
		pos += 2
	}
}

// computeLiveness computes live intervals for all values
func (a *Allocator) computeLiveness() error {
	// Build def-use chains
	defs := make(map[ir.Value]int)
	uses := make(map[ir.Value][]int)

	// Parameters are defined at position 0 (beginning of function)
	for _, param := range a.fn.Params {
		defs[param] = 0
		a.valuePositions[param] = 0
	}

	pos := 0
	for _, block := range a.fn.Blocks {
		// Process phi nodes
		for _, phi := range block.Phis {
			defs[phi.Dest] = pos
			a.valuePositions[phi.Dest] = pos
			pos += 2
		}

		// Process regular instructions
		for _, inst := range block.Insts {
			currentPos := a.instPositions[inst]

			// Record uses
			for _, val := range getUses(inst) {
				uses[val] = append(uses[val], currentPos)
			}

			// Record definitions
			if def := getDef(inst); def != nil {
				defs[def] = currentPos
				a.valuePositions[def] = currentPos
			}
		}

		// Process terminator uses
		if block.Term != nil {
			pos += 2
			for _, val := range getTermUses(block.Term) {
				uses[val] = append(uses[val], pos)
			}
		}
	}

	// Create intervals
	for val, defPos := range defs {
		// Skip constants - they don't need registers
		if _, isConst := val.(*ir.Const); isConst {
			continue
		}

		endPos := defPos
		if useList, ok := uses[val]; ok && len(useList) > 0 {
			// Find last use
			for _, usePos := range useList {
				if usePos > endPos {
					endPos = usePos
				}
			}
		}

		// Split intervals at call sites if value spans a call
		a.splitAtCalls(val, defPos, endPos, defs, uses)
	}

	return nil
}

// splitAtCalls splits intervals at call sites for values in caller-saved registers
func (a *Allocator) splitAtCalls(val ir.Value, start, end int, defs map[ir.Value]int, uses map[ir.Value][]int) {
	// Find all call sites this interval spans
	callsInRange := make([]int, 0)
	for _, callPos := range a.callSites {
		if callPos > start && callPos < end {
			callsInRange = append(callsInRange, callPos)
		}
	}

	// If no calls in range, create single interval
	if len(callsInRange) == 0 {
		interval := &Interval{
			Value: val,
			Start: start,
			End:   end,
			Spill: -1,
		}
		a.intervals = append(a.intervals, interval)
		return
	}

	// Split the interval at each call site
	currentStart := start
	for _, callPos := range callsInRange {
		// Create interval up to call
		interval := &Interval{
			Value: val,
			Start: currentStart,
			End:   callPos - 1,
			Spill: -1,
		}
		a.intervals = append(a.intervals, interval)

		// Start new interval after call
		currentStart = callPos + 1
	}

	// Create final interval after last call
	if currentStart <= end {
		interval := &Interval{
			Value: val,
			Start: currentStart,
			End:   end,
			Spill: -1,
		}
		a.intervals = append(a.intervals, interval)
	}
}

// allocateInterval allocates a register or spills an interval
func (a *Allocator) allocateInterval(interval *Interval) error {
	// Expire old intervals
	a.expireOldIntervals(interval)

	// Check if interval spans a call site
	spansCall := false
	for _, callPos := range a.callSites {
		if interval.Start < callPos && interval.End > callPos {
			spansCall = true
			break
		}
	}

	// Try to allocate a free register
	if len(a.free) > 0 {
		// Prefer callee-saved registers for intervals spanning calls
		reg := a.selectRegister(spansCall)
		if reg != "" {
			interval.Reg = reg
			a.regMap[interval.Value] = reg
			a.active = append(a.active, interval)
			a.sortActiveByEnd()
			logger.Debug("Allocated register", "value", valStr(interval.Value), "reg", reg, "spansCall", spansCall)
			return nil
		}
	}

	// No free registers - need to spill
	return a.spillAtInterval(interval)
}

// selectRegister chooses the best available register
func (a *Allocator) selectRegister(preferCalleeSaved bool) string {
	if len(a.free) == 0 {
		return ""
	}

	// If we prefer callee-saved and have one available, use it
	if preferCalleeSaved {
		for i, reg := range a.free {
			if a.cfg.isCalleeSaved(reg) {
				// Remove from free list
				a.free = append(a.free[:i], a.free[i+1:]...)
				return reg
			}
		}
	}

	// Otherwise, just take the last one
	reg := a.free[len(a.free)-1]
	a.free = a.free[:len(a.free)-1]
	return reg
}

// expireOldIntervals removes intervals that are no longer active
func (a *Allocator) expireOldIntervals(interval *Interval) {
	newActive := make([]*Interval, 0, len(a.active))
	for _, active := range a.active {
		if active.End >= interval.Start {
			newActive = append(newActive, active)
		} else {
			// This interval is dead, free its register
			a.free = append(a.free, active.Reg)
			logger.Debug("Freed register", "reg", active.Reg)
		}
	}
	a.active = newActive
}

// spillAtInterval spills either the current interval or an active one
func (a *Allocator) spillAtInterval(interval *Interval) error {
	// Find the interval that ends last
	spill := a.active[len(a.active)-1]

	if spill.End > interval.End {
		// Spill the active interval with longest lifetime
		interval.Reg = spill.Reg
		a.regMap[interval.Value] = spill.Reg

		// Spill the old interval
		spill.Spill = a.nextSpillSlot
		a.spillMap[spill.Value] = a.nextSpillSlot
		a.nextSpillSlot += 8 // 8 bytes per spill slot (64-bit)

		logger.Debug("Spilled interval", "value", valStr(spill.Value), "slot", spill.Spill)

		// Update active list
		a.active[len(a.active)-1] = interval
		a.sortActiveByEnd()
	} else {
		// Spill current interval
		interval.Spill = a.nextSpillSlot
		a.spillMap[interval.Value] = a.nextSpillSlot
		a.nextSpillSlot += 8

		logger.Debug("Spilled new interval", "value", valStr(interval.Value), "slot", interval.Spill)
	}

	return nil
}

// sortActiveByEnd sorts active intervals by end position
func (a *Allocator) sortActiveByEnd() {
	sort.Slice(a.active, func(i, j int) bool {
		return a.active[i].End < a.active[j].End
	})
}

// GetRegister returns the register assigned to a value
func (a *Allocator) GetRegister(val ir.Value) (string, bool) {
	reg, ok := a.regMap[val]
	return reg, ok
}

// GetSpillSlot returns the spill slot for a value
func (a *Allocator) GetSpillSlot(val ir.Value) (int, bool) {
	slot, ok := a.spillMap[val]
	return slot, ok
}

// GetStackSize returns total stack space needed for spills
func (a *Allocator) GetStackSize() int {
	return a.nextSpillSlot
}

// GetFunction returns the function being allocated
func (a *Allocator) GetFunction() *ssa.Function {
	return a.fn
}

// Helper functions to extract uses and defs from instructions

func getUses(inst ir.Inst) []ir.Value {
	var uses []ir.Value
	switch i := inst.(type) {
	case *ir.BinOp:
		if i.L != nil {
			uses = append(uses, i.L)
		}
		if i.R != nil {
			uses = append(uses, i.R)
		}
	case *ir.Call:
		uses = append(uses, i.Args...)
	case *ir.Load:
		if i.Src != nil {
			uses = append(uses, i.Src)
		}
	case *ir.Store:
		if i.Src != nil {
			uses = append(uses, i.Src)
		}
		if i.Dest != nil {
			uses = append(uses, i.Dest)
		}
	case *ir.GetAttr:
		if i.Obj != nil {
			uses = append(uses, i.Obj)
		}
	case *ir.SetAttr:
		if i.Obj != nil {
			uses = append(uses, i.Obj)
		}
		if i.Value != nil {
			uses = append(uses, i.Value)
		}
	case *ir.GetItem:
		if i.Obj != nil {
			uses = append(uses, i.Obj)
		}
		if i.Index != nil {
			uses = append(uses, i.Index)
		}
	case *ir.SetItem:
		if i.Obj != nil {
			uses = append(uses, i.Obj)
		}
		if i.Index != nil {
			uses = append(uses, i.Index)
		}
		if i.Value != nil {
			uses = append(uses, i.Value)
		}
	case *ir.MethodCall:
		if i.Obj != nil {
			uses = append(uses, i.Obj)
		}
		uses = append(uses, i.Args...)
	case *ir.ClosureCall:
		if i.Closure != nil {
			uses = append(uses, i.Closure)
		}
		uses = append(uses, i.Args...)
	case *ir.MakeClosure:
		uses = append(uses, i.Captures...)
	case *ir.Yield:
		if i.Value != nil {
			uses = append(uses, i.Value)
		}
	}
	return uses
}

func getDef(inst ir.Inst) ir.Value {
	switch i := inst.(type) {
	case *ir.BinOp:
		return i.Dest
	case *ir.Call:
		return i.Dest
	case *ir.Load:
		return i.Dest
	case *ir.Alloc:
		return i.Dest
	case *ir.AllocObject:
		return i.Dest
	case *ir.GetAttr:
		return i.Dest
	case *ir.GetItem:
		return i.Dest
	case *ir.MethodCall:
		return i.Dest
	case *ir.ClosureCall:
		return i.Dest
	case *ir.MakeClosure:
		return i.Dest
	}
	return nil
}

func getTermUses(term ir.Terminator) []ir.Value {
	var uses []ir.Value
	switch t := term.(type) {
	case *ir.Return:
		if t.Value != nil {
			uses = append(uses, t.Value)
		}
	case *ir.CondBranch:
		if t.Cond != nil {
			uses = append(uses, t.Cond)
		}
	}
	return uses
}

func valStr(val ir.Value) string {
	switch v := val.(type) {
	case *ir.Temp:
		return fmt.Sprintf("t%d", v.ID)
	case *ir.Param:
		return v.Name
	case *ir.Const:
		return fmt.Sprintf("%d", v.Val)
	default:
		return fmt.Sprintf("%T", val)
	}
}
