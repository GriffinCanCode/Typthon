// HTTP client - network programming primitives
package stdlib

import (
	"bytes"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// HTTPClient wraps http.Client with sensible defaults
type HTTPClient struct {
	client  *http.Client
	headers map[string]string
}

// HTTPResponse encapsulates response data
type HTTPResponse struct {
	Status     int64
	StatusText string
	Body       string
	Headers    map[string]string
}

// HTTPRequest encapsulates request data
type HTTPRequest struct {
	Method  string
	URL     string
	Headers map[string]string
	Body    string
	Timeout int64
}

// HTTPClientNew creates a new HTTP client
func HTTPClientNew() *HTTPClient {
	return &HTTPClient{
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
		headers: make(map[string]string),
	}
}

// HTTPClientWithTimeout creates client with custom timeout
func HTTPClientWithTimeout(seconds int64) *HTTPClient {
	return &HTTPClient{
		client: &http.Client{
			Timeout: time.Duration(seconds) * time.Second,
		},
		headers: make(map[string]string),
	}
}

// HTTPGet performs GET request
func HTTPGet(url string) *HTTPResponse {
	client := HTTPClientNew()
	return client.Get(url)
}

// HTTPPost performs POST request
func HTTPPost(url, body string) *HTTPResponse {
	client := HTTPClientNew()
	return client.Post(url, body, "application/json")
}

// HTTPPostForm performs POST with form data
func HTTPPostForm(url string, data map[string]string) *HTTPResponse {
	client := HTTPClientNew()
	form := convertToURLValues(data)
	return client.PostForm(url, form)
}

// Get performs GET request
func (c *HTTPClient) Get(url string) *HTTPResponse {
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// Post performs POST request
func (c *HTTPClient) Post(url, body, contentType string) *HTTPResponse {
	req, err := http.NewRequest("POST", url, strings.NewReader(body))
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	req.Header.Set("Content-Type", contentType)
	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// Put performs PUT request
func (c *HTTPClient) Put(url, body, contentType string) *HTTPResponse {
	req, err := http.NewRequest("PUT", url, strings.NewReader(body))
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	req.Header.Set("Content-Type", contentType)
	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// Delete performs DELETE request
func (c *HTTPClient) Delete(url string) *HTTPResponse {
	req, err := http.NewRequest("DELETE", url, nil)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// Patch performs PATCH request
func (c *HTTPClient) Patch(url, body, contentType string) *HTTPResponse {
	req, err := http.NewRequest("PATCH", url, strings.NewReader(body))
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	req.Header.Set("Content-Type", contentType)
	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// Head performs HEAD request
func (c *HTTPClient) Head(url string) *HTTPResponse {
	req, err := http.NewRequest("HEAD", url, nil)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// PostForm performs POST with form-encoded data
func (c *HTTPClient) PostForm(url string, data url.Values) *HTTPResponse {
	resp, err := c.client.PostForm(url, data)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// Request performs custom HTTP request
func (c *HTTPClient) Request(req *HTTPRequest) *HTTPResponse {
	var bodyReader io.Reader
	if req.Body != "" {
		bodyReader = strings.NewReader(req.Body)
	}

	httpReq, err := http.NewRequest(req.Method, req.URL, bodyReader)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}

	// Set custom headers
	for k, v := range req.Headers {
		httpReq.Header.Set(k, v)
	}

	// Override timeout if specified
	client := c.client
	if req.Timeout > 0 {
		client = &http.Client{
			Timeout: time.Duration(req.Timeout) * time.Second,
		}
	}

	resp, err := client.Do(httpReq)
	if err != nil {
		return &HTTPResponse{Status: 0, StatusText: err.Error()}
	}
	defer resp.Body.Close()

	return c.parseResponse(resp)
}

// SetHeader sets default header for all requests
func (c *HTTPClient) SetHeader(key, value string) {
	c.headers[key] = value
}

// SetTimeout sets client timeout in seconds
func (c *HTTPClient) SetTimeout(seconds int64) {
	c.client.Timeout = time.Duration(seconds) * time.Second
}

// parseResponse converts http.Response to HTTPResponse
func (c *HTTPClient) parseResponse(resp *http.Response) *HTTPResponse {
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return &HTTPResponse{
			Status:     int64(resp.StatusCode),
			StatusText: resp.Status,
		}
	}

	headers := make(map[string]string)
	for k, v := range resp.Header {
		if len(v) > 0 {
			headers[k] = v[0]
		}
	}

	return &HTTPResponse{
		Status:     int64(resp.StatusCode),
		StatusText: resp.Status,
		Body:       string(body),
		Headers:    headers,
	}
}

// Utility functions

// URLEncode encodes string for use in URLs
func URLEncode(s string) string {
	return url.QueryEscape(s)
}

// URLDecode decodes URL-encoded string
func URLDecode(s string) string {
	decoded, err := url.QueryUnescape(s)
	if err != nil {
		return s
	}
	return decoded
}

// URLParse parses URL string
func URLParse(rawURL string) (string, string, string, string, bool) {
	u, err := url.Parse(rawURL)
	if err != nil {
		return "", "", "", "", false
	}
	return u.Scheme, u.Host, u.Path, u.RawQuery, true
}

// URLBuild builds URL from components
func URLBuild(scheme, host, path, query string) string {
	var buf bytes.Buffer
	buf.WriteString(scheme)
	buf.WriteString("://")
	buf.WriteString(host)
	buf.WriteString(path)
	if query != "" {
		buf.WriteString("?")
		buf.WriteString(query)
	}
	return buf.String()
}

// convertToURLValues converts map to url.Values
func convertToURLValues(data map[string]string) url.Values {
	values := url.Values{}
	for k, v := range data {
		values.Set(k, v)
	}
	return values
}

// Response helper methods

// IsSuccess checks if status code indicates success (2xx)
func (r *HTTPResponse) IsSuccess() bool {
	return r.Status >= 200 && r.Status < 300
}

// IsRedirect checks if status code indicates redirect (3xx)
func (r *HTTPResponse) IsRedirect() bool {
	return r.Status >= 300 && r.Status < 400
}

// IsClientError checks if status code indicates client error (4xx)
func (r *HTTPResponse) IsClientError() bool {
	return r.Status >= 400 && r.Status < 500
}

// IsServerError checks if status code indicates server error (5xx)
func (r *HTTPResponse) IsServerError() bool {
	return r.Status >= 500 && r.Status < 600
}

// GetHeader retrieves response header
func (r *HTTPResponse) GetHeader(key string) (string, bool) {
	val, ok := r.Headers[key]
	return val, ok
}

// JSON attempts to parse response body as JSON
func (r *HTTPResponse) JSON() (interface{}, bool) {
	return JSONParse(r.Body)
}
