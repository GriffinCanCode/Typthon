# Typthon Standard Library

Comprehensive standard library implementations for compiled Python code. All modules are implemented in Go for maximum performance and zero-overhead abstractions.

## Architecture

The stdlib follows these design principles:

1. **Zero-overhead abstractions** - Compile-time optimizations, no runtime penalty
2. **Minimal error handling** - Returns nil/false on error for performance
3. **Direct mapping to Go stdlib** - Leverages battle-tested implementations
4. **Thread-safe where needed** - Concurrent data structures use sync primitives
5. **Python-compatible APIs** - Maintains familiar Python semantics

## Modules

### Core Collections (`collections.go`)

Basic collection utilities:
- `Range` - Efficient iteration with start/stop/step
- List and dict operations

### Advanced Collections (`advancedcollections.go`)

Specialized data structures from Python's collections module:

#### OrderedDict
Maintains insertion order of keys. Thread-safe with RWMutex.

```go
od := NewOrderedDict()
od.Set("first", 1)
od.Set("second", 2)
keys := od.Keys() // ["first", "second"] - insertion order preserved
```

**Operations**: Set, Get, Delete, Keys, Values, Items, PopFirst, PopLast, Move

#### DefaultDict
Provides default values for missing keys via factory function.

```go
dd := NewDefaultDictInt() // Returns 0 for missing keys
dd.Set("count", 5)
val := dd.Get("missing") // Returns 0
```

**Factory types**: Int (0), List (empty slice), Custom

#### Counter
Counts hashable objects efficiently.

```go
c := NewCounter()
c.Increment("apple")
c.IncrementBy("apple", 2)
top := c.MostCommon(5) // Top 5 items with counts
```

**Operations**: Increment, Get, Total, MostCommon, Elements, Update

#### Deque
Double-ended queue for efficient append/pop from both ends.

```go
dq := NewDeque()
dq.Append(1)
dq.AppendLeft(0)
val, ok := dq.PopLeft()
```

**Operations**: Append, AppendLeft, Pop, PopLeft, Rotate, Extend

### Regex (`regex.go`)

Comprehensive pattern matching with compile caching for performance.

```go
// Automatic pattern caching
matches := RegexFindAll(`\d+`, "abc 123 def 456")

// Named groups
groups, ok := RegexNamedGroups(`(?P<year>\d{4})-(?P<month>\d{2})`, "2024-11")

// Compiled regex objects
re := RegexCompile(`\w+`)
result := re.Replace("hello world", "X")
```

**Key features**:
- Thread-safe pattern cache with RWMutex
- Named capturing groups
- Match, Search, FindAll, Replace, Split
- Index-based operations
- Escape utility

### HTTP Client (`http.go`)

Full-featured HTTP client for network programming.

```go
// Simple requests
resp := HTTPGet("https://api.example.com/data")
if resp.IsSuccess() {
    data, _ := resp.JSON()
}

// Custom client with timeout
client := HTTPClientWithTimeout(10)
client.SetHeader("Authorization", "Bearer token")
resp = client.Post("/api/users", `{"name":"John"}`, "application/json")
```

**Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
**Features**:
- Custom headers and timeouts
- Form-encoded POST
- Response helpers (IsSuccess, IsError, JSON parsing)
- URL encoding/decoding utilities

### Async Primitives (`async.go`)

Modern async/await support with Go's concurrency model.

#### Future
Represents asynchronous computation with blocking and non-blocking operations.

```go
future := AsyncRun(func() interface{} {
    // Long-running computation
    return compute()
})

// Non-blocking check
if future.IsReady() {
    result := future.Await()
}

// Timeout
result, timedOut := future.AwaitTimeout(5)
```

#### Task
Cancelable asynchronous work.

```go
task := NewTask(func() (interface{}, error) {
    return fetchData()
})

future := task.Start()
// Later: task.Cancel()
```

#### Channel
Type-safe async communication with buffering.

```go
ch := NewChannel(10) // Buffer size 10
ch.Send("hello")

val, ok := ch.Recv()          // Blocking
val, ok = ch.TryRecv()        // Non-blocking
val, ok = ch.RecvTimeout(5)   // With timeout
```

#### WaitGroup
Coordinates multiple async operations.

```go
wg := NewWaitGroup()
for i := 0; i < 10; i++ {
    wg.Add(1)
    go func() {
        defer wg.Done()
        work()
    }()
}
wg.Wait()
```

#### Semaphore
Limits concurrent operations.

```go
sem := NewSemaphore(3) // Max 3 concurrent
sem.Acquire()
defer sem.Release()
// Critical section
```

**Utilities**: AsyncAll, AsyncRace, AsyncGather, Retry, Timeout, Sleep

### Itertools (`itertools.go`)

Functional programming primitives for efficient iteration.

```go
// Chain multiple iterators
chain := NewChain([]int64{1,2}, []int64{3,4})

// Zip sequences
pairs := Zip([]int64{1,2,3}, []int64{4,5,6})

// Higher-order functions
evens := Filter(nums, func(x int64) bool { return x%2 == 0 })
squares := Map(nums, func(x int64) int64 { return x*x })
sum := Reduce(nums, func(a,b int64) int64 { return a+b }, 0)
```

### String Operations (`strings.go`)

Comprehensive string manipulation.

```go
parts := StrSplit("a,b,c", ",")
joined := StrJoin(parts, ";")
upper := StrUpper("hello")
found := StrFind("hello world", "world") // Returns index
count := StrCount("aaabbb", "aa")        // Returns 2
```

**Operations**: Split, Join, Upper, Lower, Strip, Replace, Contains, StartsWith, EndsWith, Find, Count

### Math (`math.go`)

Mathematical operations for integers and floats.

```go
abs := Abs(-5)
result := Pow(2, 10)
sqrt := Sqrt(16.0)
rounded := Floor(3.7)
larger := Max(10, 20)
```

**Operations**: Abs, Pow, Sqrt, Floor, Ceil, Min, Max (with int/float variants)

### File I/O (`io.go`)

Complete file system operations.

```go
// File operations
f := FileOpen("data.txt", "r")
content := FileRead(f)
FileClose(f)

// Writing
f = FileOpen("output.txt", "w")
FileWrite(f, "Hello World")
FileFlush(f)

// Directory operations
DirCreateAll("/path/to/dir")
files := DirList("/path")
```

**Operations**: Open, Read, Write, ReadLine, Flush, Close, Exists, Remove, Rename
**Dir operations**: Create, CreateAll, Remove, RemoveAll, List

### JSON (`json.go`)

JSON parsing and serialization with builder API.

```go
// Parse
data, ok := JSONParse(`{"name":"John","age":30}`)
obj, ok := JSONParseObject(`{"key":"value"}`)

// Stringify
json, ok := JSONStringify(data)
pretty, ok := JSONStringifyPretty(data)

// Builder API
obj := JSONObjectNew()
JSONObjectSet(obj, "name", "John")
JSONObjectSet(obj, "age", 30)

arr := JSONArrayNew()
arr = JSONArrayAppend(arr, "item1")
```

## Performance Characteristics

| Module | Thread-Safe | Caching | Allocation Strategy |
|--------|-------------|---------|---------------------|
| Regex | Yes (RWMutex) | Pattern cache | Reuse compiled patterns |
| HTTP | No | No | Connection pooling via net/http |
| Async | Yes | No | Channel-based, minimal allocation |
| OrderedDict | Yes (RWMutex) | No | Slice + map |
| DefaultDict | Yes (RWMutex) | No | Lazy initialization |
| Counter | Yes (RWMutex) | No | Map-based |
| Deque | Yes (RWMutex) | No | Slice-based |

## Thread Safety

- **Thread-safe**: OrderedDict, DefaultDict, Counter, Deque, Async primitives, Regex (cache only)
- **Not thread-safe**: HTTPClient, File handles, basic collections

For concurrent access to non-thread-safe types, use sync.Mutex or Semaphore.

## Integration with Typthon

All stdlib functions are automatically available in compiled Typthon code:

```python
# Typthon code
import regex
import http
import async

# Regex with caching
matches = regex.find_all(r'\d+', text)

# HTTP client
response = http.get('https://api.example.com')
if response.is_success():
    data = response.json()

# Async operations
future = async.run(lambda: expensive_computation())
result = future.await()
```

## Design Decisions

### Why Go implementations?
- **Performance**: Native code, no interpreter overhead
- **Concurrency**: Built-in goroutines and channels
- **Safety**: Strong typing, memory safety
- **Ecosystem**: Rich stdlib to build upon

### Pattern caching
Regex patterns are cached after first compilation for repeated use, trading memory for speed. Cache is thread-safe with RWMutex for concurrent access.

### Error handling
Most functions return nil/false/empty on error rather than panicking, allowing compiled code to continue execution. Critical errors (e.g., OOM) still panic.

### Mutex granularity
Collections use RWMutex for optimal read performance. Writers get exclusive access, readers share.

## Future Enhancements

- [ ] Database drivers (SQLite, PostgreSQL)
- [ ] Cryptography module
- [ ] XML/CSV parsing
- [ ] Compression utilities
- [ ] Process management
- [ ] Network sockets (raw TCP/UDP)
- [ ] Time and date utilities
- [ ] Path manipulation
- [ ] Templating engine

## Contributing

When adding new stdlib modules:

1. Follow existing patterns (simple functions, minimal errors)
2. Use Go stdlib where possible
3. Document performance characteristics
4. Add thread safety where needed (collections, shared state)
5. Keep functions focused and composable
6. Test with real-world use cases

