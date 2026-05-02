package service

import (
	"context"

	"github.com/google/uuid"
	"gorm.io/gorm"

	"products-service/internal/domain"
	"products-service/internal/repository"
)

// StockMovementService records ledger lines and keeps inventories in sync (one transaction).
type StockMovementService struct {
	db   *gorm.DB
	prod repository.ProductRepository
}

func NewStockMovementService(db *gorm.DB, prod repository.ProductRepository) *StockMovementService {
	return &StockMovementService{db: db, prod: prod}
}

type RecordStockMovementInput struct {
	Type        domain.StockMovementType
	Quantity    int
	ReferenceID *string
	Note        *string
}

func (s *StockMovementService) Record(
	ctx context.Context,
	productUUID uuid.UUID,
	in RecordStockMovementInput,
) (domain.Inventory, domain.StockMovement, error) {
	if !domain.IsKnownMovementType(in.Type) {
		return domain.Inventory{}, domain.StockMovement{}, domain.ErrInvalidInput
	}

	var outInv domain.Inventory
	var outMov domain.StockMovement

	err := s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		prodTx := repository.NewGORMProductRepository(tx)
		invTx := repository.NewGORMInventoryRepository(tx)
		movTx := repository.NewGORMStockMovementRepository(tx)

		pid, err := prodTx.GetSerialIDByUUID(ctx, productUUID)
		if err != nil {
			return err
		}
		if err := invTx.EnsureDefault(ctx, pid); err != nil {
			return err
		}
		inv, err := invTx.GetByProductSerialIDForUpdate(ctx, pid)
		if err != nil {
			return err
		}
		if err := domain.ApplyStockMovement(&inv, in.Type, in.Quantity); err != nil {
			return err
		}
		inv, err = invTx.Update(ctx, inv)
		if err != nil {
			return err
		}
		mov := &domain.StockMovement{
			ProductSerialID: pid,
			Type:            in.Type,
			Quantity:        in.Quantity,
			ReferenceID:     in.ReferenceID,
			Note:            in.Note,
		}
		if err := movTx.Insert(ctx, mov); err != nil {
			return err
		}
		outInv = inv
		outMov = *mov
		return nil
	})
	return outInv, outMov, err
}

func (s *StockMovementService) ListByProductUUID(
	ctx context.Context,
	productUUID uuid.UUID,
	limit, offset int32,
) ([]domain.StockMovement, error) {
	pid, err := s.prod.GetSerialIDByUUID(ctx, productUUID)
	if err != nil {
		return nil, err
	}
	movRepo := repository.NewGORMStockMovementRepository(s.db)
	return movRepo.ListByProductSerialID(ctx, pid, int(limit), int(offset))
}
