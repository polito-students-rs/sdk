//! MFA push authenticator: status, enrolment, challenge and validation.
//!
//! The device holds a secp256k1 key and enrols its public half (SPKI DER,
//! base64). A login on another channel creates a challenge, approved or
//! declined by signing `nonce|serial[|decline]` (ECDSA with SHA-256, DER,
//! base32).

use std::fmt;

use k256::{
    ecdsa::{Signature, SigningKey, VerifyingKey, signature::Signer}, elliptic_curve::Generate
};
use chrono::{DateTime, Utc};
use data_encoding::{BASE32, BASE64};
use zeroize::Zeroizing;

use crate::auth::Auth;
use crate::PolitoClient;
use crate::error::{ApiError, Result};

use polito_api::models::mfa_status_response::Status;
use polito_api::models::{EnrolMfaRequest, MfaChallenge, MfaStatusResponse, MfaStatusResponseDetails};
use polito_api::models::{MfaFetchChallenge200Response, ValidateMfaRequest};
use polito_api::apis::{self, auth_api::MfaFetchChallengeError};


/// Login request waiting for approval, from [`Mfa::pending_challenge`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    serial: String,
    nonce: String,
    requested_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl Challenge {
    pub(crate) fn from_model(model: MfaChallenge) -> Self {
        Self {
            serial: model.serial,
            nonce: model.challenge,
            requested_at: model.request_ts.with_timezone(&Utc),
            expires_at: model.expiration_ts.with_timezone(&Utc),
        }
    }

    pub fn serial(&self) -> &str { &self.serial }
    pub fn requested_at(&self) -> DateTime<Utc> { self.requested_at }
    pub fn expires_at(&self) -> DateTime<Utc> { self.expires_at }
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool { now >= self.expires_at }
}

/// Authenticator enrolled for the user, carried by [`MfaStatus::Active`] and
/// [`MfaStatus::Locked`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MfaDetails {
    last_auth: DateTime<Utc>,
    auth_count: u32,
    serial: Option<String>,
    description: Option<String>,
}

impl MfaDetails {
    pub(crate) fn from_model(model: MfaStatusResponseDetails) -> Result<Self> {
        let auth_count = u32::try_from(model.auth_count).map_err(|_| {
            ApiError::Malformed(format!("negative MFA auth count: {}", model.auth_count))
        })?;
        Ok(Self {
            last_auth: model.last_auth.with_timezone(&Utc),
            auth_count,
            serial: model.serial,
            description: model.description,
        })
    }

    pub fn last_auth(&self) -> DateTime<Utc> { self.last_auth }
    pub fn auth_count(&self) -> u32 { self.auth_count }
    pub fn serial(&self) -> Option<&str> { self.serial.as_deref() }
    /// Description chosen at enrolment.
    pub fn description(&self) -> Option<&str> { self.description.as_deref() }
}

/// MFA state of the current user.
///
/// The server may add states without breaking callers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MfaStatus {
    Active(MfaDetails),
    Locked(MfaDetails),
    Available,
    Unavailable,
    NeedsReauth,
}

impl MfaStatus {
    pub(crate) fn from_model(model: MfaStatusResponse) -> Result<Self> {
        let details = |status: &str| match model.details {
            Some(details) => MfaDetails::from_model(*details),
            None => Err(ApiError::Malformed(format!("MFA status {status} without details"))),
        };
        Ok(match model.status {
            Status::Active => Self::Active(details("active")?),
            Status::Locked => Self::Locked(details("locked")?),
            Status::Available => Self::Available,
            Status::Unavailable => Self::Unavailable,
            Status::NeedsReauth => Self::NeedsReauth,
        })
    }
}

/// Enrolled authenticator: serial assigned by the server and private key.
///
/// `Debug` never shows the private key.
pub struct MfaKey {
    serial: String,
    signing_key: SigningKey,
}

impl fmt::Debug for MfaKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MfaKey")
            .field("serial", &self.serial)
            .field("signing_key", &"<redacted>")
            .finish()
    }
}

impl MfaKey {
    /// Restores a key saved with [`MfaKey::to_parts`]. Any problem with
    /// `secret_b64` gives [`ApiError::InvalidMfaKey`], without details.
    pub fn from_parts(serial: String, secret_b64: &str) -> Result<Self> {
        let secret = Zeroizing::new(BASE64
            .decode(secret_b64.as_bytes())
            .map_err(|_| ApiError::InvalidMfaKey)?
        );
        if secret.len() != 32 {
            return Err(ApiError::InvalidMfaKey);
        };
        Ok(MfaKey {
            serial,
            signing_key:
                SigningKey::from_slice(&secret)
                .map_err(|_| ApiError::InvalidMfaKey)?
        })
    }
    /// Exports `(serial, private key in base64)`, to restore the key later
    /// with [`MfaKey::from_parts`]. The second element is the private key in
    /// clear: store it as a secret.
    pub fn to_parts(&self) -> (String, String) {
        let signing_key_bytes = Zeroizing::new(self.signing_key.to_bytes());
        (self.serial.clone(), BASE64.encode(&signing_key_bytes))
    }
    pub fn serial(&self) -> &str {
        &self.serial
    }
    fn sign(&self, nonce: &str, decline: bool) -> Result<String> {
        let mut message = format!("{nonce}|{}", self.serial);
        if decline {
            message.push_str("|decline");
        }
        let signature: Signature = self
            .signing_key
            .try_sign(message.as_bytes())
            .map_err(|_| ApiError::FailedMfaSignature)?;
        Ok(BASE32.encode(&signature.to_der().to_bytes()))
    }
}


/// MFA operations, obtained from [`Auth::mfa`].
pub struct Mfa<'a> {
    client: &'a PolitoClient,
}

impl Mfa<'_> {

    /// Registers this device as an authenticator of the user.
    pub async fn enrol(&self, description: &str) -> Result<MfaKey> {
        let signing_key = SigningKey::generate();
        let verifying_key = signing_key
            .verifying_key();
        let pubkey = spki_b64(verifying_key);
        let response = self
            .client
            .auth_api
            .mfa_enrol_mfa(EnrolMfaRequest::new(description.to_owned(), pubkey))
            .await?;
        Ok(MfaKey { serial: response.data.serial, signing_key })
    }
    /// Login request waiting for approval, if any.
    pub async fn pending_challenge(&self) -> Result<Option<Challenge>> {
        pending_from(self.client.auth_api.mfa_fetch_challenge().await)
    }

    /// Approves the login request.
    ///
    /// Fails with [`ApiError::MfaDifferentSerial`] if `challenge` belongs to
    /// another authenticator, with [`ApiError::MfaValidationFailed`] if the
    /// server does not accept the answer.
    pub async fn approve(&self, challenge: &Challenge, key: &MfaKey) -> Result<()> {
        self.validate(challenge, key, false).await
    }

    /// Declines the login request. The refusal is signed too, so nobody else
    /// can decline on the user's behalf. Fails like [`Mfa::approve`].
    pub async fn decline(&self, challenge: &Challenge, key: &MfaKey) -> Result<()> {
        self.validate(challenge, key, true).await
    }

    /// Current MFA state of the user.
    pub async fn status(&self) -> Result<MfaStatus> {
        let response = self
            .client
            .auth_api
            .mfa_get_mfa_status()
            .await?;
        MfaStatus::from_model(*response.data)
    }

    async fn validate(&self, challenge: &Challenge, key: &MfaKey, decline: bool) -> Result<()> {
        let request = validate_request(challenge, key, decline)?;
        let response = self.client.auth_api.mfa_validate_mfa(request).await?;
        if !response.data.success {
            return Err(ApiError::MfaValidationFailed);
        }
        Ok(())
    }
}

fn pending_from(
    response: std::result::Result<MfaFetchChallenge200Response, apis::Error<MfaFetchChallengeError>>,
) -> Result<Option<Challenge>> {
    match response.map_err(ApiError::from) {
        Ok(response) => Ok(response.data.map(|model| Challenge::from_model(*model))),
        Err(ApiError::NotFound) => Ok(None),
        Err(e) => Err(e),
    }
}

fn validate_request(challenge: &Challenge, key: &MfaKey, decline: bool) -> Result<ValidateMfaRequest> {
    if challenge.serial != key.serial {
        return Err(ApiError::MfaDifferentSerial);
    }
    let signature = key.sign(&challenge.nonce, decline)?;
    let mut request = ValidateMfaRequest::new(key.serial.clone(), challenge.nonce.clone(), signature);
    request.decline = Some(decline);
    Ok(request)
}

fn spki_b64(key: &VerifyingKey) -> String {
    let point = key.to_sec1_point(true).to_bytes();
    let spki_prefix: [u8; 23] = [
        0x30, 0x36, 0x30, 0x10, 0x06, 0x07, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01,
        0x06, 0x05, 0x2B, 0x81, 0x04, 0x00, 0x0A, 0x03, 0x22, 0x00,
    ];
    let mut der = Vec::with_capacity(spki_prefix.len() + 33);
    der.extend_from_slice(&spki_prefix);
    der.extend_from_slice(&point);
    BASE64.encode(&der)
}

impl<'a> Auth<'a> {
    pub fn mfa(&self) -> Mfa<'a> {
        Mfa { client: self.client }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn restore(secret_b64: &str) -> Result<MfaKey> {
        MfaKey::from_parts("SERIAL".to_owned(), secret_b64)
    }

    fn new_key() -> MfaKey {
        MfaKey {
            serial: "SERIAL".to_owned(),
            signing_key: SigningKey::generate(),
        }
    }

    fn at(rfc3339: &str) -> DateTime<chrono::FixedOffset> {
        DateTime::parse_from_rfc3339(rfc3339).unwrap()
    }

    fn utc(rfc3339: &str) -> DateTime<Utc> {
        at(rfc3339).with_timezone(&Utc)
    }

    fn challenge() -> Challenge {
        Challenge::from_model(MfaChallenge::new(
            "SERIAL".to_owned(),
            "NONCE".to_owned(),
            at("2026-11-25T10:00:00+01:00"),
            at("2026-11-25T10:02:00+01:00"),
        ))
    }

    fn details_model(auth_count: i32) -> MfaStatusResponseDetails {
        let mut details = MfaStatusResponseDetails::new(at("2026-11-25T10:00:00+01:00"), auth_count);
        details.serial = Some("SERIAL".to_owned());
        details.description = Some("laptop".to_owned());
        details
    }

    fn status(status: Status, details: Option<MfaStatusResponseDetails>) -> Result<MfaStatus> {
        let mut model = MfaStatusResponse::new(status);
        model.details = details.map(Box::new);
        MfaStatus::from_model(model)
    }

    #[test]
    fn challenge_maps_fields_and_converts_to_utc() {
        let challenge = challenge();
        assert_eq!(challenge.serial(), "SERIAL");
        assert_eq!(challenge.nonce, "NONCE");
        assert_eq!(challenge.requested_at(), utc("2026-11-25T09:00:00Z"));
        assert_eq!(challenge.expires_at(), utc("2026-11-25T09:02:00Z"));
    }

    #[test]
    fn challenge_expires_at_its_deadline() {
        let challenge = challenge();
        assert!(!challenge.is_expired(utc("2026-11-25T09:01:59Z")));
        assert!(challenge.is_expired(utc("2026-11-25T09:02:00Z")));
        assert!(challenge.is_expired(utc("2026-11-25T09:05:00Z")));
    }

    fn http_error(status: u16) -> apis::Error<MfaFetchChallengeError> {
        apis::Error::ResponseError(apis::ResponseContent {
            status: reqwest::StatusCode::from_u16(status).unwrap(),
            content: String::new(),
            entity: None,
        })
    }

    #[test]
    fn nothing_pending_as_null_or_404() {
        let null = MfaFetchChallenge200Response::new(None);
        assert_eq!(pending_from(Ok(null)).unwrap(), None);
        assert_eq!(pending_from(Err(http_error(404))).unwrap(), None);
    }

    #[test]
    fn pending_challenge_is_converted() {
        let model = MfaChallenge::new(
            "SERIAL".to_owned(),
            "NONCE".to_owned(),
            at("2026-11-25T10:00:00+01:00"),
            at("2026-11-25T10:02:00+01:00"),
        );
        let response = MfaFetchChallenge200Response::new(Some(model));
        assert_eq!(pending_from(Ok(response)).unwrap(), Some(challenge()));
    }

    #[test]
    fn other_challenge_errors_are_kept() {
        assert!(matches!(pending_from(Err(http_error(401))), Err(ApiError::Unauthorized)));
        assert!(matches!(
            pending_from(Err(http_error(500))),
            Err(ApiError::Server { status: 500, .. })
        ));
    }

    #[test]
    fn active_and_locked_carry_details() {
        let expected = MfaDetails {
            last_auth: utc("2026-11-25T09:00:00Z"),
            auth_count: 3,
            serial: Some("SERIAL".to_owned()),
            description: Some("laptop".to_owned()),
        };
        assert_eq!(
            status(Status::Active, Some(details_model(3))).unwrap(),
            MfaStatus::Active(expected.clone())
        );
        assert_eq!(
            status(Status::Locked, Some(details_model(3))).unwrap(),
            MfaStatus::Locked(expected)
        );
    }

    #[test]
    fn active_or_locked_without_details_is_malformed() {
        for s in [Status::Active, Status::Locked] {
            assert!(matches!(status(s, None), Err(ApiError::Malformed(_))));
        }
    }

    #[test]
    fn negative_auth_count_is_malformed() {
        let result = status(Status::Active, Some(details_model(-1)));
        assert!(matches!(result, Err(ApiError::Malformed(_))));
    }

    #[test]
    fn other_states_ignore_details() {
        for (s, expected) in [
            (Status::Available, MfaStatus::Available),
            (Status::Unavailable, MfaStatus::Unavailable),
            (Status::NeedsReauth, MfaStatus::NeedsReauth),
        ] {
            assert_eq!(status(s, None).unwrap(), expected);
            assert_eq!(status(s, Some(details_model(3))).unwrap(), expected);
        }
    }

    /// Decodes a signature from [`MfaKey::sign`] and verifies it on `message`.
    fn verifies(key: &MfaKey, message: &str, signature_b32: &str) -> bool {
        use k256::ecdsa::signature::Verifier;

        let der = BASE32.decode(signature_b32.as_bytes()).unwrap();
        let signature = Signature::from_der(&der).unwrap();
        key.signing_key
            .verifying_key()
            .verify(message.as_bytes(), &signature)
            .is_ok()
    }

    #[test]
    fn approval_signs_nonce_and_serial() {
        let key = new_key();
        let signature = key.sign("NONCE", false).unwrap();
        assert!(verifies(&key, "NONCE|SERIAL", &signature));
        assert!(!verifies(&key, "NONCE|SERIAL|decline", &signature));
    }

    #[test]
    fn decline_signs_a_different_message() {
        let key = new_key();
        let signature = key.sign("NONCE", true).unwrap();
        assert!(verifies(&key, "NONCE|SERIAL|decline", &signature));
        assert!(!verifies(&key, "NONCE|SERIAL", &signature));
        assert_ne!(signature, key.sign("NONCE", false).unwrap());
    }

    #[test]
    fn validate_request_flag_matches_signed_message() {
        let key = new_key();
        for (decline, message) in [(false, "NONCE|SERIAL"), (true, "NONCE|SERIAL|decline")] {
            let request = validate_request(&challenge(), &key, decline).unwrap();
            assert_eq!(request.serial, "SERIAL");
            assert_eq!(request.nonce, "NONCE");
            assert_eq!(request.decline, Some(decline));
            assert!(verifies(&key, message, &request.signature), "decline = {decline}");
        }
    }

    #[test]
    fn validate_request_rejects_another_authenticator() {
        let other = MfaKey {
            serial: "OTHER".to_owned(),
            signing_key: SigningKey::generate(),
        };
        for decline in [false, true] {
            let result = validate_request(&challenge(), &other, decline);
            assert!(matches!(result, Err(ApiError::MfaDifferentSerial)));
        }
    }

    #[test]
    fn spki_is_a_compressed_secp256k1_key() {
        use k256::pkcs8::DecodePublicKey;

        let key = new_key();
        let public_key = key.signing_key.verifying_key();
        let der = BASE64.decode(spki_b64(public_key).as_bytes()).unwrap();

        assert_eq!(der.len(), 56);
        assert!(matches!(der[23], 0x02 | 0x03));
        // an independent SPKI parser reads it back as the same key
        let parsed = VerifyingKey::from_public_key_der(&der).unwrap();
        assert_eq!(&parsed, public_key);
    }

    #[test]
    fn parts_round_trip() {
        let key = new_key();
        let (serial, secret_b64) = key.to_parts();
        assert_eq!(BASE64.decode(secret_b64.as_bytes()).unwrap().len(), 32);

        let restored = MfaKey::from_parts(serial, &secret_b64).unwrap();
        assert_eq!(restored.serial(), key.serial());
        assert_eq!(restored.signing_key.verifying_key(), key.signing_key.verifying_key());
    }

    #[test]
    fn debug_hides_the_secret() {
        let key = new_key();
        let (_, secret_b64) = key.to_parts();
        let shown = format!("{key:?}");
        assert!(shown.contains("SERIAL"));
        assert!(shown.contains("<redacted>"));
        assert!(!shown.contains(&secret_b64));
    }

    #[test]
    fn rejects_invalid_base64() {
        assert!(matches!(restore("not base64!"), Err(ApiError::InvalidMfaKey)));
    }

    #[test]
    fn rejects_short_secret() {
        // `SigningKey::from_slice` would accept it: only the length check catches it
        let short = BASE64.encode(&[1u8; 24]);
        assert!(matches!(restore(&short), Err(ApiError::InvalidMfaKey)));
    }

    #[test]
    fn rejects_out_of_range_scalar() {
        let zero = BASE64.encode(&[0u8; 32]);
        assert!(matches!(restore(&zero), Err(ApiError::InvalidMfaKey)));
    }

    #[test]
    fn error_does_not_echo_the_secret() {
        let secret = "c2VjcmV0LWtleS1tYXRlcmlhbA==";
        let err = restore(secret).unwrap_err();
        let shown = format!("{err} {err:?}");
        assert!(!shown.contains(secret));
    }
}
