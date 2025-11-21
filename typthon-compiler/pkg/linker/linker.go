// Package linker handles object file generation and linking.
//
// Design: Emit ELF/Mach-O/PE, link with system linker or custom linking.
// Static linking by default for standalone binaries.
package linker

import (
	"os/exec"
)

// Linker links object files into executables
type Linker struct {
	target  string
	objects []string
	output  string
	runtime string
}

func New(target, output, runtime string) *Linker {
	return &Linker{
		target:  target,
		output:  output,
		runtime: runtime,
	}
}

func (l *Linker) AddObject(path string) {
	l.objects = append(l.objects, path)
}

// Link produces final executable
func (l *Linker) Link() error {
	// Use system linker for now (ld, lld, etc.)
	// TODO: Custom linker for faster linking

	var linker string
	switch l.target {
	case "darwin":
		linker = "ld"
	case "linux":
		linker = "ld.lld"
	default:
		linker = "ld"
	}

	args := []string{
		"-o", l.output,
		"-static", // Static linking by default
	}
	args = append(args, l.objects...)
	args = append(args, l.runtime)

	cmd := exec.Command(linker, args...)
	return cmd.Run()
}

// Emit generates object file from assembly
func EmitObject(asmPath, objPath string) error {
	// Use system assembler (as, nasm, etc.)
	cmd := exec.Command("as", "-o", objPath, asmPath)
	return cmd.Run()
}
