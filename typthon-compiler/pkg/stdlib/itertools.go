// Package stdlib - Itertools implementations
package stdlib

// Chain chains multiple iterators together
type Chain struct {
	Iterators [][]int64
	Current   int
	Index     int
}

// NewChain creates a new chain iterator
func NewChain(iterators ...[]int64) *Chain {
	return &Chain{Iterators: iterators}
}

// Next returns the next value and whether there are more values
func (c *Chain) Next() (int64, bool) {
	for c.Current < len(c.Iterators) {
		if c.Index < len(c.Iterators[c.Current]) {
			val := c.Iterators[c.Current][c.Index]
			c.Index++
			return val, true
		}
		c.Current++
		c.Index = 0
	}
	return 0, false
}

// Zip zips multiple sequences together
func Zip(seqs ...[]int64) [][]int64 {
	if len(seqs) == 0 {
		return nil
	}

	minLen := len(seqs[0])
	for _, seq := range seqs {
		if len(seq) < minLen {
			minLen = len(seq)
		}
	}

	result := make([][]int64, minLen)
	for i := 0; i < minLen; i++ {
		result[i] = make([]int64, len(seqs))
		for j, seq := range seqs {
			result[i][j] = seq[i]
		}
	}

	return result
}

// Enumerate returns pairs of (index, value)
type EnumerateIter struct {
	Seq   []int64
	Index int64
}

// NewEnumerate creates a new enumerate iterator
func NewEnumerate(seq []int64) *EnumerateIter {
	return &EnumerateIter{Seq: seq, Index: 0}
}

// Next returns the next (index, value) pair
func (e *EnumerateIter) Next() (int64, int64, bool) {
	if int(e.Index) < len(e.Seq) {
		idx := e.Index
		val := e.Seq[e.Index]
		e.Index++
		return idx, val, true
	}
	return 0, 0, false
}

// Filter filters elements based on a predicate
func Filter(seq []int64, pred func(int64) bool) []int64 {
	result := make([]int64, 0, len(seq))
	for _, val := range seq {
		if pred(val) {
			result = append(result, val)
		}
	}
	return result
}

// Map applies a function to each element
func Map(seq []int64, fn func(int64) int64) []int64 {
	result := make([]int64, len(seq))
	for i, val := range seq {
		result[i] = fn(val)
	}
	return result
}

// Reduce applies a binary operation cumulatively
func Reduce(seq []int64, fn func(int64, int64) int64, initial int64) int64 {
	result := initial
	for _, val := range seq {
		result = fn(result, val)
	}
	return result
}
