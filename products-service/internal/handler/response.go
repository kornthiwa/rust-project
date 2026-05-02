package handler

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"products-service/internal/domain"
)

type errorBody struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

func writeError(c *gin.Context, status int, code, message string) {
	c.JSON(status, errorBody{Code: code, Message: message})
}

func mapDomainErr(c *gin.Context, err error) bool {
	switch {
	case err == nil:
		return false
	case err == domain.ErrProductNotFound:
		writeError(c, http.StatusNotFound, "product_not_found", "product not found")
		return true
	case err == domain.ErrInvalidInput:
		writeError(c, http.StatusBadRequest, "invalid_input", "invalid input")
		return true
	case err == domain.ErrInventoryNotFound:
		writeError(c, http.StatusNotFound, "inventory_not_found", "inventory not found")
		return true
	case err == domain.ErrInventoryInvariant:
		writeError(c, http.StatusBadRequest, "inventory_invariant", "reserved quantity exceeds available quantity")
		return true
	case err == domain.ErrInsufficientStock:
		writeError(c, http.StatusConflict, "insufficient_stock", "not enough stock for this movement")
		return true
	default:
		writeError(c, http.StatusInternalServerError, "internal_error", "something went wrong")
		return true
	}
}
