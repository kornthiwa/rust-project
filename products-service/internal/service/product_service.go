package service

import (
	"context"
	"strings"
	"time"

	"github.com/google/uuid"

	"products-service/internal/domain"
	"products-service/internal/repository"
)

// ProductService orchestrates product use cases.
type ProductService struct {
	repo repository.ProductRepository
	inv  repository.InventoryRepository
}

func NewProductService(
	repo repository.ProductRepository,
	inv repository.InventoryRepository,
) *ProductService {
	return &ProductService{repo: repo, inv: inv}
}

type CreateProductInput struct {
	Name string
	SKU  *string
}

func normalizeSKU(s *string) *string {
	if s == nil {
		return nil
	}
	t := strings.TrimSpace(*s)
	if t == "" {
		return nil
	}
	return &t
}

func (s *ProductService) Create(ctx context.Context, in CreateProductInput) (domain.Product, error) {
	now := time.Now().UTC()
	sku := normalizeSKU(in.SKU)
	p := domain.Product{
		ID:        uuid.New(),
		Name:      strings.TrimSpace(in.Name),
		SKU:       sku,
		CreatedAt: now,
		UpdatedAt: now,
	}
	if err := p.Validate(); err != nil {
		return domain.Product{}, err
	}
	p, err := s.repo.Create(ctx, p)
	if err != nil {
		return domain.Product{}, err
	}
	if err := s.inv.EnsureDefault(ctx, p.SerialID); err != nil {
		return domain.Product{}, err
	}
	return p, nil
}

func (s *ProductService) GetByID(ctx context.Context, id uuid.UUID) (domain.Product, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *ProductService) List(ctx context.Context, limit, offset int32) ([]domain.Product, error) {
	return s.repo.List(ctx, limit, offset)
}

type UpdateProductInput struct {
	Name *string
	SKU  *string
}

func (s *ProductService) Update(ctx context.Context, id uuid.UUID, in UpdateProductInput) (domain.Product, error) {
	existing, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return domain.Product{}, err
	}
	changed := false
	if in.Name != nil {
		existing.Name = strings.TrimSpace(*in.Name)
		changed = true
	}
	if in.SKU != nil {
		existing.SKU = normalizeSKU(in.SKU)
		changed = true
	}
	if !changed {
		return existing, nil
	}
	existing.UpdatedAt = time.Now().UTC()
	if err := existing.Validate(); err != nil {
		return domain.Product{}, err
	}
	return s.repo.Update(ctx, existing)
}

func (s *ProductService) Delete(ctx context.Context, id uuid.UUID) error {
	return s.repo.Delete(ctx, id)
}
