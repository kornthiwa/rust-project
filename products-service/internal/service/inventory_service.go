package service

import (
	"context"

	"github.com/google/uuid"

	"products-service/internal/domain"
	"products-service/internal/repository"
)

// InventoryService manages stock rows keyed by product DB id (FK products.id).
type InventoryService struct {
	products repository.ProductRepository
	inv      repository.InventoryRepository
}

func NewInventoryService(
	products repository.ProductRepository,
	inv repository.InventoryRepository,
) *InventoryService {
	return &InventoryService{products: products, inv: inv}
}

func (s *InventoryService) GetForProductUUID(ctx context.Context, productUUID uuid.UUID) (domain.Inventory, error) {
	pid, err := s.products.GetSerialIDByUUID(ctx, productUUID)
	if err != nil {
		return domain.Inventory{}, err
	}
	if err := s.inv.EnsureDefault(ctx, pid); err != nil {
		return domain.Inventory{}, err
	}
	return s.inv.GetByProductSerialID(ctx, pid)
}

type PatchInventoryInput struct {
	Quantity *int
	Reserved *int
}

func (s *InventoryService) PatchForProductUUID(ctx context.Context, productUUID uuid.UUID, in PatchInventoryInput) (domain.Inventory, error) {
	pid, err := s.products.GetSerialIDByUUID(ctx, productUUID)
	if err != nil {
		return domain.Inventory{}, err
	}
	if err := s.inv.EnsureDefault(ctx, pid); err != nil {
		return domain.Inventory{}, err
	}
	cur, err := s.inv.GetByProductSerialID(ctx, pid)
	if err != nil {
		return domain.Inventory{}, err
	}
	changed := false
	if in.Quantity != nil {
		cur.Quantity = *in.Quantity
		changed = true
	}
	if in.Reserved != nil {
		cur.Reserved = *in.Reserved
		changed = true
	}
	if !changed {
		return cur, nil
	}
	if err := cur.Validate(); err != nil {
		return domain.Inventory{}, err
	}
	return s.inv.Update(ctx, cur)
}
