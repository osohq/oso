package types

import "fmt"

func (e *FormattedPolarError) Error() string {
	return fmt.Sprintf("Error: %#v\n%s", e.Kind, e.Formatted)
}
