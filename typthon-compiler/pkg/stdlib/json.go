// JSON parsing and serialization
package stdlib

import (
	"encoding/json"
)

// JSONValue represents a generic JSON value
type JSONValue interface {
	jsonValue()
}

// JSON types

type JSONNull struct{}

func (JSONNull) jsonValue() {}

type JSONBool struct {
	Value bool
}

func (JSONBool) jsonValue() {}

type JSONNumber struct {
	Value float64
}

func (JSONNumber) jsonValue() {}

type JSONString struct {
	Value string
}

func (JSONString) jsonValue() {}

type JSONArray struct {
	Values []JSONValue
}

func (JSONArray) jsonValue() {}

type JSONObject struct {
	Fields map[string]JSONValue
}

func (JSONObject) jsonValue() {}

// JSONParse parses JSON string into native Go value
func JSONParse(s string) (interface{}, bool) {
	var result interface{}
	err := json.Unmarshal([]byte(s), &result)
	if err != nil {
		return nil, false
	}
	return result, true
}

// JSONStringify converts value to JSON string
func JSONStringify(v interface{}) (string, bool) {
	bytes, err := json.Marshal(v)
	if err != nil {
		return "", false
	}
	return string(bytes), true
}

// JSONStringifyPretty converts value to pretty-printed JSON
func JSONStringifyPretty(v interface{}) (string, bool) {
	bytes, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		return "", false
	}
	return string(bytes), true
}

// Typed JSON operations

// JSONParseObject parses JSON object
func JSONParseObject(s string) (map[string]interface{}, bool) {
	var result map[string]interface{}
	err := json.Unmarshal([]byte(s), &result)
	if err != nil {
		return nil, false
	}
	return result, true
}

// JSONParseArray parses JSON array
func JSONParseArray(s string) ([]interface{}, bool) {
	var result []interface{}
	err := json.Unmarshal([]byte(s), &result)
	if err != nil {
		return nil, false
	}
	return result, true
}

// JSON builder API

// JSONObjectNew creates a new JSON object
func JSONObjectNew() map[string]interface{} {
	return make(map[string]interface{})
}

// JSONObjectSet sets a field in JSON object
func JSONObjectSet(obj map[string]interface{}, key string, value interface{}) {
	obj[key] = value
}

// JSONObjectGet gets a field from JSON object
func JSONObjectGet(obj map[string]interface{}, key string) (interface{}, bool) {
	val, ok := obj[key]
	return val, ok
}

// JSONObjectHas checks if object has a field
func JSONObjectHas(obj map[string]interface{}, key string) bool {
	_, ok := obj[key]
	return ok
}

// JSONObjectKeys returns all keys in object
func JSONObjectKeys(obj map[string]interface{}) []string {
	keys := make([]string, 0, len(obj))
	for k := range obj {
		keys = append(keys, k)
	}
	return keys
}

// JSONArrayNew creates a new JSON array
func JSONArrayNew() []interface{} {
	return make([]interface{}, 0)
}

// JSONArrayAppend appends value to array
func JSONArrayAppend(arr []interface{}, value interface{}) []interface{} {
	return append(arr, value)
}

// JSONArrayGet gets value at index
func JSONArrayGet(arr []interface{}, index int64) (interface{}, bool) {
	if index < 0 || index >= int64(len(arr)) {
		return nil, false
	}
	return arr[index], true
}

// JSONArrayLen returns length of array
func JSONArrayLen(arr []interface{}) int64 {
	return int64(len(arr))
}
