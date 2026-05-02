package handler

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

	"products-service/internal/domain"
	"products-service/internal/service"
)

type ProductHandler struct {
	svc *service.ProductService
}

func NewProductHandler(svc *service.ProductService) *ProductHandler {
	return &ProductHandler{svc: svc}
}

type productJSON struct {
	ID        string  `json:"id"`
	Name      string  `json:"name"`
	SKU       *string `json:"sku,omitempty"`
	CreatedAt string  `json:"created_at"`
	UpdatedAt string  `json:"updated_at"`
}

func toProductJSON(p domain.Product) productJSON {
	return productJSON{
		ID:        p.ID.String(),
		Name:      p.Name,
		SKU:       p.SKU,
		CreatedAt: p.CreatedAt.UTC().Format(time.RFC3339Nano),
		UpdatedAt: p.UpdatedAt.UTC().Format(time.RFC3339Nano),
	}
}

type createProductRequest struct {
	Name string  `json:"name"`
	SKU  *string `json:"sku"`
}

func (h *ProductHandler) Create(c *gin.Context) {
	var body createProductRequest
	if err := c.ShouldBindJSON(&body); err != nil {
		writeError(c, http.StatusBadRequest, "bad_request", "invalid JSON body")
		return
	}
	p, err := h.svc.Create(c.Request.Context(), service.CreateProductInput{
		Name: body.Name,
		SKU:  body.SKU,
	})
	if mapDomainErr(c, err) {
		return
	}
	c.JSON(http.StatusCreated, toProductJSON(p))
}

func (h *ProductHandler) GetByID(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	p, err := h.svc.GetByID(c.Request.Context(), id)
	if mapDomainErr(c, err) {
		return
	}
	c.JSON(http.StatusOK, toProductJSON(p))
}

func (h *ProductHandler) List(c *gin.Context) {
	limit := int32(parseIntDefault(c.Query("limit"), 50))
	offset := int32(parseIntDefault(c.Query("offset"), 0))
	list, err := h.svc.List(c.Request.Context(), limit, offset)
	if mapDomainErr(c, err) {
		return
	}
	out := make([]productJSON, 0, len(list))
	for _, p := range list {
		out = append(out, toProductJSON(p))
	}
	c.JSON(http.StatusOK, gin.H{"items": out})
}

type patchProductRequest struct {
	Name *string `json:"name"`
	SKU  *string `json:"sku"`
}

func (h *ProductHandler) Update(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	var body patchProductRequest
	if err := c.ShouldBindJSON(&body); err != nil {
		writeError(c, http.StatusBadRequest, "bad_request", "invalid JSON body")
		return
	}
	p, err := h.svc.Update(c.Request.Context(), id, service.UpdateProductInput{
		Name: body.Name,
		SKU:  body.SKU,
	})
	if mapDomainErr(c, err) {
		return
	}
	c.JSON(http.StatusOK, toProductJSON(p))
}

func (h *ProductHandler) Delete(c *gin.Context) {
	id, err := uuid.Parse(c.Param("id"))
	if err != nil {
		writeError(c, http.StatusBadRequest, "invalid_id", "invalid product id")
		return
	}
	err = h.svc.Delete(c.Request.Context(), id)
	if mapDomainErr(c, err) {
		return
	}
	c.Status(http.StatusNoContent)
}

func parseIntDefault(s string, def int) int {
	if s == "" {
		return def
	}
	v, err := strconv.Atoi(s)
	if err != nil {
		return def
	}
	return v
}
