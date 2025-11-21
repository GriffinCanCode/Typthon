// Pattern matching implementation
package ir

import (
	"fmt"

	"github.com/GriffinCanCode/typthon-compiler/pkg/frontend"
)

func (b *Builder) buildMatch(match *frontend.Match) error {
	// Evaluate subject expression
	subject, err := b.buildExpression(match.Subject)
	if err != nil {
		return err
	}

	// Create blocks for each case and exit
	exitBlock := b.newBlock("match_exit")
	var irCases []MatchCase

	// Build each case
	for _, c := range match.Cases {
		caseBlock := b.newBlock("match_case")

		// Convert pattern
		irPattern, err := b.buildPattern(c.Pattern)
		if err != nil {
			return err
		}

		// Convert guard if present
		var guard Value
		if c.Guard != nil {
			guard, err = b.buildExpression(c.Guard)
			if err != nil {
				return err
			}
		}

		irCases = append(irCases, MatchCase{
			Pattern:     irPattern,
			Guard:       guard,
			TargetBlock: caseBlock.Label,
		})

		// Build case body
		b.currentFn.Blocks = append(b.currentFn.Blocks, caseBlock)
		prevBlock := b.currentBl
		b.currentBl = caseBlock

		for _, stmt := range c.Body {
			if err := b.buildStatement(stmt); err != nil {
				return err
			}
		}

		// Jump to exit if no terminator
		if b.currentBl.Term == nil {
			b.currentBl.Term = &Branch{Target: exitBlock.Label}
		}

		b.currentBl = prevBlock
	}

	// Add match instruction
	b.currentBl.Insts = append(b.currentBl.Insts, &MatchJump{
		Subject: subject,
		Cases:   irCases,
	})

	// Jump to exit (in case no pattern matches)
	b.currentBl.Term = &Branch{Target: exitBlock.Label}

	// Set exit block as current
	b.currentFn.Blocks = append(b.currentFn.Blocks, exitBlock)
	b.currentBl = exitBlock

	return nil
}

func (b *Builder) buildPattern(pattern frontend.Pattern) (Pattern, error) {
	switch p := pattern.(type) {
	case *frontend.LiteralPattern:
		val, err := b.buildExpression(p.Value)
		if err != nil {
			return nil, err
		}
		return &LiteralPattern{Value: val}, nil

	case *frontend.CapturePattern:
		return &CapturePattern{Name: p.Name}, nil

	case *frontend.OrPattern:
		var patterns []Pattern
		for _, subp := range p.Patterns {
			irp, err := b.buildPattern(subp)
			if err != nil {
				return nil, err
			}
			patterns = append(patterns, irp)
		}
		return &OrPattern{Patterns: patterns}, nil

	case *frontend.ClassPattern:
		var args []Pattern
		for _, arg := range p.Args {
			irp, err := b.buildPattern(arg)
			if err != nil {
				return nil, err
			}
			args = append(args, irp)
		}
		return &ClassPattern{ClassName: p.Class, Args: args}, nil

	default:
		return nil, fmt.Errorf("unsupported pattern type: %T", pattern)
	}
}
