module github.com/osohq/go-oso/tests

go 1.14

require (
	github.com/fatih/color v1.13.0 // indirect
	github.com/goccy/go-yaml v1.9.5
	github.com/google/go-cmp v0.5.7
	github.com/mattn/go-colorable v0.1.12 // indirect
	github.com/mattn/go-sqlite3 v1.14.10 // indirect
	github.com/osohq/go-oso v0.25.1
	golang.org/x/crypto v0.0.0-20210921155107-089bfa567519 // indirect
	golang.org/x/sys v0.0.0-20220114195835-da31bd327af9 // indirect
	gorm.io/driver/sqlite v1.2.6
	gorm.io/gorm v1.22.5
)

replace github.com/osohq/go-oso => ../
