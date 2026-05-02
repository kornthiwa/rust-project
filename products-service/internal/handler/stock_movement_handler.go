package handler

import (
	"net/http"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"products-service/internal/domain"
	"products-service/internal/service"
)

type StockMovementHandler struct {
	svc *service.StockMovementService
}

func NewStockMovementHandler(svc *service.StockMovementService) *StockMovementHandler {
	return &StockMovementHandler{svc: svc}
}

type movementJSON struct {
	ID          int64   `json:"id"`
	Type        string  `json:"type"`
	Quantity    int     `json:"quantity"`
	ReferenceID *string `json:"reference_id,omitempty"`
	Note        *string `json:"note,omitempty"`
	CreatedAt   string  `json:"created_at"`
}

func movementToJSON(m domain.StockMovement) movementJSON {
	return movementJSON{
		ID:          m.SerialID,
		Type:        string(m.Type),
		Quantity:    m.Quantity,
		ReferenceID: m.ReferenceID,
		Note:        m.Note,
		CreatedAt:   m.CreatedAt.UTC().Format(time.RFC3339Nano),
	}
}

type postStockMovementBody struct {
	Type        string  `json:"type" binding:"required"`
	Quantity    int     `json:"quantity"`
	ReferenceID *string `json:"reference_id"`
	Note        *string `json:"note"`
}

func (h *StockMovementHandler) CreateForProduct(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	var body postStockMovementBody
	if err := c.ShouldBindJSON(&body); err != nil {
		writeError(c, http.StatusBadRequest, "bad_request", "invalid JSON body")
		return
	}
	moveType := domain.StockMovementType(strings.TrimSpace(strings.ToUpper(body.Type)))
	if !domain.IsKnownMovementType(moveType) {
		writeError(c, http.StatusBadRequest, "invalid_movement_type", "unknown movement type")
		return
	}

	inv, mov, err := h.svc.Record(c.Request.Context(), id, service.RecordStockMovementInput{
		Type:        moveType,
		Quantity:    body.Quantity,
		ReferenceID: body.ReferenceID,
		Note:        body.Note,
	})
	if mapDomainErr(c, err) {
		return
	}
	c.JSON(http.StatusCreated, gin.H{
		"inventory": inventorySnapshotJSON(inv),
		"movement":  movementToJSON(mov),
	})
}

func inventorySnapshotJSON(inv domain.Inventory) gin.H {
	return gin.H{
		"quantity":   inv.Quantity,
		"reserved":   inv.Reserved,
		"updated_at": inv.UpdatedAt.UTC().Format(time.RFC3339Nano),
	}
}

func (h *StockMovementHandler) ListForProduct(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	limit := int32(parseIntDefault(c.Query("limit"), 50))
	offset := int32(parseIntDefault(c.Query("offset"), 0))
	list, err := h.svc.ListByProductUUID(c.Request.Context(), id, limit, offset)
	if mapDomainErr(c, err) {
		return
	}
	out := make([]movementJSON, 0, len(list))
	for _, m := range list {
		out = append(out, movementToJSON(m))
	}
	c.JSON(http.StatusOK, gin.H{"items": out})
}
