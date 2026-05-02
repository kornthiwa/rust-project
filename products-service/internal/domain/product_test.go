package domain

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestProductValidate(t *testing.T) {
	sku := "SKU-1"
	base := Product{
		ID:        uuid.New(),
		Name:      "Widget",
		SKU:       &sku,
		CreatedAt: time.Now().UTC(),
		UpdatedAt: time.Now().UTC(),
	}
	if err := base.Validate(); err != nil {
		t.Fatalf("valid product: %v", err)
	}

	nilSKU := base
	nilSKU.SKU = nil
	if err := nilSKU.Validate(); err != nil {
		t.Fatalf("nil sku: %v", err)
	}

	emptyName := base
	emptyName.Name = ""
	if err := emptyName.Validate(); err != ErrInvalidInput {
		t.Fatalf("empty name: got %v want %v", err, ErrInvalidInput)
	}

	longSKU := base
	longSKU.Name = "ok"
	long := make([]byte, maxSKULen+1)
	for i := range long {
		long[i] = 'a'
	}
	s := string(long)
	longSKU.SKU = &s
	if err := longSKU.Validate(); err != ErrInvalidInput {
		t.Fatalf("long sku: got %v want %v", err, ErrInvalidInput)
	}
}
