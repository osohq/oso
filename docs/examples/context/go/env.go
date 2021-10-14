import "os"

type Env struct{}

func (e Env) Var(key string) string {
	return os.Getenv(key)
}