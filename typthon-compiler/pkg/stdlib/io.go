// File I/O operations
package stdlib

import (
	"bufio"
	"os"
)

// File represents an open file handle
type File struct {
	handle *os.File
	reader *bufio.Reader
	writer *bufio.Writer
}

// FileOpen opens a file with specified mode
// Modes: "r" (read), "w" (write), "a" (append), "r+" (read/write)
func FileOpen(path, mode string) *File {
	var f *os.File
	var err error

	switch mode {
	case "r":
		f, err = os.Open(path)
	case "w":
		f, err = os.Create(path)
	case "a":
		f, err = os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	case "r+":
		f, err = os.OpenFile(path, os.O_RDWR, 0644)
	default:
		return nil
	}

	if err != nil {
		return nil
	}

	file := &File{handle: f}

	// Initialize reader/writer based on mode
	if mode == "r" || mode == "r+" {
		file.reader = bufio.NewReader(f)
	}
	if mode == "w" || mode == "a" || mode == "r+" {
		file.writer = bufio.NewWriter(f)
	}

	return file
}

// FileClose closes the file
func FileClose(f *File) bool {
	if f == nil || f.handle == nil {
		return false
	}

	// Flush writer if present
	if f.writer != nil {
		f.writer.Flush()
	}

	err := f.handle.Close()
	return err == nil
}

// FileRead reads entire file contents
func FileRead(f *File) string {
	if f == nil || f.reader == nil {
		return ""
	}

	content, err := os.ReadFile(f.handle.Name())
	if err != nil {
		return ""
	}

	return string(content)
}

// FileReadLine reads one line from file
func FileReadLine(f *File) string {
	if f == nil || f.reader == nil {
		return ""
	}

	line, err := f.reader.ReadString('\n')
	if err != nil {
		return ""
	}

	return line
}

// FileReadLines reads all lines from file
func FileReadLines(f *File) []string {
	if f == nil || f.reader == nil {
		return nil
	}

	var lines []string
	for {
		line, err := f.reader.ReadString('\n')
		if err != nil {
			break
		}
		lines = append(lines, line)
	}

	return lines
}

// FileWrite writes string to file
func FileWrite(f *File, data string) bool {
	if f == nil || f.writer == nil {
		return false
	}

	_, err := f.writer.WriteString(data)
	if err != nil {
		return false
	}

	return true
}

// FileWriteLines writes multiple lines to file
func FileWriteLines(f *File, lines []string) bool {
	if f == nil || f.writer == nil {
		return false
	}

	for _, line := range lines {
		if _, err := f.writer.WriteString(line + "\n"); err != nil {
			return false
		}
	}

	return true
}

// FileFlush flushes buffered writes to disk
func FileFlush(f *File) bool {
	if f == nil || f.writer == nil {
		return false
	}

	err := f.writer.Flush()
	return err == nil
}

// File system operations

// FileExists checks if file exists
func FileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

// FileRemove deletes a file
func FileRemove(path string) bool {
	err := os.Remove(path)
	return err == nil
}

// FileRename renames/moves a file
func FileRename(oldPath, newPath string) bool {
	err := os.Rename(oldPath, newPath)
	return err == nil
}

// DirCreate creates a directory
func DirCreate(path string) bool {
	err := os.Mkdir(path, 0755)
	return err == nil
}

// DirCreateAll creates directory and all parents
func DirCreateAll(path string) bool {
	err := os.MkdirAll(path, 0755)
	return err == nil
}

// DirRemove removes a directory
func DirRemove(path string) bool {
	err := os.Remove(path)
	return err == nil
}

// DirRemoveAll removes directory and all contents
func DirRemoveAll(path string) bool {
	err := os.RemoveAll(path)
	return err == nil
}

// DirList lists files in directory
func DirList(path string) []string {
	entries, err := os.ReadDir(path)
	if err != nil {
		return nil
	}

	var names []string
	for _, entry := range entries {
		names = append(names, entry.Name())
	}

	return names
}
