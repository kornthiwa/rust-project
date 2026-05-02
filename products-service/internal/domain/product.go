package domain

import (
	"errors"
	"strings"
	"time"

	"github.com/google/uuid"
)

var (
	ErrProductNotFound = errors.New("product_not_found")
	ErrInvalidInput    = errors.New("invalid_input")
)

const maxNameLen = 10000
const maxSKULen = 2048

// Product is the domain entity for catalog items.
// SerialID is the database BIGSERIAL (internal); ID is the stable public UUID stored in column uuid.
type Product struct {
	SerialID  int64
	ID        uuid.UUID
	Name      string
	SKU       *string
	CreatedAt time.Time
	UpdatedAt time.Time
}

func validateName(name string) error {
	name = strings.TrimSpace(name)
	if name == "" || len(name) > maxNameLen {
		return ErrInvalidInput
	}
	return nil
}

func validateSKU(sku *string) error {
	if sku == nil {
		return nil
	}
	if len(strings.TrimSpace(*sku)) > maxSKULen {
		return ErrInvalidInput
	}
	return nil
}

// Validate checks domain rules for create/update payloads.
func (p Product) Validate() error {
	if err := validateName(p.Name); err != nil {
		return err
	}
	return validateSKU(p.SKU)
}
