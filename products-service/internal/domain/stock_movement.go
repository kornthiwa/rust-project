package domain

import (
	"errors"
	"time"
)

// Stock movement enum (PostgreSQL stock_movement_type).
type StockMovementType string

const (
	MovementIN      StockMovementType = "IN"
	MovementOUT     StockMovementType = "OUT"
	MovementRESERVE StockMovementType = "RESERVE"
	MovementRELEASE StockMovementType = "RELEASE"
	MovementADJUST  StockMovementType = "ADJUST"
)

var ErrInsufficientStock = errors.New("insufficient_stock")

func IsKnownMovementType(t StockMovementType) bool {
	switch t {
	case MovementIN, MovementOUT, MovementRESERVE, MovementRELEASE, MovementADJUST:
		return true
	default:
		return false
	}
}

// ValidateMovementQuantity checks magnitude/sign rules before applying.
func ValidateMovementQuantity(t StockMovementType, qty int) error {
	switch t {
	case MovementADJUST:
		if qty == 0 {
			return ErrInvalidInput
		}
	default:
		if qty <= 0 {
			return ErrInvalidInput
		}
	}
	return nil
}

// ApplyStockMovement updates inventory in-memory according to ledger semantics.
func ApplyStockMovement(inv *Inventory, t StockMovementType, qty int) error {
	if err := ValidateMovementQuantity(t, qty); err != nil {
		return err
	}

	switch t {
	case MovementIN:
		inv.Quantity += qty
	case MovementOUT:
		available := inv.Quantity - inv.Reserved
		if available < qty {
			return ErrInsufficientStock
		}
		inv.Quantity -= qty
	case MovementRESERVE:
		available := inv.Quantity - inv.Reserved
		if available < qty {
			return ErrInsufficientStock
		}
		inv.Reserved += qty
	case MovementRELEASE:
		if inv.Reserved < qty {
			return ErrInsufficientStock
		}
		inv.Reserved -= qty
	case MovementADJUST:
		inv.Quantity += qty
	default:
		return ErrInvalidInput
	}

	return inv.Validate()
}

// StockMovement is one immutable ledger row.
type StockMovement struct {
	SerialID        int64
	ProductSerialID int64
	Type            StockMovementType
	Quantity        int
	ReferenceID     *string
	Note            *string
	CreatedAt       time.Time
}
