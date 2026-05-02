package repository

import (
	"gorm.io/gorm"
)

const ensureStockMovementEnumSQL = `
DO $$ BEGIN
    CREATE TYPE stock_movement_type AS ENUM ('IN', 'OUT', 'RESERVE', 'RELEASE', 'ADJUST');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;
`

// AutoMigrate applies schema for products, inventories, and stock_movements.
func AutoMigrate(db *gorm.DB) error {
	if err := db.Exec(ensureStockMovementEnumSQL).Error; err != nil {
		return err
	}
	return db.AutoMigrate(&productModel{}, &inventoryModel{}, &stockMovementModel{})
}
