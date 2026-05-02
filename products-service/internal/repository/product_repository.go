package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"products-service/internal/domain"
)

// ProductRepository persists products.
type ProductRepository interface {
	Create(ctx context.Context, p domain.Product) (domain.Product, error)
	GetByID(ctx context.Context, id uuid.UUID) (domain.Product, error)
	GetSerialIDByUUID(ctx context.Context, id uuid.UUID) (int64, error)
	List(ctx context.Context, limit, offset int32) ([]domain.Product, error)
	Update(ctx context.Context, p domain.Product) (domain.Product, error)
	Delete(ctx context.Context, id uuid.UUID) error
}

// productModel maps the products table for GORM (see SQL: id BIGSERIAL, uuid UUID UNIQUE, …).
type productModel struct {
	SerialID  int64     `gorm:"column:id;primaryKey;autoIncrement"`
	PublicID  uuid.UUID `gorm:"column:uuid;type:uuid;uniqueIndex;not null"`
	Name      string    `gorm:"type:text;not null"`
	SKU       *string   `gorm:"type:text;uniqueIndex"`
	CreatedAt time.Time `gorm:"column:created_at"`
	UpdatedAt time.Time `gorm:"column:updated_at"`
}

func (productModel) TableName() string { return "products" }

// GORMProductRepository implements ProductRepository using GORM.
type GORMProductRepository struct {
	db *gorm.DB
}

func NewGORMProductRepository(db *gorm.DB) *GORMProductRepository {
	return &GORMProductRepository{db: db}
}

func (r *GORMProductRepository) Create(ctx context.Context, p domain.Product) (domain.Product, error) {
	row := domainToModel(p)
	if err := r.db.WithContext(ctx).Create(&row).Error; err != nil {
		return domain.Product{}, err
	}
	return modelToDomain(row), nil
}

func (r *GORMProductRepository) GetByID(ctx context.Context, id uuid.UUID) (domain.Product, error) {
	var row productModel
	err := r.db.WithContext(ctx).Where("uuid = ?", id).First(&row).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return domain.Product{}, domain.ErrProductNotFound
		}
		return domain.Product{}, err
	}
	return modelToDomain(row), nil
}

func (r *GORMProductRepository) GetSerialIDByUUID(ctx context.Context, id uuid.UUID) (int64, error) {
	var row productModel
	err := r.db.WithContext(ctx).Select("id").Where("uuid = ?", id).First(&row).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return 0, domain.ErrProductNotFound
		}
		return 0, err
	}
	return row.SerialID, nil
}

func (r *GORMProductRepository) List(ctx context.Context, limit, offset int32) ([]domain.Product, error) {
	if limit <= 0 {
		limit = 50
	}
	if limit > 200 {
		limit = 200
	}
	if offset < 0 {
		offset = 0
	}
	var rows []productModel
	err := r.db.WithContext(ctx).
		Order("created_at DESC").
		Limit(int(limit)).
		Offset(int(offset)).
		Find(&rows).Error
	if err != nil {
		return nil, err
	}
	out := make([]domain.Product, 0, len(rows))
	for _, row := range rows {
		out = append(out, modelToDomain(row))
	}
	return out, nil
}

func (r *GORMProductRepository) Update(ctx context.Context, p domain.Product) (domain.Product, error) {
	updates := map[string]any{
		"name":       p.Name,
		"updated_at": p.UpdatedAt.UTC(),
	}
	if p.SKU != nil {
		updates["sku"] = *p.SKU
	} else {
		updates["sku"] = nil
	}
	res := r.db.WithContext(ctx).Model(&productModel{}).
		Where("uuid = ?", p.ID).
		Updates(updates)
	if res.Error != nil {
		return domain.Product{}, res.Error
	}
	if res.RowsAffected == 0 {
		return domain.Product{}, domain.ErrProductNotFound
	}
	return r.GetByID(ctx, p.ID)
}

func (r *GORMProductRepository) Delete(ctx context.Context, id uuid.UUID) error {
	res := r.db.WithContext(ctx).Where("uuid = ?", id).Delete(&productModel{})
	if res.Error != nil {
		return res.Error
	}
	if res.RowsAffected == 0 {
		return domain.ErrProductNotFound
	}
	return nil
}

func domainToModel(p domain.Product) productModel {
	return productModel{
		SerialID:  p.SerialID,
		PublicID:  p.ID,
		Name:      p.Name,
		SKU:       p.SKU,
		CreatedAt: p.CreatedAt.UTC(),
		UpdatedAt: p.UpdatedAt.UTC(),
	}
}

func modelToDomain(m productModel) domain.Product {
	return domain.Product{
		SerialID:  m.SerialID,
		ID:        m.PublicID,
		Name:      m.Name,
		SKU:       m.SKU,
		CreatedAt: m.CreatedAt.UTC(),
		UpdatedAt: m.UpdatedAt.UTC(),
	}
}
