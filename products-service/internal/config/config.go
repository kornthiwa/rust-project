package config

import (
	"fmt"
	"os"
)

// Load reads service configuration from environment variables.
func Load() (port string, databaseURL string, jwtSecret string, err error) {
	port = os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	databaseURL = os.Getenv("DATABASE_URL")
	if databaseURL == "" {
		return "", "", "", fmt.Errorf("DATABASE_URL is required")
	}
	jwtSecret = os.Getenv("JWT_SECRET")
	if jwtSecret == "" {
		return "", "", "", fmt.Errorf("JWT_SECRET is required")
	}
	return port, databaseURL, jwtSecret, nil
}
