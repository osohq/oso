module github.com/osohq/go-oso/tests

go 1.14

require (
	github.com/osohq/go-oso v0.25.1
	github.com/fatih/color v1.13.0 // indirect
	github.com/goccy/go-yaml v1.9.4
	github.com/google/go-cmp v0.5.6
	github.com/mattn/go-colorable v0.1.11 // indirect
	golang.org/x/crypto v0.0.0-20210921155107-089bfa567519 // indirect
	golang.org/x/sys v0.0.0-20211020174200-9d6173849985 // indirect
	gorm.io/driver/sqlite v1.2.6
	gorm.io/gorm v1.22.5
)

replace github.com/osohq/go-oso => ../
