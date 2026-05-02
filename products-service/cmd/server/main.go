package main

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/joho/godotenv"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"

	"products-service/internal/app"
	"products-service/internal/config"
	"products-service/internal/handler"
	"products-service/internal/repository"
	"products-service/internal/service"
)

func main() {
	// Local dev: load .env into the process env (missing file is OK; production uses real env vars).
	_ = godotenv.Load()

	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))

	port, databaseURL, jwtSecret, err := config.Load()
	if err != nil {
		logger.Error("config", slog.String("error", err.Error()))
		os.Exit(1)
	}

	gormDB, err := gorm.Open(postgres.Open(databaseURL), &gorm.Config{})
	if err != nil {
		logger.Error("db_open", slog.String("error", err.Error()))
		os.Exit(1)
	}

	sqlDB, err := gormDB.DB()
	if err != nil {
		logger.Error("db_sql", slog.String("error", err.Error()))
		os.Exit(1)
	}
	sqlDB.SetMaxIdleConns(8)
	sqlDB.SetMaxOpenConns(32)
	sqlDB.SetConnMaxLifetime(time.Hour)
	defer func() {
		if cerr := sqlDB.Close(); cerr != nil {
			logger.Error("db_close", slog.String("error", cerr.Error()))
		}
	}()

	if err := repository.AutoMigrate(gormDB); err != nil {
		logger.Error("migrate", slog.String("error", err.Error()))
		os.Exit(1)
	}

	prodRepo := repository.NewGORMProductRepository(gormDB)
	invRepo := repository.NewGORMInventoryRepository(gormDB)

	prodSvc := service.NewProductService(prodRepo, invRepo)
	invSvc := service.NewInventoryService(prodRepo, invRepo)

	movSvc := service.NewStockMovementService(gormDB, prodRepo)

	prodHandler := handler.NewProductHandler(prodSvc)
	invHandler := handler.NewInventoryHandler(invSvc)
	movHandler := handler.NewStockMovementHandler(movSvc)
	r := app.NewRouter(prodHandler, invHandler, movHandler, logger, jwtSecret)

	srv := &http.Server{
		Addr:              ":" + port,
		Handler:           r,
		ReadHeaderTimeout: 10 * time.Second,
		ReadTimeout:       30 * time.Second,
		WriteTimeout:      30 * time.Second,
		IdleTimeout:       60 * time.Second,
	}

	go func() {
		logger.Info("listening", slog.String("addr", srv.Addr))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Error("server", slog.String("error", err.Error()))
			os.Exit(1)
		}
	}()

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig

	shutdownCtx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()
	if err := srv.Shutdown(shutdownCtx); err != nil {
		logger.Error("shutdown", slog.String("error", err.Error()))
		_ = srv.Close()
		os.Exit(1)
	}
	logger.Info("stopped")
}
