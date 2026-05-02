package repository

import (
	"context"
	"time"

	"gorm.io/gorm"

	"products-service/internal/domain"
)

// StockMovementRepository appends ledger rows for stock changes.
type StockMovementRepository interface {
	Insert(ctx context.Context, m *domain.StockMovement) error
	ListByProductSerialID(ctx context.Context, productSerialID int64, limit, offset int) ([]domain.StockMovement, error)
}

type stockMovementModel struct {
	SerialID    int64     `gorm:"column:id;primaryKey;autoIncrement"`
	ProductID   int64     `gorm:"column:product_id;not null;index"`
	MoveType    string    `gorm:"column:type;type:stock_movement_type;not null"`
	Quantity    int       `gorm:"column:quantity;not null"`
	ReferenceID *string   `gorm:"column:reference_id"`
	Note        *string   `gorm:"column:note"`
	CreatedAt   time.Time `gorm:"column:created_at"`

	Product *productModel `gorm:"foreignKey:ProductID;references:SerialID;constraint:OnDelete:CASCADE"`
}

func (stockMovementModel) TableName() string { return "stock_movements" }

type GORMStockMovementRepository struct {
	db *gorm.DB
}

func NewGORMStockMovementRepository(db *gorm.DB) *GORMStockMovementRepository {
	return &GORMStockMovementRepository{db: db}
}

func (r *GORMStockMovementRepository) Insert(ctx context.Context, m *domain.StockMovement) error {
	now := time.Now().UTC()
	if m.CreatedAt.IsZero() {
		m.CreatedAt = now
	}
	row := stockMovementModel{
		ProductID:   m.ProductSerialID,
		MoveType:    string(m.Type),
		Quantity:    m.Quantity,
		ReferenceID: m.ReferenceID,
		Note:        m.Note,
		CreatedAt:   m.CreatedAt.UTC(),
	}
	if err := r.db.WithContext(ctx).Create(&row).Error; err != nil {
		return err
	}
	m.SerialID = row.SerialID
	m.CreatedAt = row.CreatedAt.UTC()
	return nil
}

func (r *GORMStockMovementRepository) ListByProductSerialID(
	ctx context.Context,
	productSerialID int64,
	limit, offset int,
) ([]domain.StockMovement, error) {
	if limit <= 0 {
		limit = 50
	}
	if limit > 200 {
		limit = 200
	}
	if offset < 0 {
		offset = 0
	}
	var rows []stockMovementModel
	err := r.db.WithContext(ctx).
		Where("product_id = ?", productSerialID).
		Order("created_at DESC, id DESC").
		Limit(limit).
		Offset(offset).
		Find(&rows).Error
	if err != nil {
		return nil, err
	}
	out := make([]domain.StockMovement, 0, len(rows))
	for _, row := range rows {
		out = append(out, movementModelToDomain(row))
	}
	return out, nil
}

func movementModelToDomain(m stockMovementModel) domain.StockMovement {
	return domain.StockMovement{
		SerialID:        m.SerialID,
		ProductSerialID: m.ProductID,
		Type:            domain.StockMovementType(m.MoveType),
		Quantity:        m.Quantity,
		ReferenceID:     m.ReferenceID,
		Note:            m.Note,
		CreatedAt:       m.CreatedAt.UTC(),
	}
}
