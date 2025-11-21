// Package stdlib - Standard library implementations
// Design: Native implementations of Python stdlib for compiled code
package stdlib

// Collections - core collection utilities

// Range represents a Python range object
type Range struct {
	Start int64
	Stop  int64
	Step  int64
}

// NewRange creates a new range
func NewRange(stop int64) *Range {
	return &Range{Start: 0, Stop: stop, Step: 1}
}

// NewRangeStartStop creates a new range with start and stop
func NewRangeStartStop(start, stop int64) *Range {
	return &Range{Start: start, Stop: stop, Step: 1}
}

// NewRangeStartStopStep creates a new range with start, stop, and step
func NewRangeStartStopStep(start, stop, step int64) *Range {
	return &Range{Start: start, Stop: stop, Step: step}
}

// Len returns the length of the range
func (r *Range) Len() int64 {
	if r.Step > 0 {
		if r.Stop <= r.Start {
			return 0
		}
		return (r.Stop - r.Start + r.Step - 1) / r.Step
	}
	if r.Stop >= r.Start {
		return 0
	}
	return (r.Start - r.Stop - r.Step - 1) / (-r.Step)
}

// At returns the value at index i
func (r *Range) At(i int64) int64 {
	return r.Start + i*r.Step
}
