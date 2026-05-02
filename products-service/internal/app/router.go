package app

import (
	"log/slog"

	"github.com/gin-gonic/gin"

	"products-service/internal/handler"
	"products-service/internal/middleware"
)

func NewRouter(
	products *handler.ProductHandler,
	inventories *handler.InventoryHandler,
	movements *handler.StockMovementHandler,
	logger *slog.Logger,
	jwtSecret string,
) *gin.Engine {
	r := gin.New()
	r.Use(gin.Recovery())
	r.Use(requestLogger(logger))

	v1 := r.Group("/api/v1")
	{
		v1.GET("/health", func(c *gin.Context) {
			c.JSON(200, gin.H{"status": "ok"})
		})

		withAuth := v1.Group("")
		withAuth.Use(middleware.JwtAuth(jwtSecret))
		{
			withAuth.POST("/products", products.Create)
			withAuth.GET("/products", products.List)

			// More specific paths before /products/:id
			withAuth.GET("/products/:id/inventory", inventories.GetForProduct)
			withAuth.PATCH("/products/:id/inventory", inventories.PatchForProduct)
			withAuth.GET("/products/:id/stock-movements", movements.ListForProduct)
			withAuth.POST("/products/:id/stock-movements", movements.CreateForProduct)

			withAuth.GET("/products/:id", products.GetByID)
			withAuth.PATCH("/products/:id", products.Update)
			withAuth.DELETE("/products/:id", products.Delete)
		}
	}

	r.NoRoute(func(c *gin.Context) {
		c.JSON(404, gin.H{
			"code":    "route_not_found",
			"message": "route not found",
		})
	})

	return r
}

func requestLogger(logger *slog.Logger) gin.HandlerFunc {
	return func(c *gin.Context) {
		logger.Info("request",
			slog.String("method", c.Request.Method),
			slog.String("path", c.Request.URL.Path),
		)
		c.Next()
	}
}
