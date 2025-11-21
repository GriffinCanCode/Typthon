// Package regalloc - Graph coloring register allocation
// Design: Chaitin-style graph coloring with coalescing
package regalloc

import (
	"github.com/GriffinCanCode/typthon-compiler/pkg/ir"
	"github.com/GriffinCanCode/typthon-compiler/pkg/logger"
	"github.com/GriffinCanCode/typthon-compiler/pkg/ssa"
)

// GraphAllocator performs graph coloring register allocation
// More sophisticated than linear scan, better for complex control flow
type GraphAllocator struct {
	fn            *ssa.Function
	cfg           *Config
	interferenceG *InterferenceGraph
	regMap        map[ir.Value]string
	spillMap      map[ir.Value]int
	nextSpillSlot int
	colorToReg    map[int]string
	regToColor    map[string]int
}

// InterferenceGraph represents variable interference
type InterferenceGraph struct {
	nodes map[ir.Value]*IGNode
	edges map[ir.Value]map[ir.Value]bool
}

// IGNode represents a node in the interference graph
type IGNode struct {
	value     ir.Value
	neighbors map[ir.Value]bool
	degree    int
	color     int // Register color (-1 if uncolored)
	spilled   bool
	coalesced *IGNode // Coalesced with this node
}

// NewGraphAllocator creates a graph coloring allocator
func NewGraphAllocator(fn *ssa.Function, cfg *Config) *GraphAllocator {
	ga := &GraphAllocator{
		fn:            fn,
		cfg:           cfg,
		interferenceG: newInterferenceGraph(),
		regMap:        make(map[ir.Value]string),
		spillMap:      make(map[ir.Value]int),
		nextSpillSlot: 0,
		colorToReg:    make(map[int]string),
		regToColor:    make(map[string]int),
	}

	// Map colors to registers
	for i, reg := range cfg.Available {
		ga.colorToReg[i] = reg
		ga.regToColor[reg] = i
	}

	return ga
}

// Allocate performs graph coloring allocation
func (ga *GraphAllocator) Allocate() error {
	logger.Debug("Starting graph coloring register allocation", "function", ga.fn.Name)

	// 1. Build interference graph from liveness analysis
	if err := ga.buildInterferenceGraph(); err != nil {
		return err
	}

	// 2. Coalesce move-related nodes
	ga.coalesce()

	// 3. Simplify: iteratively remove low-degree nodes
	simplified := ga.simplify()

	// 4. Select colors for nodes
	ga.select_colors(simplified)

	// 5. Assign registers based on colors
	ga.assignRegisters()

	logger.Debug("Graph coloring complete",
		"allocated", len(ga.regMap),
		"spilled", len(ga.spillMap))

	return nil
}

// buildInterferenceGraph constructs the interference graph
func (ga *GraphAllocator) buildInterferenceGraph() error {
	// Compute liveness for each block
	liveness := ga.computeLiveness()

	// Add nodes for all values
	for _, block := range ga.fn.Blocks {
		for _, inst := range block.Insts {
			if def := getDef(inst); def != nil {
				ga.interferenceG.addNode(def)
			}
			for _, use := range getUses(inst) {
				if _, ok := use.(*ir.Const); !ok {
					ga.interferenceG.addNode(use)
				}
			}
		}
	}

	// Add interference edges
	for _, block := range ga.fn.Blocks {
		liveOut := liveness[block]

		// Process instructions in reverse
		for i := len(block.Insts) - 1; i >= 0; i-- {
			inst := block.Insts[i]

			// Def interferes with everything live after it
			if def := getDef(inst); def != nil {
				for liveVal := range liveOut {
					if liveVal != def {
						ga.interferenceG.addEdge(def, liveVal)
					}
				}

				// Remove def from live set
				delete(liveOut, def)
			}

			// Add uses to live set
			for _, use := range getUses(inst) {
				if _, ok := use.(*ir.Const); !ok {
					liveOut[use] = true
				}
			}
		}
	}

	logger.Debug("Built interference graph",
		"nodes", len(ga.interferenceG.nodes),
		"edges", ga.interferenceG.edgeCount())

	return nil
}

// computeLiveness performs liveness analysis
func (ga *GraphAllocator) computeLiveness() map[*ssa.Block]map[ir.Value]bool {
	liveness := make(map[*ssa.Block]map[ir.Value]bool)

	// Initialize
	for _, block := range ga.fn.Blocks {
		liveness[block] = make(map[ir.Value]bool)
	}

	// Iterate until fixed point
	changed := true
	for changed {
		changed = false

		// Process blocks in reverse postorder
		for i := len(ga.fn.Blocks) - 1; i >= 0; i-- {
			block := ga.fn.Blocks[i]
			oldSize := len(liveness[block])

			// Union of successor liveIn sets
			for _, succ := range block.Succs {
				for val := range liveness[succ] {
					liveness[block][val] = true
				}
			}

			// Remove defs, add uses
			for j := len(block.Insts) - 1; j >= 0; j-- {
				inst := block.Insts[j]
				if def := getDef(inst); def != nil {
					delete(liveness[block], def)
				}
				for _, use := range getUses(inst) {
					if _, ok := use.(*ir.Const); !ok {
						liveness[block][use] = true
					}
				}
			}

			if len(liveness[block]) != oldSize {
				changed = true
			}
		}
	}

	return liveness
}

// coalesce merges move-related nodes when possible
func (ga *GraphAllocator) coalesce() {
	// Find move instructions (Load where src is not memory)
	for _, block := range ga.fn.Blocks {
		for _, inst := range block.Insts {
			if load, ok := inst.(*ir.Load); ok {
				// Check if this is a register-to-register move
				if _, isConst := load.Src.(*ir.Const); !isConst {
					ga.tryCoalesce(load.Dest, load.Src)
				}
			}
		}
	}
}

// tryCoalesce attempts to coalesce two nodes
func (ga *GraphAllocator) tryCoalesce(v1, v2 ir.Value) {
	n1 := ga.interferenceG.getNode(v1)
	n2 := ga.interferenceG.getNode(v2)

	if n1 == nil || n2 == nil {
		return
	}

	// Can't coalesce if they interfere
	if ga.interferenceG.interferes(v1, v2) {
		return
	}

	// Conservative coalescing: only if combined degree < k
	k := len(ga.cfg.Available)
	if n1.degree+n2.degree < k {
		ga.interferenceG.coalesceNodes(n1, n2)
		logger.Debug("Coalesced nodes", "v1", valStr(v1), "v2", valStr(v2))
	}
}

// simplify removes nodes and builds a stack
func (ga *GraphAllocator) simplify() []ir.Value {
	stack := make([]ir.Value, 0)
	k := len(ga.cfg.Available)
	remaining := make(map[ir.Value]bool)

	for val := range ga.interferenceG.nodes {
		remaining[val] = true
	}

	// Repeatedly remove nodes with degree < k
	for len(remaining) > 0 {
		// Find a node with degree < k
		var toRemove ir.Value
		for val := range remaining {
			node := ga.interferenceG.getNode(val)
			if node != nil && node.degree < k {
				toRemove = val
				break
			}
		}

		// If no such node, pick potential spill candidate
		if toRemove == nil {
			// Pick node with highest degree (most constrained)
			maxDegree := -1
			for val := range remaining {
				node := ga.interferenceG.getNode(val)
				if node != nil && node.degree > maxDegree {
					maxDegree = node.degree
					toRemove = val
				}
			}
		}

		if toRemove != nil {
			stack = append(stack, toRemove)
			delete(remaining, toRemove)

			// Decrease degree of neighbors
			node := ga.interferenceG.getNode(toRemove)
			if node != nil {
				for neighbor := range node.neighbors {
					if remaining[neighbor] {
						n := ga.interferenceG.getNode(neighbor)
						if n != nil {
							n.degree--
						}
					}
				}
			}
		}
	}

	return stack
}

// select_colors assigns colors to nodes from stack
func (ga *GraphAllocator) select_colors(stack []ir.Value) {
	k := len(ga.cfg.Available)

	// Pop from stack and color
	for i := len(stack) - 1; i >= 0; i-- {
		val := stack[i]
		node := ga.interferenceG.getNode(val)
		if node == nil {
			continue
		}

		// Find available colors
		usedColors := make(map[int]bool)
		for neighbor := range node.neighbors {
			n := ga.interferenceG.getNode(neighbor)
			if n != nil && n.color >= 0 {
				usedColors[n.color] = true
			}
		}

		// Assign first available color
		assigned := false
		for color := 0; color < k; color++ {
			if !usedColors[color] {
				node.color = color
				assigned = true
				logger.Debug("Colored node", "value", valStr(val), "color", color)
				break
			}
		}

		// If no color available, mark for spilling
		if !assigned {
			node.spilled = true
			logger.Debug("Spilling node", "value", valStr(val))
		}
	}
}

// assignRegisters maps colors to actual registers
func (ga *GraphAllocator) assignRegisters() {
	for val, node := range ga.interferenceG.nodes {
		if node.spilled {
			ga.spillMap[val] = ga.nextSpillSlot
			ga.nextSpillSlot += 8
		} else if node.color >= 0 {
			if reg, ok := ga.colorToReg[node.color]; ok {
				ga.regMap[val] = reg
			}
		}
	}
}

// Get methods for compatibility with linear scan interface

func (ga *GraphAllocator) GetRegister(val ir.Value) (string, bool) {
	reg, ok := ga.regMap[val]
	return reg, ok
}

func (ga *GraphAllocator) GetSpillSlot(val ir.Value) (int, bool) {
	slot, ok := ga.spillMap[val]
	return slot, ok
}

func (ga *GraphAllocator) GetStackSize() int {
	return ga.nextSpillSlot
}

func (ga *GraphAllocator) GetFunction() *ssa.Function {
	return ga.fn
}

// InterferenceGraph methods

func newInterferenceGraph() *InterferenceGraph {
	return &InterferenceGraph{
		nodes: make(map[ir.Value]*IGNode),
		edges: make(map[ir.Value]map[ir.Value]bool),
	}
}

func (ig *InterferenceGraph) addNode(val ir.Value) {
	if _, exists := ig.nodes[val]; !exists {
		ig.nodes[val] = &IGNode{
			value:     val,
			neighbors: make(map[ir.Value]bool),
			degree:    0,
			color:     -1,
			spilled:   false,
		}
		ig.edges[val] = make(map[ir.Value]bool)
	}
}

func (ig *InterferenceGraph) addEdge(v1, v2 ir.Value) {
	ig.addNode(v1)
	ig.addNode(v2)

	if !ig.edges[v1][v2] {
		ig.edges[v1][v2] = true
		ig.edges[v2][v1] = true
		ig.nodes[v1].neighbors[v2] = true
		ig.nodes[v2].neighbors[v1] = true
		ig.nodes[v1].degree++
		ig.nodes[v2].degree++
	}
}

func (ig *InterferenceGraph) interferes(v1, v2 ir.Value) bool {
	return ig.edges[v1][v2]
}

func (ig *InterferenceGraph) getNode(val ir.Value) *IGNode {
	return ig.nodes[val]
}

func (ig *InterferenceGraph) coalesceNodes(n1, n2 *IGNode) {
	// Merge n2 into n1
	for neighbor := range n2.neighbors {
		if neighbor != n1.value {
			n1.neighbors[neighbor] = true
			ig.edges[n1.value][neighbor] = true
			ig.edges[neighbor][n1.value] = true
		}
	}
	n2.coalesced = n1
}

func (ig *InterferenceGraph) edgeCount() int {
	count := 0
	for _, edges := range ig.edges {
		count += len(edges)
	}
	return count / 2 // Each edge counted twice
}

// AllocatorStrategy selects allocation strategy
type AllocatorStrategy string

const (
	LinearScan    AllocatorStrategy = "linear_scan"
	GraphColoring AllocatorStrategy = "graph_coloring"
)

// NewAllocatorWithStrategy creates allocator based on strategy
func NewAllocatorWithStrategy(fn *ssa.Function, cfg *Config, strategy AllocatorStrategy) interface{} {
	switch strategy {
	case GraphColoring:
		logger.Info("Using graph coloring register allocation")
		return NewGraphAllocator(fn, cfg)
	default:
		logger.Info("Using linear scan register allocation")
		return NewAllocator(fn, cfg)
	}
}
