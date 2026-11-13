use std::time::Duration;

use axum::{
    Router,
    extract::{DefaultBodyLimit, Request},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::Response,
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::{SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer},
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::config::http::HttpConfig;

/// Applies the default HTTP middleware stack based on `HttpConfig`.
///
/// Layer order: in axum, the LAST `.layer()` call becomes the OUTERMOST layer.
/// So the code below goes from innermost (closest to handlers) to outermost.
/// A request therefore passes: CatchPanic -> RequestId -> Trace -> Timeout
/// -> Compression -> CORS -> security headers -> CSRF guard -> body limit -> handler.
pub fn apply<S>(mut router: Router<S>, cfg: &HttpConfig) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // --- innermost ---

    // Limits body size for extractors (Json, Form, Multipart, Bytes).
    // If you ever read the raw body manually, use tower_http's
    // `RequestBodyLimitLayer` together with `DefaultBodyLimit::disable()`.
    router = router.layer(DefaultBodyLimit::max(cfg.max_body_size));

    // CSRF protection for cookie-based web routes (see `csrf_guard` below).
    if cfg.csrf_guard {
        router = router.layer(middleware::from_fn(csrf_guard));
    }

    // Baseline security headers.
    // CSP is intentionally NOT set by default: it depends on the frontend
    // and a generic policy would either be useless or break pages.
    router = router
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    if cfg.hsts {
        router = router.layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }

    // CORS: only enabled when explicit origins are configured.
    // Never use `CorsLayer::permissive()` / `Any` in production.
    if !cfg.cors.allowed_origins.is_empty() {
        let origins: Vec<HeaderValue> = cfg
            .cors
            .allowed_origins
            .iter()
            .map(|o| {
                o.parse()
                    .unwrap_or_else(|_| panic!("invalid CORS origin in config: {o}"))
            })
            .collect();

        router = router.layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
                .allow_credentials(cfg.cors.allow_credentials)
                .max_age(Duration::from_secs(cfg.cors.max_age_secs)),
        );
    }

    if cfg.compression {
        router = router.layer(CompressionLayer::new());
    }

    // --- outermost ---
    router
        // Aborts slow requests (slowloris-style / hanging handlers) with 408.
        // Note: older tower-http versions use `TimeoutLayer::new(duration)`.
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(cfg.timeout_secs),
        ))
        // Copies x-request-id to the response for log correlation.
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        // Generates x-request-id (requires tower-http feature `uuid`).
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // Marks sensitive headers so Trace/Debug output never prints them.
        .layer(SetSensitiveResponseHeadersLayer::new([header::SET_COOKIE]))
        .layer(SetSensitiveRequestHeadersLayer::new([
            header::AUTHORIZATION,
            header::COOKIE,
        ]))
        // Converts panics into a generic 500 instead of dropping the connection.
        .layer(CatchPanicLayer::new())
}

/// Modern CSRF guard (no tokens needed).
///
/// How it works:
/// 1. Safe methods (GET/HEAD/OPTIONS) are always allowed.
/// 2. Modern browsers send `Sec-Fetch-Site` and it cannot be forged by
///    attacker pages. Only `same-origin` and `none` (user-initiated) pass.
/// 3. Fallback: if `Origin` exists, its host must equal the `Host` header.
/// 4. Requests with neither header come from non-browser clients
///    (curl, tests, mobile apps) where CSRF is not applicable.
///
/// Recommended together with session cookies using
/// `SameSite=Lax`, `HttpOnly` and `Secure`.
/// Not needed for JWT in the `Authorization` header. If one router serves both
/// web and API routes, apply this guard only to the web group.
pub async fn csrf_guard(req: Request, next: Next) -> Result<Response, StatusCode> {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return Ok(next.run(req).await);
    }

    let h = req.headers();

    if let Some(site) = h.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        return match site {
            "same-origin" | "none" => Ok(next.run(req).await),
            _ => Err(StatusCode::FORBIDDEN), // cross-site / same-site
        };
    }

    if let Some(origin) = h.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let host = h
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let origin_host = origin.split("://").nth(1).unwrap_or("");
        return if origin_host == host {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::FORBIDDEN)
        };
    }

    Ok(next.run(req).await)
}
