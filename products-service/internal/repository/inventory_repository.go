package repository

import (
	"context"
	"errors"
	"time"

	"gorm.io/gorm"
	"gorm.io/gorm/clause"

	"products-service/internal/domain"
)

// InventoryRepository persists per-product stock rows (product_id UNIQUE).
type InventoryRepository interface {
	EnsureDefault(ctx context.Context, productSerialID int64) error
	GetByProductSerialID(ctx context.Context, productSerialID int64) (domain.Inventory, error)
	GetByProductSerialIDForUpdate(ctx context.Context, productSerialID int64) (domain.Inventory, error)
	Update(ctx context.Context, inv domain.Inventory) (domain.Inventory, error)
}

type inventoryModel struct {
	SerialID  int64     `gorm:"column:id;primaryKey;autoIncrement"`
	ProductID int64     `gorm:"column:product_id;uniqueIndex;not null"`
	Quantity  int       `gorm:"column:quantity;not null;default:0"`
	Reserved  int       `gorm:"column:reserved;not null;default:0"`
	UpdatedAt time.Time `gorm:"column:updated_at"`

	Product *productModel `gorm:"foreignKey:ProductID;references:SerialID;constraint:OnDelete:CASCADE"`
}

func (inventoryModel) TableName() string { return "inventories" }

type GORMInventoryRepository struct {
	db *gorm.DB
}

func NewGORMInventoryRepository(db *gorm.DB) *GORMInventoryRepository {
	return &GORMInventoryRepository{db: db}
}

func (r *GORMInventoryRepository) EnsureDefault(ctx context.Context, productSerialID int64) error {
	now := time.Now().UTC()
	row := inventoryModel{ProductID: productSerialID}
	return r.db.WithContext(ctx).
		Where(inventoryModel{ProductID: productSerialID}).
		Attrs(inventoryModel{Quantity: 0, Reserved: 0, UpdatedAt: now}).
		FirstOrCreate(&row).Error
}

func (r *GORMInventoryRepository) GetByProductSerialID(ctx context.Context, productSerialID int64) (domain.Inventory, error) {
	var row inventoryModel
	err := r.db.WithContext(ctx).Where("product_id = ?", productSerialID).First(&row).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return domain.Inventory{}, domain.ErrInventoryNotFound
		}
		return domain.Inventory{}, err
	}
	return inventoryModelToDomain(row), nil
}

func (r *GORMInventoryRepository) GetByProductSerialIDForUpdate(ctx context.Context, productSerialID int64) (domain.Inventory, error) {
	var row inventoryModel
	err := r.db.WithContext(ctx).
		Clauses(clause.Locking{Strength: "UPDATE"}).
		Where("product_id = ?", productSerialID).
		First(&row).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return domain.Inventory{}, domain.ErrInventoryNotFound
		}
		return domain.Inventory{}, err
	}
	return inventoryModelToDomain(row), nil
}

func (r *GORMInventoryRepository) Update(ctx context.Context, inv domain.Inventory) (domain.Inventory, error) {
	inv.UpdatedAt = time.Now().UTC()
	res := r.db.WithContext(ctx).Model(&inventoryModel{}).
		Where("product_id = ?", inv.ProductSerialID).
		Updates(map[string]any{
			"quantity":   inv.Quantity,
			"reserved":   inv.Reserved,
			"updated_at": inv.UpdatedAt.UTC(),
		})
	if res.Error != nil {
		return domain.Inventory{}, res.Error
	}
	if res.RowsAffected == 0 {
		return domain.Inventory{}, domain.ErrInventoryNotFound
	}
	return r.GetByProductSerialID(ctx, inv.ProductSerialID)
}

func inventoryModelToDomain(m inventoryModel) domain.Inventory {
	return domain.Inventory{
		SerialID:        m.SerialID,
		ProductSerialID: m.ProductID,
		Quantity:        m.Quantity,
		Reserved:        m.Reserved,
		UpdatedAt:       m.UpdatedAt.UTC(),
	}
}
