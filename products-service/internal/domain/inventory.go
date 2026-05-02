package domain

import (
	"errors"
	"time"
)

var (
	ErrInventoryNotFound  = errors.New("inventory_not_found")
	ErrInventoryInvariant = errors.New("inventory_invariant")
)

// Inventory is stock for one product row (FK products.id).
type Inventory struct {
	SerialID        int64
	ProductSerialID int64
	Quantity        int
	Reserved        int
	UpdatedAt       time.Time
}

func (i Inventory) Validate() error {
	if i.Quantity < 0 || i.Reserved < 0 {
		return ErrInvalidInput
	}
	if i.Reserved > i.Quantity {
		return ErrInventoryInvariant
	}
	return nil
}
