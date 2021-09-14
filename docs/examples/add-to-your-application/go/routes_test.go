package main

import (
	"net/http"
	"testing"
)

func TestRoutesWork(t *testing.T) {
	_ = initOso()
	app := InitApp()

	req, _ := http.NewRequest(
		"GET",
		"/repo/gmail",
		nil,
	)

	res, err := app.Test(req, -1)

	if err != nil {
		t.Fatalf("Err not nil")
	}

	if res.StatusCode != 200 {
		t.Fatalf("Status not 200, it was %d", res.StatusCode)
	}
}
