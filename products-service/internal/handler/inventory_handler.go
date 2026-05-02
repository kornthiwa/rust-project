package handler

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"products-service/internal/domain"
	"products-service/internal/service"
)

type InventoryHandler struct {
	svc *service.InventoryService
}

func NewInventoryHandler(svc *service.InventoryService) *InventoryHandler {
	return &InventoryHandler{svc: svc}
}

type inventoryJSON struct {
	Quantity  int    `json:"quantity"`
	Reserved  int    `json:"reserved"`
	UpdatedAt string `json:"updated_at"`
}

func toInventoryJSON(i domain.Inventory) inventoryJSON {
	return inventoryJSON{
		Quantity:  i.Quantity,
		Reserved:  i.Reserved,
		UpdatedAt: i.UpdatedAt.UTC().Format(time.RFC3339Nano),
	}
}

func (h *InventoryHandler) GetForProduct(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	inv, err := h.svc.GetForProductUUID(c.Request.Context(), id)
	if mapDomainErr(c, err) {
		return
	}
	c.JSON(http.StatusOK, toInventoryJSON(inv))
}

type patchInventoryBody struct {
	Quantity *int `json:"quantity"`
	Reserved *int `json:"reserved"`
}

func (h *InventoryHandler) PatchForProduct(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	var body patchInventoryBody
	if err := c.ShouldBindJSON(&body); err != nil {
		writeError(c, http.StatusBadRequest, "bad_request", "invalid JSON body")
		return
	}
	inv, err := h.svc.PatchForProductUUID(c.Request.Context(), id, service.PatchInventoryInput{
		Quantity: body.Quantity,
		Reserved: body.Reserved,
	})
	if mapDomainErr(c, err) {
		return
	}
	c.JSON(http.StatusOK, toInventoryJSON(inv))
}
