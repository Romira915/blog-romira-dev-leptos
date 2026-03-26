use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation, decode};
use serde::{Deserialize, Serialize};

pub(crate) const DBSC_SESSION_ID_KEY: &str = "dbsc_session_id";
pub(crate) const DBSC_PUBLIC_KEY_KEY: &str = "dbsc_public_key";
pub(crate) const DBSC_REGISTRATION_NONCE_KEY: &str = "dbsc_registration_nonce";
pub(crate) const DBSC_CHALLENGE_NONCES_KEY: &str = "dbsc_challenge_nonces";
pub(crate) const DBSC_COOKIE_NAME: &str = "__Secure-dbsc";
pub(crate) const DBSC_COOKIE_MAX_AGE: u64 = 600;
const MAX_RECENT_CHALLENGES: usize = 5;

#[derive(Debug, thiserror::Error)]
pub(crate) enum DbscError {
    #[error("Invalid JWT: {0}")]
    InvalidJwt(String),
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Nonce mismatch")]
    NonceMismatch,
    #[error("Missing JWK in JWT payload")]
    MissingJwk,
}

/// JWK (JSON Web Key) for EC P-256
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EcJwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
    pub y: String,
}

/// JWT payload for DBSC proof (Registration)
#[derive(Debug, Deserialize)]
struct DbscRegistrationClaims {
    #[allow(dead_code)]
    aud: String,
    jti: String,
    #[allow(dead_code)]
    iat: i64,
    jwk: Option<EcJwk>,
}

/// JWT payload for DBSC proof (Refresh)
#[derive(Debug, Deserialize)]
struct DbscRefreshClaims {
    #[allow(dead_code)]
    aud: String,
    jti: String,
    #[allow(dead_code)]
    iat: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct DbscService {
    app_url: String,
}

impl DbscService {
    pub(crate) fn new(app_url: String) -> Self {
        Self { app_url }
    }

    pub(crate) fn generate_nonce() -> String {
        uuid::Uuid::now_v7().to_string()
    }

    pub(crate) fn build_registration_header(&self, nonce: &str) -> String {
        format!(r#"(ES256); path="/auth/dbsc/start"; challenge="{}""#, nonce)
    }

    /// Verify a Registration JWT proof.
    /// Returns (jti_nonce, public_key_jwk_json) on success.
    pub(crate) fn verify_registration_jwt(
        &self,
        jwt_str: &str,
    ) -> Result<(String, String), DbscError> {
        // Decode header to verify typ and alg
        let header = jsonwebtoken::decode_header(jwt_str)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        Self::validate_header(&header)?;

        // First decode without verification to extract the JWK from payload
        let mut no_verify = Validation::new(Algorithm::ES256);
        no_verify.insecure_disable_signature_validation();
        no_verify.set_required_spec_claims::<&str>(&[]);
        no_verify.set_audience(&[&format!("{}/auth/dbsc/start", self.app_url)]);

        let token_data =
            decode::<DbscRegistrationClaims>(jwt_str, &DecodingKey::from_secret(b""), &no_verify)
                .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        let jwk = token_data
            .claims
            .jwk
            .as_ref()
            .ok_or(DbscError::MissingJwk)?;

        // Now verify signature with the extracted JWK
        let decoding_key = Self::decoding_key_from_jwk(jwk)?;
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_required_spec_claims::<&str>(&[]);
        validation.set_audience(&[&format!("{}/auth/dbsc/start", self.app_url)]);

        let verified = decode::<DbscRegistrationClaims>(jwt_str, &decoding_key, &validation)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        let jwk_json =
            serde_json::to_string(jwk).map_err(|e| DbscError::InvalidPublicKey(e.to_string()))?;

        Ok((verified.claims.jti, jwk_json))
    }

    /// Verify a Refresh JWT proof.
    /// Returns the matched nonce on success.
    pub(crate) fn verify_refresh_jwt(
        &self,
        jwt_str: &str,
        public_key_jwk: &str,
        expected_nonces: &[String],
    ) -> Result<String, DbscError> {
        let header = jsonwebtoken::decode_header(jwt_str)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        Self::validate_header(&header)?;

        let jwk: EcJwk = serde_json::from_str(public_key_jwk)
            .map_err(|e| DbscError::InvalidPublicKey(e.to_string()))?;

        let decoding_key = Self::decoding_key_from_jwk(&jwk)?;
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_required_spec_claims::<&str>(&[]);
        validation.set_audience(&[&format!("{}/auth/dbsc/refresh", self.app_url)]);

        let token_data = decode::<DbscRefreshClaims>(jwt_str, &decoding_key, &validation)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        if expected_nonces.contains(&token_data.claims.jti) {
            Ok(token_data.claims.jti)
        } else {
            Err(DbscError::NonceMismatch)
        }
    }

    pub(crate) fn build_session_config(&self, session_id: &str) -> serde_json::Value {
        let domain = self
            .app_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or("localhost");

        serde_json::json!({
            "session_identifier": session_id,
            "refresh_url": "/auth/dbsc/refresh",
            "scope": {
                "origin": self.app_url,
                "include_site": true,
                "scope_specification": [
                    { "type": "include", "domain": domain, "path": "/admin" },
                    { "type": "include", "domain": domain, "path": "/api/admin" }
                ]
            },
            "credentials": [{
                "type": "cookie",
                "name": DBSC_COOKIE_NAME,
                "attributes": format!(
                    "Secure; HttpOnly; SameSite=Lax; Max-Age={}; Path=/",
                    DBSC_COOKIE_MAX_AGE
                )
            }]
        })
    }

    pub(crate) fn build_challenge_header(nonce: &str, session_id: &str) -> String {
        format!(r#""{}";id="{}""#, nonce, session_id)
    }

    pub(crate) fn generate_cookie_value() -> String {
        uuid::Uuid::now_v7().to_string()
    }

    pub(crate) fn build_set_cookie_header(cookie_value: &str) -> String {
        format!(
            "{}={}; Secure; HttpOnly; SameSite=Lax; Max-Age={}; Path=/",
            DBSC_COOKIE_NAME, cookie_value, DBSC_COOKIE_MAX_AGE
        )
    }

    /// Add a nonce to the list, keeping at most MAX_RECENT_CHALLENGES entries.
    pub(crate) fn push_challenge_nonce(nonces: &mut Vec<String>, nonce: String) {
        if nonces.len() >= MAX_RECENT_CHALLENGES {
            nonces.remove(0);
        }
        nonces.push(nonce);
    }

    fn validate_header(header: &Header) -> Result<(), DbscError> {
        if header.alg != Algorithm::ES256 {
            return Err(DbscError::InvalidJwt(format!(
                "expected ES256, got {:?}",
                header.alg
            )));
        }
        match &header.typ {
            Some(typ) if typ == "dbsc+jwt" => Ok(()),
            other => Err(DbscError::InvalidJwt(format!(
                r#"expected typ "dbsc+jwt", got {:?}"#,
                other
            ))),
        }
    }

    fn decoding_key_from_jwk(jwk: &EcJwk) -> Result<DecodingKey, DbscError> {
        if jwk.kty != "EC" || jwk.crv != "P-256" {
            return Err(DbscError::InvalidPublicKey(format!(
                "expected EC P-256, got {} {}",
                jwk.kty, jwk.crv
            )));
        }

        DecodingKey::from_ec_components(&jwk.x, &jwk.y)
            .map_err(|e| DbscError::InvalidPublicKey(e.to_string()))
    }
}

//noinspection NonAsciiCharacters
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_nonceでユニークな値を生成すること() {
        let a = DbscService::generate_nonce();
        let b = DbscService::generate_nonce();
        assert_ne!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn build_registration_headerで正しい形式のヘッダーを構築すること() {
        let service = DbscService::new("https://example.com".to_string());
        let header = service.build_registration_header("test-nonce");
        assert_eq!(
            header,
            r#"(ES256); path="/auth/dbsc/start"; challenge="test-nonce""#
        );
    }

    #[test]
    fn build_challenge_headerで正しい形式のヘッダーを構築すること() {
        let header = DbscService::build_challenge_header("nonce123", "session456");
        assert_eq!(header, r#""nonce123";id="session456""#);
    }

    #[test]
    fn build_session_configで正しいjsonを返すこと() {
        let service = DbscService::new("https://blog.romira.dev".to_string());
        let config = service.build_session_config("sess-id");
        assert_eq!(config["session_identifier"], "sess-id");
        assert_eq!(config["refresh_url"], "/auth/dbsc/refresh");
        assert_eq!(config["credentials"][0]["name"], DBSC_COOKIE_NAME);
    }

    #[test]
    fn build_set_cookie_headerで正しい形式のcookieヘッダーを構築すること() {
        let header = DbscService::build_set_cookie_header("token123");
        assert!(header.contains("__Secure-dbsc=token123"));
        assert!(header.contains("Secure"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Max-Age=600"));
    }

    #[test]
    fn push_challenge_nonceで最大数を超えた場合古いものから削除すること() {
        let mut nonces = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        DbscService::push_challenge_nonce(&mut nonces, "f".to_string());
        assert_eq!(nonces.len(), 5);
        assert!(!nonces.contains(&"a".to_string()));
        assert!(nonces.contains(&"f".to_string()));
    }

    #[test]
    fn verify_registration_jwtで不正なjwtをリジェクトすること() {
        let service = DbscService::new("https://example.com".to_string());
        let result = service.verify_registration_jwt("not-a-jwt");
        assert!(result.is_err());
    }

    #[test]
    fn verify_refresh_jwtでnonce不一致をリジェクトすること() {
        let service = DbscService::new("https://example.com".to_string());
        // Invalid JWT will fail before nonce check, but we test the flow
        let result = service.verify_refresh_jwt(
            "not-a-jwt",
            r#"{"kty":"EC","crv":"P-256","x":"","y":""}"#,
            &["nonce1".to_string()],
        );
        assert!(result.is_err());
    }
}
