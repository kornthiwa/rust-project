package domain

import (
	"testing"
	"time"
)

func TestInventoryValidate(t *testing.T) {
	ok := Inventory{
		Quantity:  10,
		Reserved:  3,
		UpdatedAt: time.Now().UTC(),
	}
	if err := ok.Validate(); err != nil {
		t.Fatal(err)
	}

	bad := ok
	bad.Reserved = 11
	if err := bad.Validate(); err != ErrInventoryInvariant {
		t.Fatalf("reserved > qty: got %v want %v", err, ErrInventoryInvariant)
	}

	neg := ok
	neg.Quantity = -1
	if err := neg.Validate(); err != ErrInvalidInput {
		t.Fatalf("negative qty: got %v want %v", err, ErrInvalidInput)
	}
}
