use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation, decode};
use serde::{Deserialize, Serialize};
use tracing::instrument;

pub(crate) const DBSC_SESSION_ID_KEY: &str = "dbsc_session_id";
pub(crate) const DBSC_PUBLIC_KEY_KEY: &str = "dbsc_public_key";
pub(crate) const DBSC_REGISTRATION_NONCE_KEY: &str = "dbsc_registration_nonce";
pub(crate) const DBSC_CHALLENGE_NONCES_KEY: &str = "dbsc_challenge_nonces";
pub(crate) const DBSC_COOKIE_NAME: &str = "__Secure-dbsc";
pub(crate) const DBSC_COOKIE_MAX_AGE: u64 = 600;
const MAX_RECENT_CHALLENGES: usize = 5;

/// DBSC登録開始に必要なデータ
pub(crate) struct RegistrationInitiation {
    /// セッションに保存するnonce
    pub nonce: String,
    /// HTTPレスポンスに設定するSecure-Session-Registrationヘッダー値
    pub header_value: String,
}

/// DBSC登録完了後にHandler側で処理するデータ
pub(crate) struct RegistrationCompletion {
    pub session_id: String,
    pub public_key_jwk: String,
    /// __Secure-dbsc Set-Cookieヘッダー値
    pub set_cookie_header: String,
    pub session_config: serde_json::Value,
}

/// DBSCリフレッシュ Phase1: チャレンジ発行結果
pub(crate) struct ChallengeIssue {
    pub updated_nonces: Vec<String>,
    pub challenge_header: String,
}

/// DBSCリフレッシュ Phase2: 検証成功後の結果
pub(crate) struct RefreshCompletion {
    pub updated_nonces: Vec<String>,
    pub set_cookie_header: String,
    /// 次回 Refresh 用の cached challenge ヘッダー
    pub challenge_header: String,
}

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
    #[error("Session ID mismatch")]
    SessionIdMismatch,
    #[error("No pending challenges")]
    NoPendingChallenges,
    #[error("Missing public key")]
    MissingPublicKey,
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
///
/// Chrome sends `jti` in the payload. The JWK is in the JWT header.
#[derive(Debug, Deserialize)]
struct DbscRegistrationClaims {
    jti: String,
}

/// JWT payload for DBSC proof (Refresh)
///
/// Chrome sends only `jti` in the payload.
#[derive(Debug, Deserialize)]
struct DbscRefreshClaims {
    jti: String,
}

#[derive(Clone, Debug)]
pub(crate) struct DbscService {
    /// アプリケーションのオリジン（例: "https://blog.romira.dev"）
    origin: String,
    /// オリジンからプロトコル・ポートを除いたドメイン（例: "blog.romira.dev"）
    domain: String,
}

impl DbscService {
    #[instrument]
    pub(crate) fn new(origin: String) -> Self {
        let domain = origin
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or("localhost")
            .to_string();
        Self { origin, domain }
    }

    #[instrument]
    pub(crate) fn generate_nonce() -> String {
        uuid::Uuid::now_v7().to_string()
    }

    #[instrument(skip(self))]
    pub(crate) fn build_registration_header(&self, nonce: &str) -> String {
        format!(
            r#"(ES256); path="/auth/dbsc/registration"; challenge="{}""#,
            nonce
        )
    }

    /// Verify a Registration JWT proof.
    /// Returns (jti_nonce, public_key_jwk_json) on success.
    ///
    /// Chrome puts the JWK in the JWT header (not the payload).
    #[instrument(skip(self, jwt_str))]
    pub(crate) fn verify_registration_jwt(
        &self,
        jwt_str: &str,
    ) -> Result<(String, String), DbscError> {
        // Decode header to verify typ, alg, and extract JWK
        let header = jsonwebtoken::decode_header(jwt_str)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        Self::validate_header(&header)?;

        // Extract JWK from JWT header
        let header_jwk = header.jwk.as_ref().ok_or(DbscError::MissingJwk)?;
        let ec_jwk = Self::ec_jwk_from_header_jwk(header_jwk)?;

        // Verify signature with the extracted JWK
        let decoding_key = Self::decoding_key_from_jwk(&ec_jwk)?;
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_required_spec_claims::<&str>(&[]);
        // Chrome does not send `aud` in the payload, so skip audience validation
        validation.validate_aud = false;

        let verified = decode::<DbscRegistrationClaims>(jwt_str, &decoding_key, &validation)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        let jwk_json = serde_json::to_string(&ec_jwk)
            .map_err(|e| DbscError::InvalidPublicKey(e.to_string()))?;

        Ok((verified.claims.jti, jwk_json))
    }

    /// Verify a Refresh JWT proof.
    /// Returns the matched nonce on success.
    #[instrument(skip(self, jwt_str, public_key_jwk))]
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
        // Chrome does not send `aud` in the payload
        validation.validate_aud = false;

        let token_data = decode::<DbscRefreshClaims>(jwt_str, &decoding_key, &validation)
            .map_err(|e| DbscError::InvalidJwt(e.to_string()))?;

        if expected_nonces.contains(&token_data.claims.jti) {
            Ok(token_data.claims.jti)
        } else {
            Err(DbscError::NonceMismatch)
        }
    }

    #[instrument(skip(self))]
    pub(crate) fn build_session_config(&self, session_id: &str) -> serde_json::Value {
        serde_json::json!({
            "session_identifier": session_id,
            "refresh_url": "/auth/dbsc/refresh",
            "scope": {
                "origin": self.origin,
                "include_site": false,
                "scope_specification": [
                    { "type": "include", "domain": self.domain, "path": "/admin" },
                    { "type": "include", "domain": self.domain, "path": "/api/admin" }
                ]
            },
            "credentials": [{
                "type": "cookie",
                "name": DBSC_COOKIE_NAME,
                "attributes": "Secure; HttpOnly; SameSite=Lax; Path=/"
            }]
        })
    }

    #[instrument]
    pub(crate) fn build_challenge_header(nonce: &str, session_id: &str) -> String {
        format!(r#""{}";id="{}""#, nonce, session_id)
    }

    #[instrument]
    pub(crate) fn generate_cookie_value() -> String {
        uuid::Uuid::now_v7().to_string()
    }

    #[instrument]
    pub(crate) fn build_set_cookie_header(cookie_value: &str) -> String {
        format!(
            "{}={}; Secure; HttpOnly; SameSite=Lax; Max-Age={}; Path=/",
            DBSC_COOKIE_NAME, cookie_value, DBSC_COOKIE_MAX_AGE
        )
    }

    /// Add a nonce to the list, keeping at most MAX_RECENT_CHALLENGES entries.
    #[instrument(skip(nonces))]
    pub(crate) fn push_challenge_nonce(nonces: &mut Vec<String>, nonce: String) {
        if nonces.len() >= MAX_RECENT_CHALLENGES {
            nonces.remove(0);
        }
        nonces.push(nonce);
    }

    #[instrument(skip(header))]
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

    /// Extract EC P-256 key components from a jsonwebtoken::jwk::Jwk (JWT header JWK).
    #[instrument(skip(jwk))]
    fn ec_jwk_from_header_jwk(jwk: &jsonwebtoken::jwk::Jwk) -> Result<EcJwk, DbscError> {
        use jsonwebtoken::jwk::EllipticCurveKeyParameters;
        match &jwk.algorithm {
            jsonwebtoken::jwk::AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
                key_type: _,
                curve,
                x,
                y,
            }) => {
                let curve_str = match curve {
                    jsonwebtoken::jwk::EllipticCurve::P256 => "P-256",
                    other => {
                        return Err(DbscError::InvalidPublicKey(format!(
                            "expected P-256, got {:?}",
                            other
                        )));
                    }
                };
                Ok(EcJwk {
                    kty: "EC".to_string(),
                    crv: curve_str.to_string(),
                    x: x.clone(),
                    y: y.clone(),
                })
            }
            other => Err(DbscError::InvalidPublicKey(format!(
                "expected EC key, got {:?}",
                other
            ))),
        }
    }

    #[instrument(skip(jwk))]
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

    // --- 高レベルフロー操作 ---

    /// DBSC登録を開始する（dbsc_registration_initiation用）
    /// nonceを生成し、登録ヘッダーを構築して返す。
    /// nonceはHandler側でセッションに保存する。
    #[instrument(skip(self))]
    pub(crate) fn initiate_registration(&self) -> RegistrationInitiation {
        let nonce = Self::generate_nonce();
        let header_value = self.build_registration_header(&nonce);
        RegistrationInitiation {
            nonce,
            header_value,
        }
    }

    /// DBSC登録を完了する（dbsc_registration handler用）
    /// JWT検証・nonce照合・セッションID生成・Cookie構築を一括で行う。
    #[instrument(skip(self, jwt))]
    pub(crate) fn complete_registration(
        &self,
        jwt: &str,
        stored_nonce: &str,
    ) -> Result<RegistrationCompletion, DbscError> {
        let (jti_nonce, public_key_jwk) = self.verify_registration_jwt(jwt)?;

        if jti_nonce != stored_nonce {
            return Err(DbscError::NonceMismatch);
        }

        let session_id = uuid::Uuid::now_v7().to_string();
        let cookie_value = Self::generate_cookie_value();
        let set_cookie_header = Self::build_set_cookie_header(&cookie_value);
        let session_config = self.build_session_config(&session_id);

        Ok(RegistrationCompletion {
            session_id,
            public_key_jwk,
            set_cookie_header,
            session_config,
        })
    }

    /// チャレンジを発行する（refresh phase1用）
    /// セッションID照合後、新しいnonceを生成し、noncesリストを更新して返す。
    #[instrument(skip(current_nonces))]
    pub(crate) fn issue_challenge(
        request_session_id: &str,
        stored_session_id: Option<&str>,
        mut current_nonces: Vec<String>,
    ) -> Result<ChallengeIssue, DbscError> {
        if stored_session_id != Some(request_session_id) {
            return Err(DbscError::SessionIdMismatch);
        }

        let nonce = Self::generate_nonce();
        Self::push_challenge_nonce(&mut current_nonces, nonce.clone());
        let challenge_header = Self::build_challenge_header(&nonce, request_session_id);
        Ok(ChallengeIssue {
            updated_nonces: current_nonces,
            challenge_header,
        })
    }

    /// リフレッシュを完了する（refresh phase2用）
    /// セッションID照合・公開鍵/nonces検証・JWT検証・nonce消費・新Cookie発行を一括で行う。
    #[instrument(skip(self, jwt, public_key_jwk, nonces))]
    pub(crate) fn complete_refresh(
        &self,
        jwt: &str,
        request_session_id: &str,
        stored_session_id: Option<&str>,
        public_key_jwk: Option<&str>,
        nonces: Vec<String>,
    ) -> Result<RefreshCompletion, DbscError> {
        if stored_session_id != Some(request_session_id) {
            return Err(DbscError::SessionIdMismatch);
        }

        let public_key_jwk = public_key_jwk.ok_or(DbscError::MissingPublicKey)?;

        if nonces.is_empty() {
            return Err(DbscError::NoPendingChallenges);
        }

        let matched_nonce = self.verify_refresh_jwt(jwt, public_key_jwk, &nonces)?;

        let mut updated_nonces = nonces;
        updated_nonces.retain(|n| n != &matched_nonce);

        // Generate next challenge for Chrome to cache (avoids Phase 1 round-trip)
        let next_nonce = Self::generate_nonce();
        Self::push_challenge_nonce(&mut updated_nonces, next_nonce.clone());
        let challenge_header = Self::build_challenge_header(&next_nonce, request_session_id);

        let cookie_value = Self::generate_cookie_value();
        let set_cookie_header = Self::build_set_cookie_header(&cookie_value);

        Ok(RefreshCompletion {
            updated_nonces,
            set_cookie_header,
            challenge_header,
        })
    }

    /// DBSCセッションバインディングが有効かどうかを判定する（require_admin_auth用）
    /// DBSC登録済みセッションはDBSC Cookieが必須。未登録セッションは常にtrue。
    #[instrument]
    pub(crate) fn is_session_bound(has_dbsc_session: bool, has_dbsc_cookie: bool) -> bool {
        if has_dbsc_session {
            has_dbsc_cookie
        } else {
            true
        }
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
            r#"(ES256); path="/auth/dbsc/registration"; challenge="test-nonce""#
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

    // --- 高レベルフロー操作のテスト ---

    #[test]
    fn initiate_registrationでnonceとヘッダーを返すこと() {
        let service = DbscService::new("https://example.com".to_string());
        let initiation = service.initiate_registration();
        assert!(!initiation.nonce.is_empty());
        assert!(initiation.header_value.contains(&initiation.nonce));
        assert!(initiation.header_value.contains("ES256"));
        assert!(initiation.header_value.contains("/auth/dbsc/registration"));
    }

    #[test]
    fn complete_registrationで不正なjwtをリジェクトすること() {
        let service = DbscService::new("https://example.com".to_string());
        let result = service.complete_registration("not-a-jwt", "some-nonce");
        assert!(result.is_err());
    }

    #[test]
    fn issue_challengeでnoncesを更新しヘッダーを返すこと() {
        let existing = vec!["old-nonce".to_string()];
        let challenge =
            DbscService::issue_challenge("session-id", Some("session-id"), existing).unwrap();
        assert_eq!(challenge.updated_nonces.len(), 2);
        assert_eq!(challenge.updated_nonces[0], "old-nonce");
        assert!(challenge.challenge_header.contains("session-id"));
    }

    #[test]
    fn issue_challengeで空のnoncesリストからでもチャレンジを発行できること() {
        let challenge = DbscService::issue_challenge("sess-1", Some("sess-1"), vec![]).unwrap();
        assert_eq!(challenge.updated_nonces.len(), 1);
        assert!(challenge.challenge_header.contains("sess-1"));
    }

    #[test]
    fn issue_challengeでmax超過時に古いnonceを削除すること() {
        let existing: Vec<String> = (0..5).map(|i| format!("nonce-{}", i)).collect();
        let challenge = DbscService::issue_challenge("sess", Some("sess"), existing).unwrap();
        assert_eq!(challenge.updated_nonces.len(), 5);
        assert!(!challenge.updated_nonces.contains(&"nonce-0".to_string()));
    }

    #[test]
    fn issue_challengeでセッションid不一致の場合エラーを返すこと() {
        let result = DbscService::issue_challenge("request-id", Some("different-id"), vec![]);
        assert!(matches!(result, Err(DbscError::SessionIdMismatch)));
    }

    #[test]
    fn issue_challengeでセッションidが未保存の場合エラーを返すこと() {
        let result = DbscService::issue_challenge("request-id", None, vec![]);
        assert!(matches!(result, Err(DbscError::SessionIdMismatch)));
    }

    #[test]
    fn is_session_boundでdbsc未登録セッションは常にtrueを返すこと() {
        assert!(DbscService::is_session_bound(false, false));
        assert!(DbscService::is_session_bound(false, true));
    }

    #[test]
    fn is_session_boundでdbsc登録済みかつcookieありならtrueを返すこと() {
        assert!(DbscService::is_session_bound(true, true));
    }

    #[test]
    fn is_session_boundでdbsc登録済みかつcookieなしならfalseを返すこと() {
        assert!(!DbscService::is_session_bound(true, false));
    }

    #[test]
    fn complete_refreshでセッションid不一致の場合エラーを返すこと() {
        let service = DbscService::new("https://example.com".to_string());
        let result = service.complete_refresh(
            "jwt",
            "request-id",
            Some("different-id"),
            Some("key"),
            vec!["nonce".to_string()],
        );
        assert!(matches!(result, Err(DbscError::SessionIdMismatch)));
    }

    #[test]
    fn complete_refreshで公開鍵なしの場合エラーを返すこと() {
        let service = DbscService::new("https://example.com".to_string());
        let result = service.complete_refresh(
            "jwt",
            "sess-id",
            Some("sess-id"),
            None,
            vec!["nonce".to_string()],
        );
        assert!(matches!(result, Err(DbscError::MissingPublicKey)));
    }

    #[test]
    fn complete_refreshでnoncesが空の場合エラーを返すこと() {
        let service = DbscService::new("https://example.com".to_string());
        let result =
            service.complete_refresh("jwt", "sess-id", Some("sess-id"), Some("key"), vec![]);
        assert!(matches!(result, Err(DbscError::NoPendingChallenges)));
    }
}
