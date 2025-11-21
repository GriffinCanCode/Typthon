package stdlib

import (
	"testing"
)

// Example: Using regex with caching
func ExampleRegex() {
	// Pattern is compiled once and cached
	text := "Phone: 555-1234, Mobile: 555-5678"

	// Find all phone numbers
	pattern := `\d{3}-\d{4}`
	phones := RegexFindAll(pattern, text)
	_ = phones // phones = ["555-1234", "555-5678"]

	// Named groups
	emailPattern := `(?P<user>\w+)@(?P<domain>\w+\.\w+)`
	groups, ok := RegexNamedGroups(emailPattern, "user@example.com")
	if ok {
		user := groups["user"]     // "user"
		domain := groups["domain"] // "example.com"
		_, _ = user, domain
	}

	// Replace with backreferences
	replaced := RegexReplace(`(\w+)\s+(\w+)`, "John Doe", "$2, $1")
	// replaced = "Doe, John"
	_ = replaced
}

// Example: HTTP client for REST APIs
func ExampleHTTPClient() {
	// Simple GET request
	resp := HTTPGet("https://api.github.com/users/golang")
	if resp.IsSuccess() {
		data, ok := resp.JSON()
		if ok {
			// Process JSON data
			_ = data
		}
	}

	// POST with custom headers
	client := HTTPClientNew()
	client.SetHeader("Authorization", "Bearer token123")
	client.SetHeader("Content-Type", "application/json")

	body := `{"name":"NewRepo","private":false}`
	resp = client.Post("https://api.github.com/user/repos", body, "application/json")

	if resp.Status == 201 {
		// Repository created
	}
}

// Example: Async/await patterns
func ExampleAsyncRun() {
	// Run multiple tasks concurrently
	future1 := AsyncRun(func() interface{} {
		// Expensive computation 1
		return computePrimes(1000)
	})

	future2 := AsyncRun(func() interface{} {
		// Expensive computation 2
		return computeFactorials(100)
	})

	// Wait for both to complete
	results := AsyncAll(future1, future2)
	primes := results[0]
	factorials := results[1]
	_, _ = primes, factorials

	// Race - first to complete wins
	fastest := AsyncRace(future1, future2)
	_ = fastest

	// With timeout
	result, timedOut := future1.AwaitTimeout(5)
	if !timedOut {
		// Process result
		_ = result
	}
}

// Example: Channel-based communication
func ExampleNewChannel() {
	ch := NewChannel(10) // Buffered channel

	// Producer
	go func() {
		for i := 0; i < 100; i++ {
			ch.Send(i)
		}
		ch.Close()
	}()

	// Consumer
	for {
		val, ok := ch.Recv()
		if !ok {
			break // Channel closed
		}
		// Process val
		_ = val
	}
}

// Example: Semaphore for rate limiting
func ExampleSemaphore() {
	sem := NewSemaphore(3) // Max 3 concurrent operations

	wg := NewWaitGroup()
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()

			sem.Acquire()
			defer sem.Release()

			// Only 3 tasks run concurrently
			processTask(id)
		}(i)
	}

	wg.Wait()
}

// Example: OrderedDict maintaining insertion order
func ExampleOrderedDict() {
	od := NewOrderedDict()

	od.Set("first", 1)
	od.Set("second", 2)
	od.Set("third", 3)

	// Keys in insertion order
	keys := od.Keys() // ["first", "second", "third"]
	_ = keys

	// Move to end
	od.Move("first", true)
	keys = od.Keys() // ["second", "third", "first"]
	_ = keys

	// Pop from ends
	key, val, ok := od.PopFirst()
	if ok {
		// key = "second", val = 2
		_, _ = key, val
	}

	// Iterate in order
	for _, item := range od.Items() {
		key := item[0].(string)
		val := item[1]
		_, _ = key, val
	}
}

// Example: DefaultDict with factory
func ExampleDefaultDict() {
	// Count words in text
	wordCount := NewDefaultDictInt()

	words := []string{"apple", "banana", "apple", "cherry", "banana", "apple"}
	for _, word := range words {
		count := wordCount.Get(word).(int64)
		wordCount.Set(word, count+1)
	}

	// Group items by category
	groups := NewDefaultDictList()

	type Item struct {
		category string
		name     string
	}

	items := []Item{
		{"fruit", "apple"},
		{"fruit", "banana"},
		{"veggie", "carrot"},
	}

	for _, item := range items {
		list := groups.Get(item.category).([]interface{})
		list = append(list, item.name)
		groups.Set(item.category, list)
	}
}

// Example: Counter for frequency analysis
func ExampleCounter() {
	c := NewCounter()

	// Count characters
	text := "hello world"
	for _, ch := range text {
		c.Increment(string(ch))
	}

	// Get most common
	top5 := c.MostCommon(5)
	for _, pair := range top5 {
		char := pair[0].(string)
		count := pair[1].(int64)
		_, _ = char, count
	}

	// Total count
	total := c.Total()
	_ = total

	// Merge counters
	c2 := NewCounter()
	c2.Increment("a")
	c.Update(c2)
}

// Example: Deque for queue/stack operations
func ExampleDeque() {
	dq := NewDeque()

	// Use as queue (FIFO)
	dq.Append("first")
	dq.Append("second")
	item, ok := dq.PopLeft()
	if ok {
		// item = "first"
		_ = item
	}

	// Use as stack (LIFO)
	dq.Append("a")
	dq.Append("b")
	item, ok = dq.Pop()
	if ok {
		// item = "b"
		_ = item
	}

	// Rotate
	dq.Clear()
	dq.Extend([]interface{}{1, 2, 3, 4, 5})
	dq.Rotate(2) // [4, 5, 1, 2, 3]
}

// Example: Combining async patterns
func ExampleRetry() {
	// Retry with backoff
	result, err := Retry(func() (interface{}, error) {
		return fetchDataFromAPI()
	}, 3, 1) // 3 attempts, 1 second between

	if err != nil {
		// All retries failed
	} else {
		_ = result
	}

	// Parallel fetch with gather
	urls := []string{"url1", "url2", "url3"}
	futures := make([]*Future, len(urls))

	for i, url := range urls {
		futures[i] = AsyncRun(func() interface{} {
			return HTTPGet(url)
		})
	}

	results := AsyncAll(futures...)
	_ = results

	// Task with cancellation
	var task *Task
	task = NewTask(func() (interface{}, error) {
		for i := 0; i < 1000; i++ {
			// Check if canceled
			if task.IsCanceled() {
				return nil, nil
			}
			SleepMillis(10)
		}
		return "done", nil
	})

	future := task.Start()

	// Cancel after timeout
	go func() {
		Sleep(2)
		task.Cancel()
	}()

	result, err = future.AwaitErr()
	_, _ = result, err
}

// Example: Real-world web scraping
func ExampleHTTPGet() {
	// Fetch multiple pages concurrently
	urls := []string{
		"https://example.com/page1",
		"https://example.com/page2",
		"https://example.com/page3",
	}

	// Rate limit with semaphore
	sem := NewSemaphore(2) // Max 2 concurrent requests
	ch := NewChannel(int64(len(urls)))

	for _, url := range urls {
		go func(u string) {
			sem.Acquire()
			defer sem.Release()

			resp := HTTPGet(u)
			if resp.IsSuccess() {
				// Extract data with regex
				emails := RegexFindAll(`[\w\.-]+@[\w\.-]+\.\w+`, resp.Body)
				ch.Send(emails)
			}
		}(url)
	}

	// Collect results
	allEmails := NewCounter()
	for i := 0; i < len(urls); i++ {
		emails, ok := ch.Recv()
		if ok {
			for _, email := range emails.([]string) {
				allEmails.Increment(email)
			}
		}
	}

	// Get most common emails
	top := allEmails.MostCommon(10)
	_ = top
}

// Helper functions for examples
func computePrimes(n int) []int              { return nil }
func computeFactorials(n int) []int          { return nil }
func processTask(id int)                     {}
func fetchDataFromAPI() (interface{}, error) { return nil, nil }

// Benchmark: Regex caching performance
func BenchmarkRegexCaching(b *testing.B) {
	pattern := `\d{3}-\d{3}-\d{4}`
	text := "Call us at 555-123-4567 or 555-987-6543"

	b.Run("WithCaching", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			RegexFindAll(pattern, text)
		}
	})
}

// Benchmark: Counter vs Map
func BenchmarkCounter(b *testing.B) {
	words := []string{"apple", "banana", "cherry", "apple", "banana", "apple"}

	b.Run("Counter", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			c := NewCounter()
			for _, word := range words {
				c.Increment(word)
			}
		}
	})

	b.Run("RawMap", func(b *testing.B) {
		for i := 0; i < b.N; i++ {
			m := make(map[string]int64)
			for _, word := range words {
				m[word]++
			}
		}
	})
}

// Test: OrderedDict ordering
func TestOrderedDictOrdering(t *testing.T) {
	od := NewOrderedDict()

	od.Set("z", 1)
	od.Set("a", 2)
	od.Set("m", 3)

	keys := od.Keys()
	if len(keys) != 3 {
		t.Errorf("Expected 3 keys, got %d", len(keys))
	}

	if keys[0] != "z" || keys[1] != "a" || keys[2] != "m" {
		t.Errorf("Keys not in insertion order: %v", keys)
	}
}

// Test: DefaultDict factory
func TestDefaultDictFactory(t *testing.T) {
	dd := NewDefaultDictInt()

	// First access creates default
	val := dd.Get("missing").(int64)
	if val != 0 {
		t.Errorf("Expected 0, got %d", val)
	}

	// Subsequent access returns same value
	val = dd.Get("missing").(int64)
	if val != 0 {
		t.Errorf("Expected 0, got %d", val)
	}
}

// Test: Counter operations
func TestCounterOperations(t *testing.T) {
	c := NewCounter()

	c.Increment("a")
	c.Increment("a")
	c.Increment("b")

	if c.Get("a") != 2 {
		t.Errorf("Expected count 2 for 'a', got %d", c.Get("a"))
	}

	if c.Total() != 3 {
		t.Errorf("Expected total 3, got %d", c.Total())
	}

	top := c.MostCommon(1)
	if len(top) != 1 {
		t.Errorf("Expected 1 item, got %d", len(top))
	}

	if top[0][0].(string) != "a" {
		t.Errorf("Expected 'a' as most common, got %s", top[0][0])
	}
}

// Test: Async Future operations
func TestAsyncFuture(t *testing.T) {
	future := AsyncRun(func() interface{} {
		return 42
	})

	result := future.Await()
	if result.(int) != 42 {
		t.Errorf("Expected 42, got %v", result)
	}
}

// Test: Channel communication
func TestChannelCommunication(t *testing.T) {
	ch := NewChannel(1)

	ch.Send("test")
	val, ok := ch.Recv()

	if !ok {
		t.Error("Expected successful receive")
	}

	if val.(string) != "test" {
		t.Errorf("Expected 'test', got %v", val)
	}
}
