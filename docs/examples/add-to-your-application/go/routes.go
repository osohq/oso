package main

import (
	"fmt"
	"github.com/gofiber/fiber/v2"
	"log"
)

func InitApp() *fiber.App {
	oso := initOso()
	app := fiber.New()

	// docs: begin-show-route
	app.Get("/repo/:repoName", func(c *fiber.Ctx) error {
		c.Set(fiber.HeaderContentType, fiber.MIMETextHTML)
		repoName := c.Params("repoName")
		repository := GetRepositoryByName(repoName)
		// docs: begin-authorize
		err := oso.Authorize(GetCurrentUser(), "read", repository)
		// docs: end-authorize
		if err == nil {
			return c.Status(200).SendString(fmt.Sprintf("<h1>A Repo</h1><p>Welcome to repo %s</p>", repository.Name))
		} else {
			return c.Status(404).SendString("<h1>Whoops!</h1><p>That repo was not found</p>")
		}
	})
	// docs: end-show-route

	return app
}

func main() {
	app := InitApp()
	if err := app.Listen(":5000"); err != nil {
		log.Fatalf("Failed to start: %s", err)
	}
}
