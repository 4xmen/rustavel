//! Development-only request timing for finding performance bottlenecks.
//!
//! This measures elapsed time until the inner Axum service returns a
//! response. It is a relative diagnostic tool, not a precise benchmark:
//! it includes async waiting and does not measure complete body delivery.
//! what's this MW calculate:
//!
//! | Step | Calc? |
// | Request | — |
// | Middleware before `performance_middleware` | ❌ |
// | `performance_middleware`: start |  Start here |
// | Inner middleware | ✅ |
// | Controller / Handler | ✅ |
// | Database query | ✅ |
// | Application logic | ✅ |
// | Build context for MiniJinja | ✅ |
// | Execute `template.render(...)` | ✅ |
// | Build `Response` | ✅ |
// | `performance_middleware`: elapsed = ... | End here |
// | Response returns to outer layers | — |
// | Send body to client | ❌ |

use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};

use rustavel_core::config::CONFIG;
use rustavel_core::facades::datetime::format_duration;

/// Logs the elapsed time for processing an HTTP request.
pub async fn performance_middleware(mut request: Request, next: Next) -> Response {

    // Capture request details before moving it into the next service.
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let start = Instant::now();

    request.extensions_mut().insert(start);

    // Run the inner middleware stack and handler.
    let response = next.run(request).await;

    // Measure elapsed time after the response has been constructed.
    let elapsed = start.elapsed();
    let status = response.status();

    // println!(
    //     "[PERF] {method} {path} -> {status} | {}",
    //     format_duration(elapsed),
    // );

    if CONFIG.app.debug {
        println!(
            "[PERF] {method} {path} -> {status} | {}",
            format_duration(elapsed),
        );
    }
    response
}
