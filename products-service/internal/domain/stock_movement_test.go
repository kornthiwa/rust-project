package domain

import (
	"testing"
	"time"
)

func TestApplyStockMovement(t *testing.T) {
	inv := &Inventory{
		ProductSerialID: 1,
		Quantity:        20,
		Reserved:        0,
		UpdatedAt:       time.Now().UTC(),
	}

	if err := ApplyStockMovement(inv, MovementIN, 5); err != nil || inv.Quantity != 25 {
		t.Fatalf("IN: %+v qty=%d err=%v", inv, inv.Quantity, err)
	}

	if err := ApplyStockMovement(inv, MovementRESERVE, 5); err != nil || inv.Reserved != 5 || inv.Quantity != 25 {
		t.Fatalf("RESERVE: %+v err=%v", inv, err)
	}

	if err := ApplyStockMovement(inv, MovementOUT, 3); err != nil || inv.Quantity != 22 || inv.Reserved != 5 {
		t.Fatalf("OUT: %+v err=%v", inv, err)
	}

	if err := ApplyStockMovement(inv, MovementRELEASE, 5); err != nil || inv.Reserved != 0 {
		t.Fatalf("RELEASE: %+v err=%v", inv, err)
	}

	if err := ApplyStockMovement(inv, MovementADJUST, -2); err != nil || inv.Quantity != 20 {
		t.Fatalf("ADJUST: %+v err=%v", inv, err)
	}

	bare := &Inventory{Quantity: 5, Reserved: 0, UpdatedAt: time.Now().UTC()}
	if err := ApplyStockMovement(bare, MovementOUT, 10); err != ErrInsufficientStock {
		t.Fatalf("OUT insufficient: got %v want %v", err, ErrInsufficientStock)
	}
}
