package middleware

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v5"
)

const ContextJwtClaimsKey = "jwt_claims"

// JwtClaims aligns with auth tokens carrying standard claims plus optional subject (sub).
type JwtClaims struct {
	jwt.RegisteredClaims
}

// JwtAuth validates Authorization: Bearer <JWT> using HS256 (same family as users-service).
func JwtAuth(secret string) gin.HandlerFunc {
	key := []byte(secret)
	return func(c *gin.Context) {
		raw := c.GetHeader("Authorization")
		if raw == "" {
			unauthorized(c, "missing_authorization", "missing Authorization header")
			return
		}
		const prefix = "Bearer "
		if !strings.HasPrefix(raw, prefix) {
			unauthorized(c, "invalid_authorization", "expected Bearer token")
			return
		}
		tokenStr := strings.TrimPrefix(raw, prefix)
		claims := &JwtClaims{}
		token, err := jwt.ParseWithClaims(tokenStr, claims, func(token *jwt.Token) (interface{}, error) {
			if token.Method.Alg() != jwt.SigningMethodHS256.Alg() {
				return nil, jwt.ErrSignatureInvalid
			}
			if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
				return nil, jwt.ErrSignatureInvalid
			}
			return key, nil
		})
		if err != nil || token == nil || !token.Valid {
			unauthorized(c, "invalid_token", "invalid or expired token")
			return
		}
		c.Set(ContextJwtClaimsKey, claims)
		c.Next()
	}
}

func unauthorized(c *gin.Context, code, msg string) {
	c.Header("WWW-Authenticate", `Bearer realm="api"`)
	c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"code": code, "message": msg})
}
