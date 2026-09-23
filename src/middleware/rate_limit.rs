//! src/middleware/rate_limit.rs — Rate limiting configuration for auth endpoints.
//!

use tower_governor::governor::GovernorConfigBuilder;

/// Build the rate limiter config for auth endpoints.
/// - 5 requests burst capacity
/// - Replenished at 1 request per second
pub fn auth_rate_limiter() -> tower_governor::governor::GovernorConfig<
    tower_governor::key_extractor::PeerIpKeyExtractor,
    governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>,
> {
    GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .expect("Failed to build rate limiter config")
}

// End of File
