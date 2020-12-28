package oso

import (
	"encoding/json"
	"testing"
)

func TestJson(*testing.T) {
	json_term := `{
        "Call": {
            "name": "foo",
            "args": [{"Number": {"Integer": 0}}],
            "kwargs": {"bar": {"Number": {"Integer": 1}}}
        }
	}`

	var term oso.Value
	err := json.Unmarshal(json_term, term)
	if err != nil {
		log.Println(err)
	}
	fmt.Println(term)
}