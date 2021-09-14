package main

type Repository struct {
	Id       int
	Name     string
	IsPublic bool
}

var reposDb = map[string]Repository{
	"gmail": {Id: 0, Name: "gmail"},
	"react": {Id: 1, Name: "react", IsPublic: true},
	"oso":   {Id: 2, Name: "oso"},
}

func GetRepositoryByName(name string) Repository {
	return reposDb[name]
}

// docs: start
type Role struct {
	Name       string
	Repository Repository
}

type User struct {
	Roles []Role
}

var usersDb = map[string]User{
	"larry": {Roles: []Role{{Name: "admin", Repository: reposDb["gmail"]}}},
}

// docs: end

func GetCurrentUser() User {
	return usersDb["larry"]
}
