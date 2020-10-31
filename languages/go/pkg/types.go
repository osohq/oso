package oso

// Query event
type QueryEvent struct {
	// Event kind
	Kind string `json:"kind"`
	// The actual data
	Data map[string]interface{} `json:"data"`
}
