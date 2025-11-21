// Regex module - comprehensive pattern matching with caching
package stdlib

import (
	"regexp"
	"sync"
)

// RegexCache caches compiled patterns for performance
var (
	regexCache = make(map[string]*regexp.Regexp)
	cacheMutex sync.RWMutex
)

// Regex represents a compiled regular expression
type Regex struct {
	pattern string
	re      *regexp.Regexp
}

// RegexCompile compiles a regex pattern with caching
func RegexCompile(pattern string) *Regex {
	cacheMutex.RLock()
	if re, exists := regexCache[pattern]; exists {
		cacheMutex.RUnlock()
		return &Regex{pattern: pattern, re: re}
	}
	cacheMutex.RUnlock()

	re, err := regexp.Compile(pattern)
	if err != nil {
		return nil
	}

	cacheMutex.Lock()
	regexCache[pattern] = re
	cacheMutex.Unlock()

	return &Regex{pattern: pattern, re: re}
}

// RegexMatch tests if pattern matches string
func RegexMatch(pattern, text string) bool {
	re := RegexCompile(pattern)
	if re == nil {
		return false
	}
	return re.re.MatchString(text)
}

// RegexSearch finds first match in string
func RegexSearch(pattern, text string) (string, bool) {
	re := RegexCompile(pattern)
	if re == nil {
		return "", false
	}
	match := re.re.FindString(text)
	return match, match != ""
}

// RegexFindAll finds all matches in string
func RegexFindAll(pattern, text string) []string {
	re := RegexCompile(pattern)
	if re == nil {
		return nil
	}
	return re.re.FindAllString(text, -1)
}

// RegexFindGroups finds first match with capturing groups
func RegexFindGroups(pattern, text string) []string {
	re := RegexCompile(pattern)
	if re == nil {
		return nil
	}
	return re.re.FindStringSubmatch(text)
}

// RegexFindAllGroups finds all matches with capturing groups
func RegexFindAllGroups(pattern, text string) [][]string {
	re := RegexCompile(pattern)
	if re == nil {
		return nil
	}
	return re.re.FindAllStringSubmatch(text, -1)
}

// RegexReplace replaces all matches with replacement
func RegexReplace(pattern, text, repl string) string {
	re := RegexCompile(pattern)
	if re == nil {
		return text
	}
	return re.re.ReplaceAllString(text, repl)
}

// RegexReplaceN replaces first n matches
func RegexReplaceN(pattern, text, repl string, n int64) string {
	re := RegexCompile(pattern)
	if re == nil {
		return text
	}

	matches := re.re.FindAllStringIndex(text, int(n))
	if matches == nil {
		return text
	}

	result := text
	offset := 0
	for _, match := range matches {
		start, end := match[0]+offset, match[1]+offset
		result = result[:start] + repl + result[end:]
		offset += len(repl) - (end - start)
	}
	return result
}

// RegexSplit splits string by pattern
func RegexSplit(pattern, text string) []string {
	re := RegexCompile(pattern)
	if re == nil {
		return []string{text}
	}
	return re.re.Split(text, -1)
}

// RegexSplitN splits string by pattern with max n splits
func RegexSplitN(pattern, text string, n int64) []string {
	re := RegexCompile(pattern)
	if re == nil {
		return []string{text}
	}
	return re.re.Split(text, int(n))
}

// RegexFindIndex finds index of first match
func RegexFindIndex(pattern, text string) (int64, int64, bool) {
	re := RegexCompile(pattern)
	if re == nil {
		return -1, -1, false
	}
	loc := re.re.FindStringIndex(text)
	if loc == nil {
		return -1, -1, false
	}
	return int64(loc[0]), int64(loc[1]), true
}

// RegexFindAllIndex finds indices of all matches
func RegexFindAllIndex(pattern, text string) [][]int64 {
	re := RegexCompile(pattern)
	if re == nil {
		return nil
	}
	locs := re.re.FindAllStringIndex(text, -1)
	if locs == nil {
		return nil
	}

	result := make([][]int64, len(locs))
	for i, loc := range locs {
		result[i] = []int64{int64(loc[0]), int64(loc[1])}
	}
	return result
}

// RegexGroupNames returns names of capturing groups
func RegexGroupNames(pattern string) []string {
	re := RegexCompile(pattern)
	if re == nil {
		return nil
	}
	return re.re.SubexpNames()
}

// RegexNamedGroups finds match with named groups as map
func RegexNamedGroups(pattern, text string) (map[string]string, bool) {
	re := RegexCompile(pattern)
	if re == nil {
		return nil, false
	}

	match := re.re.FindStringSubmatch(text)
	if match == nil {
		return nil, false
	}

	result := make(map[string]string)
	for i, name := range re.re.SubexpNames() {
		if i > 0 && i < len(match) && name != "" {
			result[name] = match[i]
		}
	}
	return result, len(result) > 0
}

// RegexEscape escapes special regex characters
func RegexEscape(text string) string {
	return regexp.QuoteMeta(text)
}

// Methods for Regex objects

// Match tests if regex matches string
func (r *Regex) Match(text string) bool {
	return r.re.MatchString(text)
}

// Search finds first match
func (r *Regex) Search(text string) (string, bool) {
	match := r.re.FindString(text)
	return match, match != ""
}

// FindAll finds all matches
func (r *Regex) FindAll(text string) []string {
	return r.re.FindAllString(text, -1)
}

// Replace replaces all matches
func (r *Regex) Replace(text, repl string) string {
	return r.re.ReplaceAllString(text, repl)
}

// Split splits by pattern
func (r *Regex) Split(text string) []string {
	return r.re.Split(text, -1)
}
