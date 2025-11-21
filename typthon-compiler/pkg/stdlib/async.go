// Async primitives - async/await support
package stdlib

import (
	"context"
	"sync"
	"time"
)

// Future represents an asynchronous computation
type Future struct {
	done    chan struct{}
	result  interface{}
	err     error
	mu      sync.RWMutex
	started bool
}

// Task represents a unit of asynchronous work
type Task struct {
	fn     func() (interface{}, error)
	future *Future
	ctx    context.Context
	cancel context.CancelFunc
}

// Channel provides typed async communication
type Channel struct {
	ch     chan interface{}
	closed bool
	mu     sync.RWMutex
}

// WaitGroup coordinates multiple async operations
type WaitGroup struct {
	wg sync.WaitGroup
}

// Semaphore limits concurrent operations
type Semaphore struct {
	ch chan struct{}
}

// Future operations

// NewFuture creates a new future
func NewFuture() *Future {
	return &Future{
		done: make(chan struct{}),
	}
}

// AsyncRun executes function asynchronously and returns future
func AsyncRun(fn func() interface{}) *Future {
	future := NewFuture()
	future.started = true

	go func() {
		defer close(future.done)
		result := fn()
		future.mu.Lock()
		future.result = result
		future.mu.Unlock()
	}()

	return future
}

// AsyncRunErr executes function that may error asynchronously
func AsyncRunErr(fn func() (interface{}, error)) *Future {
	future := NewFuture()
	future.started = true

	go func() {
		defer close(future.done)
		result, err := fn()
		future.mu.Lock()
		future.result = result
		future.err = err
		future.mu.Unlock()
	}()

	return future
}

// Await blocks until future completes and returns result
func (f *Future) Await() interface{} {
	<-f.done
	f.mu.RLock()
	defer f.mu.RUnlock()
	return f.result
}

// AwaitErr blocks and returns result with error
func (f *Future) AwaitErr() (interface{}, error) {
	<-f.done
	f.mu.RLock()
	defer f.mu.RUnlock()
	return f.result, f.err
}

// AwaitTimeout waits with timeout, returns (result, timedOut)
func (f *Future) AwaitTimeout(seconds int64) (interface{}, bool) {
	timeout := time.Duration(seconds) * time.Second
	select {
	case <-f.done:
		f.mu.RLock()
		defer f.mu.RUnlock()
		return f.result, false
	case <-time.After(timeout):
		return nil, true
	}
}

// IsReady checks if future is complete without blocking
func (f *Future) IsReady() bool {
	select {
	case <-f.done:
		return true
	default:
		return false
	}
}

// Complete manually completes the future with a value
func (f *Future) Complete(result interface{}) bool {
	f.mu.Lock()
	defer f.mu.Unlock()

	if f.started {
		return false
	}

	f.result = result
	f.started = true
	close(f.done)
	return true
}

// CompleteErr manually completes with error
func (f *Future) CompleteErr(result interface{}, err error) bool {
	f.mu.Lock()
	defer f.mu.Unlock()

	if f.started {
		return false
	}

	f.result = result
	f.err = err
	f.started = true
	close(f.done)
	return true
}

// Task operations

// NewTask creates a new cancelable task
func NewTask(fn func() (interface{}, error)) *Task {
	ctx, cancel := context.WithCancel(context.Background())
	return &Task{
		fn:     fn,
		future: NewFuture(),
		ctx:    ctx,
		cancel: cancel,
	}
}

// Start begins task execution
func (t *Task) Start() *Future {
	if t.future.started {
		return t.future
	}

	t.future.started = true

	go func() {
		defer close(t.future.done)

		// Check cancellation before execution
		select {
		case <-t.ctx.Done():
			t.future.mu.Lock()
			t.future.err = t.ctx.Err()
			t.future.mu.Unlock()
			return
		default:
		}

		result, err := t.fn()
		t.future.mu.Lock()
		t.future.result = result
		t.future.err = err
		t.future.mu.Unlock()
	}()

	return t.future
}

// Cancel cancels the task
func (t *Task) Cancel() {
	t.cancel()
}

// IsCanceled checks if task was canceled
func (t *Task) IsCanceled() bool {
	return t.ctx.Err() != nil
}

// Channel operations

// NewChannel creates a new channel with buffer size
func NewChannel(bufferSize int64) *Channel {
	return &Channel{
		ch: make(chan interface{}, bufferSize),
	}
}

// Send sends value to channel (non-blocking if buffered)
func (c *Channel) Send(value interface{}) bool {
	c.mu.RLock()
	if c.closed {
		c.mu.RUnlock()
		return false
	}
	c.mu.RUnlock()

	c.ch <- value
	return true
}

// Recv receives value from channel (blocking)
func (c *Channel) Recv() (interface{}, bool) {
	val, ok := <-c.ch
	return val, ok
}

// TryRecv attempts to receive without blocking
func (c *Channel) TryRecv() (interface{}, bool) {
	select {
	case val, ok := <-c.ch:
		return val, ok
	default:
		return nil, false
	}
}

// RecvTimeout receives with timeout
func (c *Channel) RecvTimeout(seconds int64) (interface{}, bool) {
	timeout := time.Duration(seconds) * time.Second
	select {
	case val, ok := <-c.ch:
		return val, ok
	case <-time.After(timeout):
		return nil, false
	}
}

// Close closes the channel
func (c *Channel) Close() {
	c.mu.Lock()
	defer c.mu.Unlock()

	if !c.closed {
		close(c.ch)
		c.closed = true
	}
}

// IsClosed checks if channel is closed
func (c *Channel) IsClosed() bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.closed
}

// WaitGroup operations

// NewWaitGroup creates a new wait group
func NewWaitGroup() *WaitGroup {
	return &WaitGroup{}
}

// Add increments the wait group counter
func (wg *WaitGroup) Add(delta int64) {
	wg.wg.Add(int(delta))
}

// Done decrements the wait group counter
func (wg *WaitGroup) Done() {
	wg.wg.Done()
}

// Wait blocks until counter is zero
func (wg *WaitGroup) Wait() {
	wg.wg.Wait()
}

// WaitTimeout waits with timeout, returns true if timed out
func (wg *WaitGroup) WaitTimeout(seconds int64) bool {
	done := make(chan struct{})
	go func() {
		wg.wg.Wait()
		close(done)
	}()

	timeout := time.Duration(seconds) * time.Second
	select {
	case <-done:
		return false
	case <-time.After(timeout):
		return true
	}
}

// Semaphore operations

// NewSemaphore creates a semaphore with given capacity
func NewSemaphore(capacity int64) *Semaphore {
	return &Semaphore{
		ch: make(chan struct{}, capacity),
	}
}

// Acquire acquires a semaphore slot (blocking)
func (s *Semaphore) Acquire() {
	s.ch <- struct{}{}
}

// TryAcquire attempts to acquire without blocking
func (s *Semaphore) TryAcquire() bool {
	select {
	case s.ch <- struct{}{}:
		return true
	default:
		return false
	}
}

// AcquireTimeout attempts to acquire with timeout
func (s *Semaphore) AcquireTimeout(seconds int64) bool {
	timeout := time.Duration(seconds) * time.Second
	select {
	case s.ch <- struct{}{}:
		return true
	case <-time.After(timeout):
		return false
	}
}

// Release releases a semaphore slot
func (s *Semaphore) Release() {
	<-s.ch
}

// Available returns number of available slots
func (s *Semaphore) Available() int64 {
	return int64(cap(s.ch) - len(s.ch))
}

// Utility functions

// Sleep pauses execution for given seconds
func Sleep(seconds int64) {
	time.Sleep(time.Duration(seconds) * time.Second)
}

// SleepMillis pauses for given milliseconds
func SleepMillis(millis int64) {
	time.Sleep(time.Duration(millis) * time.Millisecond)
}

// AsyncAll waits for all futures to complete
func AsyncAll(futures ...*Future) []interface{} {
	results := make([]interface{}, len(futures))
	for i, future := range futures {
		results[i] = future.Await()
	}
	return results
}

// AsyncRace returns first completed future's result
func AsyncRace(futures ...*Future) interface{} {
	done := make(chan interface{}, len(futures))

	for _, future := range futures {
		go func(f *Future) {
			done <- f.Await()
		}(future)
	}

	return <-done
}

// AsyncGather executes multiple functions concurrently
func AsyncGather(fns ...func() interface{}) []interface{} {
	futures := make([]*Future, len(fns))
	for i, fn := range fns {
		futures[i] = AsyncRun(fn)
	}
	return AsyncAll(futures...)
}

// Retry retries function until success or max attempts
func Retry(fn func() (interface{}, error), maxAttempts int64, delaySeconds int64) (interface{}, error) {
	var lastErr error

	for i := int64(0); i < maxAttempts; i++ {
		result, err := fn()
		if err == nil {
			return result, nil
		}

		lastErr = err
		if i < maxAttempts-1 {
			time.Sleep(time.Duration(delaySeconds) * time.Second)
		}
	}

	return nil, lastErr
}

// Timeout executes function with timeout
func Timeout(fn func() interface{}, seconds int64) (interface{}, bool) {
	future := AsyncRun(fn)
	return future.AwaitTimeout(seconds)
}
