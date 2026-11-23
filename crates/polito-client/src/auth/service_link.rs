//! Authenticated links to external services (mail, Moodle, live class,
//! smart card).
//!
//! TODO: wrap the `service_link_*` endpoints as `impl Auth<'_>` methods
//! returning the URL instead of `{ data: { url } }`.
