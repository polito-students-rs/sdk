//! MFA: status, enrolment, challenge and validation.
//!
//! TODO: wrap `mfa_get_mfa_status`, `mfa_enrol_mfa`, `mfa_fetch_challenge`,
//! `mfa_validate_mfa` as `impl Auth<'_>` methods. The signing flow (key pair,
//! signature of the challenge) needs the algorithm expected by the server,
//! which the spec does not state.
