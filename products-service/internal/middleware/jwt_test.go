package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

func TestJwtAuth_OK(t *testing.T) {
	gin.SetMode(gin.TestMode)
	secret := "test-secret-at-least-32-bytes-long!!"

	r := gin.New()
	r.GET("/x", JwtAuth(secret), func(c *gin.Context) {
		cl, ok := c.Get(ContextJwtClaimsKey)
		if !ok {
			c.Status(500)
			return
		}
		claims := cl.(*JwtClaims)
		c.JSON(200, gin.H{"sub": claims.Subject})
	})

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, JwtClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user-123",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
		},
	})
	signed, err := token.SignedString([]byte(secret))
	if err != nil {
		t.Fatal(err)
	}

	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	req.Header.Set("Authorization", "Bearer "+signed)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)
	if w.Code != http.StatusOK {
		t.Fatalf("status=%d body=%s", w.Code, w.Body.String())
	}
}

func TestJwtAuth_missingHeader(t *testing.T) {
	gin.SetMode(gin.TestMode)
	r := gin.New()
	r.GET("/x", JwtAuth("secret"), func(c *gin.Context) { c.Status(200) })
	req := httptest.NewRequest(http.MethodGet, "/x", nil)
	w := httptest.NewRecorder()
	r.ServeHTTP(w, req)
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("want 401 got %d", w.Code)
	}
}
