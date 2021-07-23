package util

import "strings"

func QueryStrip(raw string) string {
	text := strings.TrimSuffix(raw, "\r\n")
	text = strings.TrimSuffix(text, "\n")
	text = strings.TrimSuffix(text, ";")
	return text
}
