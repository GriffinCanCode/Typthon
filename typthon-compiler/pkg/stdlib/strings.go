// String operations and regex support
package stdlib

import (
	"regexp"
	"strings"
)

// String operations

// StrLen returns the length of a string
func StrLen(s string) int64 {
	return int64(len(s))
}

// StrConcat concatenates two strings
func StrConcat(a, b string) string {
	return a + b
}

// StrUpper converts string to uppercase
func StrUpper(s string) string {
	return strings.ToUpper(s)
}

// StrLower converts string to lowercase
func StrLower(s string) string {
	return strings.ToLower(s)
}

// StrStrip removes leading and trailing whitespace
func StrStrip(s string) string {
	return strings.TrimSpace(s)
}

// StrSplit splits string by separator
func StrSplit(s, sep string) []string {
	return strings.Split(s, sep)
}

// StrJoin joins strings with separator
func StrJoin(parts []string, sep string) string {
	return strings.Join(parts, sep)
}

// StrReplace replaces all occurrences of old with new
func StrReplace(s, old, new string) string {
	return strings.ReplaceAll(s, old, new)
}

// StrContains checks if string contains substring
func StrContains(s, substr string) bool {
	return strings.Contains(s, substr)
}

// StrStartsWith checks if string starts with prefix
func StrStartsWith(s, prefix string) bool {
	return strings.HasPrefix(s, prefix)
}

// StrEndsWith checks if string ends with suffix
func StrEndsWith(s, suffix string) bool {
	return strings.HasSuffix(s, suffix)
}

// StrFind returns index of first occurrence of substring (-1 if not found)
func StrFind(s, substr string) int64 {
	idx := strings.Index(s, substr)
	return int64(idx)
}

// StrCount counts non-overlapping occurrences of substring
func StrCount(s, substr string) int64 {
	return int64(strings.Count(s, substr))
}

// Regex operations

// RegexMatch checks if string matches pattern
func RegexMatch(pattern, s string) bool {
	matched, err := regexp.MatchString(pattern, s)
	if err != nil {
		return false
	}
	return matched
}

// RegexFindAll finds all matches of pattern in string
func RegexFindAll(pattern, s string) []string {
	re, err := regexp.Compile(pattern)
	if err != nil {
		return nil
	}
	return re.FindAllString(s, -1)
}

// RegexReplace replaces all matches with replacement
func RegexReplace(pattern, s, replacement string) string {
	re, err := regexp.Compile(pattern)
	if err != nil {
		return s
	}
	return re.ReplaceAllString(s, replacement)
}

// RegexSplit splits string by regex pattern
func RegexSplit(pattern, s string) []string {
	re, err := regexp.Compile(pattern)
	if err != nil {
		return []string{s}
	}
	return re.Split(s, -1)
}

// Format operations

// StrFormat formats string with arguments (simple %s substitution)
func StrFormat(format string, args ...interface{}) string {
	// Simple implementation - full printf-style formatting would be more complex
	result := format
	for _, arg := range args {
		// Replace first occurrence of %s
		result = strings.Replace(result, "%s", toString(arg), 1)
	}
	return result
}

func toString(v interface{}) string {
	switch val := v.(type) {
	case string:
		return val
	case int64:
		return intToString(val)
	case bool:
		if val {
			return "true"
		}
		return "false"
	default:
		return ""
	}
}

func intToString(n int64) string {
	if n == 0 {
		return "0"
	}

	negative := n < 0
	if negative {
		n = -n
	}

	var digits []byte
	for n > 0 {
		digits = append([]byte{byte('0' + n%10)}, digits...)
		n /= 10
	}

	if negative {
		digits = append([]byte{'-'}, digits...)
	}

	return string(digits)
}
