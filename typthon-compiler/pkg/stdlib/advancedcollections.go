// Advanced collections - OrderedDict, defaultdict, Counter
package stdlib

import (
	"sort"
	"sync"
)

// OrderedDict maintains insertion order
type OrderedDict struct {
	keys   []string
	values map[string]interface{}
	mu     sync.RWMutex
}

// defaultdict provides default values for missing keys
type DefaultDict struct {
	data    map[string]interface{}
	factory func() interface{}
	mu      sync.RWMutex
}

// Counter counts hashable objects
type Counter struct {
	counts map[string]int64
	mu     sync.RWMutex
}

// Deque is a double-ended queue
type Deque struct {
	items []interface{}
	mu    sync.RWMutex
}

// OrderedDict operations

// NewOrderedDict creates a new ordered dictionary
func NewOrderedDict() *OrderedDict {
	return &OrderedDict{
		keys:   make([]string, 0),
		values: make(map[string]interface{}),
	}
}

// Set sets a key-value pair, maintaining order
func (od *OrderedDict) Set(key string, value interface{}) {
	od.mu.Lock()
	defer od.mu.Unlock()

	if _, exists := od.values[key]; !exists {
		od.keys = append(od.keys, key)
	}
	od.values[key] = value
}

// Get retrieves value by key
func (od *OrderedDict) Get(key string) (interface{}, bool) {
	od.mu.RLock()
	defer od.mu.RUnlock()
	val, ok := od.values[key]
	return val, ok
}

// Delete removes a key
func (od *OrderedDict) Delete(key string) bool {
	od.mu.Lock()
	defer od.mu.Unlock()

	if _, exists := od.values[key]; !exists {
		return false
	}

	delete(od.values, key)
	for i, k := range od.keys {
		if k == key {
			od.keys = append(od.keys[:i], od.keys[i+1:]...)
			break
		}
	}
	return true
}

// Has checks if key exists
func (od *OrderedDict) Has(key string) bool {
	od.mu.RLock()
	defer od.mu.RUnlock()
	_, exists := od.values[key]
	return exists
}

// Keys returns keys in insertion order
func (od *OrderedDict) Keys() []string {
	od.mu.RLock()
	defer od.mu.RUnlock()
	result := make([]string, len(od.keys))
	copy(result, od.keys)
	return result
}

// Values returns values in insertion order
func (od *OrderedDict) Values() []interface{} {
	od.mu.RLock()
	defer od.mu.RUnlock()
	result := make([]interface{}, len(od.keys))
	for i, key := range od.keys {
		result[i] = od.values[key]
	}
	return result
}

// Items returns key-value pairs in insertion order
func (od *OrderedDict) Items() [][2]interface{} {
	od.mu.RLock()
	defer od.mu.RUnlock()
	result := make([][2]interface{}, len(od.keys))
	for i, key := range od.keys {
		result[i] = [2]interface{}{key, od.values[key]}
	}
	return result
}

// Len returns number of items
func (od *OrderedDict) Len() int64 {
	od.mu.RLock()
	defer od.mu.RUnlock()
	return int64(len(od.keys))
}

// Clear removes all items
func (od *OrderedDict) Clear() {
	od.mu.Lock()
	defer od.mu.Unlock()
	od.keys = make([]string, 0)
	od.values = make(map[string]interface{})
}

// PopFirst removes and returns first item
func (od *OrderedDict) PopFirst() (string, interface{}, bool) {
	od.mu.Lock()
	defer od.mu.Unlock()

	if len(od.keys) == 0 {
		return "", nil, false
	}

	key := od.keys[0]
	value := od.values[key]
	od.keys = od.keys[1:]
	delete(od.values, key)
	return key, value, true
}

// PopLast removes and returns last item
func (od *OrderedDict) PopLast() (string, interface{}, bool) {
	od.mu.Lock()
	defer od.mu.Unlock()

	if len(od.keys) == 0 {
		return "", nil, false
	}

	idx := len(od.keys) - 1
	key := od.keys[idx]
	value := od.values[key]
	od.keys = od.keys[:idx]
	delete(od.values, key)
	return key, value, true
}

// Move moves key to end (or beginning if toEnd=false)
func (od *OrderedDict) Move(key string, toEnd bool) bool {
	od.mu.Lock()
	defer od.mu.Unlock()

	if _, exists := od.values[key]; !exists {
		return false
	}

	// Remove from current position
	for i, k := range od.keys {
		if k == key {
			od.keys = append(od.keys[:i], od.keys[i+1:]...)
			break
		}
	}

	// Add to new position
	if toEnd {
		od.keys = append(od.keys, key)
	} else {
		od.keys = append([]string{key}, od.keys...)
	}
	return true
}

// DefaultDict operations

// NewDefaultDict creates a defaultdict with factory function
func NewDefaultDict(factory func() interface{}) *DefaultDict {
	return &DefaultDict{
		data:    make(map[string]interface{}),
		factory: factory,
	}
}

// NewDefaultDictInt creates defaultdict returning 0 for missing keys
func NewDefaultDictInt() *DefaultDict {
	return NewDefaultDict(func() interface{} { return int64(0) })
}

// NewDefaultDictList creates defaultdict returning empty slice
func NewDefaultDictList() *DefaultDict {
	return NewDefaultDict(func() interface{} { return make([]interface{}, 0) })
}

// Get retrieves value, creating default if missing
func (dd *DefaultDict) Get(key string) interface{} {
	dd.mu.Lock()
	defer dd.mu.Unlock()

	if val, exists := dd.data[key]; exists {
		return val
	}

	val := dd.factory()
	dd.data[key] = val
	return val
}

// Set sets a value
func (dd *DefaultDict) Set(key string, value interface{}) {
	dd.mu.Lock()
	defer dd.mu.Unlock()
	dd.data[key] = value
}

// Has checks if key exists (doesn't create default)
func (dd *DefaultDict) Has(key string) bool {
	dd.mu.RLock()
	defer dd.mu.RUnlock()
	_, exists := dd.data[key]
	return exists
}

// Delete removes a key
func (dd *DefaultDict) Delete(key string) bool {
	dd.mu.Lock()
	defer dd.mu.Unlock()

	if _, exists := dd.data[key]; !exists {
		return false
	}
	delete(dd.data, key)
	return true
}

// Keys returns all keys
func (dd *DefaultDict) Keys() []string {
	dd.mu.RLock()
	defer dd.mu.RUnlock()

	keys := make([]string, 0, len(dd.data))
	for k := range dd.data {
		keys = append(keys, k)
	}
	return keys
}

// Len returns number of items
func (dd *DefaultDict) Len() int64 {
	dd.mu.RLock()
	defer dd.mu.RUnlock()
	return int64(len(dd.data))
}

// Counter operations

// NewCounter creates a new counter
func NewCounter() *Counter {
	return &Counter{
		counts: make(map[string]int64),
	}
}

// NewCounterFromSlice creates counter from slice
func NewCounterFromSlice(items []string) *Counter {
	c := NewCounter()
	for _, item := range items {
		c.Increment(item)
	}
	return c
}

// Increment increments count for item
func (c *Counter) Increment(item string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.counts[item]++
}

// IncrementBy increments by specified amount
func (c *Counter) IncrementBy(item string, amount int64) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.counts[item] += amount
}

// Decrement decrements count for item
func (c *Counter) Decrement(item string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.counts[item] > 0 {
		c.counts[item]--
	}
}

// Get returns count for item
func (c *Counter) Get(item string) int64 {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.counts[item]
}

// Set sets count for item
func (c *Counter) Set(item string, count int64) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.counts[item] = count
}

// Delete removes item from counter
func (c *Counter) Delete(item string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	delete(c.counts, item)
}

// Total returns sum of all counts
func (c *Counter) Total() int64 {
	c.mu.RLock()
	defer c.mu.RUnlock()

	total := int64(0)
	for _, count := range c.counts {
		total += count
	}
	return total
}

// MostCommon returns n most common items with counts
func (c *Counter) MostCommon(n int64) [][2]interface{} {
	c.mu.RLock()
	defer c.mu.RUnlock()

	type pair struct {
		item  string
		count int64
	}

	pairs := make([]pair, 0, len(c.counts))
	for item, count := range c.counts {
		pairs = append(pairs, pair{item, count})
	}

	sort.Slice(pairs, func(i, j int) bool {
		return pairs[i].count > pairs[j].count
	})

	limit := int(n)
	if limit > len(pairs) || limit < 0 {
		limit = len(pairs)
	}

	result := make([][2]interface{}, limit)
	for i := 0; i < limit; i++ {
		result[i] = [2]interface{}{pairs[i].item, pairs[i].count}
	}
	return result
}

// Elements returns slice with items repeated by their counts
func (c *Counter) Elements() []string {
	c.mu.RLock()
	defer c.mu.RUnlock()

	result := make([]string, 0)
	for item, count := range c.counts {
		for i := int64(0); i < count; i++ {
			result = append(result, item)
		}
	}
	return result
}

// Clear resets all counts
func (c *Counter) Clear() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.counts = make(map[string]int64)
}

// Update adds counts from another counter
func (c *Counter) Update(other *Counter) {
	c.mu.Lock()
	defer c.mu.Unlock()
	other.mu.RLock()
	defer other.mu.RUnlock()

	for item, count := range other.counts {
		c.counts[item] += count
	}
}

// Deque operations

// NewDeque creates a new double-ended queue
func NewDeque() *Deque {
	return &Deque{
		items: make([]interface{}, 0),
	}
}

// Append adds item to right end
func (d *Deque) Append(item interface{}) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.items = append(d.items, item)
}

// AppendLeft adds item to left end
func (d *Deque) AppendLeft(item interface{}) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.items = append([]interface{}{item}, d.items...)
}

// Pop removes and returns item from right end
func (d *Deque) Pop() (interface{}, bool) {
	d.mu.Lock()
	defer d.mu.Unlock()

	if len(d.items) == 0 {
		return nil, false
	}

	idx := len(d.items) - 1
	item := d.items[idx]
	d.items = d.items[:idx]
	return item, true
}

// PopLeft removes and returns item from left end
func (d *Deque) PopLeft() (interface{}, bool) {
	d.mu.Lock()
	defer d.mu.Unlock()

	if len(d.items) == 0 {
		return nil, false
	}

	item := d.items[0]
	d.items = d.items[1:]
	return item, true
}

// Len returns number of items
func (d *Deque) Len() int64 {
	d.mu.RLock()
	defer d.mu.RUnlock()
	return int64(len(d.items))
}

// Clear removes all items
func (d *Deque) Clear() {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.items = make([]interface{}, 0)
}

// Extend adds multiple items to right end
func (d *Deque) Extend(items []interface{}) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.items = append(d.items, items...)
}

// ExtendLeft adds multiple items to left end
func (d *Deque) ExtendLeft(items []interface{}) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.items = append(items, d.items...)
}

// Rotate rotates deque n steps to right (negative for left)
func (d *Deque) Rotate(n int64) {
	d.mu.Lock()
	defer d.mu.Unlock()

	length := int64(len(d.items))
	if length == 0 {
		return
	}

	n = n % length
	if n < 0 {
		n += length
	}

	d.items = append(d.items[length-n:], d.items[:length-n]...)
}
