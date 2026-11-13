use axum::{
    body::{Body, Bytes, to_bytes},
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::Response,
};
use futures_util::{StreamExt, stream};
use percent_encoding::percent_decode_str;

use crate::{config::CONFIG, http::session::Session};

const FIELD: &str = "_token";
const HEADER: &str = "x-csrf-token";

/// How much of a multipart body we are willing to look at to find the token.
/// The token must be the FIRST field, so a few KB is more than enough.
const MULTIPART_PEEK_LIMIT: usize = 8 * 1024;

/// Token-based CSRF verification (second layer, after `csrf_guard`).
/// Must run INSIDE `session::layer`. Apply it only to cookie-session web routes.
///
/// Token sources (the token is NEVER accepted from the URL, to keep it out of
/// access logs, browser history and proxies):
/// 1. `X-CSRF-TOKEN` header (fetch/XHR, any content type)
/// 2. `_token` field of an `application/x-www-form-urlencoded` body
/// 3. `_token` as the FIRST field of a `multipart/form-data` body
pub async fn verify(req: Request, next: Next) -> Result<Response, StatusCode> {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return Ok(next.run(req).await);
    }

    let session = req
        .extensions()
        .get::<Session>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let (req, given) = extract_token(req).await;

    match given {
        Some(t) if session.verify_csrf(&t) => Ok(next.run(req).await),
        // 419 "Page Expired", same as Laravel.
        _ => Err(StatusCode::from_u16(419).unwrap()),
    }
}

async fn extract_token(req: Request) -> (Request, Option<String>) {
    if let Some(t) = req.headers().get(HEADER).and_then(|v| v.to_str().ok()) {
        let t = t.to_owned();
        return (req, Some(t));
    }

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    if content_type
        .to_ascii_lowercase()
        .starts_with("application/x-www-form-urlencoded")
    {
        return read_urlencoded(req).await;
    }

    if let Some(boundary) = multipart_boundary(&content_type) {
        return peek_multipart(req, boundary).await;
    }

    (req, None)
}

/// Urlencoded forms are small and `Form<T>` buffers them anyway,
/// so we buffer (bounded by the global body limit) and rebuild the request.
async fn read_urlencoded(req: Request) -> (Request, Option<String>) {
    let (parts, body) = req.into_parts();
    match to_bytes(body, CONFIG.http.max_body_size).await {
        Ok(bytes) => {
            let found = std::str::from_utf8(&bytes)
                .ok()
                .and_then(|s| find_param(s, FIELD));
            (Request::from_parts(parts, Body::from(bytes)), found)
        }
        Err(_) => (Request::from_parts(parts, Body::empty()), None),
    }
}

// ───────────────────────── multipart (streaming) ─────────────────────────

enum Peek {
    /// Not enough bytes yet to decide.
    NeedMore,
    /// First field is not `_token` (or the body is malformed): reject.
    Missing,
    Found(String),
}

/// Reads body chunks only until the first multipart field is complete,
/// then returns a request whose body is: (bytes already read) + (rest of the stream).
/// Nothing beyond the first few KB is buffered, so large uploads still stream.
async fn peek_multipart(req: Request, boundary: String) -> (Request, Option<String>) {
    let (parts, body) = req.into_parts();
    let mut rest = body.into_data_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut token = None;

    while buf.len() < MULTIPART_PEEK_LIMIT {
        match rest.next().await {
            Some(Ok(chunk)) => {
                buf.extend_from_slice(&chunk);
                match first_field_token(&buf, &boundary) {
                    Peek::NeedMore => continue,
                    Peek::Missing => break,
                    Peek::Found(t) => {
                        token = Some(t);
                        break;
                    }
                }
            }
            // Stream error or early EOF: fail closed.
            _ => break,
        }
    }

    // Give the consumed bytes back, followed by the untouched remainder.
    let replay = stream::once(async move { Ok::<Bytes, axum::Error>(Bytes::from(buf)) });
    let body = Body::from_stream(replay.chain(rest));
    (Request::from_parts(parts, body), token)
}

/// Parses just enough of:
///   --BOUNDARY\r\n
///   Content-Disposition: form-data; name="_token"\r\n\r\n
///   VALUE\r\n--BOUNDARY...
fn first_field_token(buf: &[u8], boundary: &str) -> Peek {
    let open = format!("--{boundary}\r\n");
    let close = format!("\r\n--{boundary}");

    if buf.len() < open.len() {
        return if open.as_bytes().starts_with(buf) {
            Peek::NeedMore
        } else {
            Peek::Missing
        };
    }
    if !buf.starts_with(open.as_bytes()) {
        return Peek::Missing;
    }

    let rest = &buf[open.len()..];
    let Some(headers_end) = find(rest, b"\r\n\r\n") else {
        return Peek::NeedMore;
    };

    // `; name="_token"` with the leading `; ` so `filename="_token"` cannot match.
    let headers = String::from_utf8_lossy(&rest[..headers_end]).to_ascii_lowercase();
    if !headers.contains(&format!("; name=\"{FIELD}\"")) {
        return Peek::Missing;
    }

    let value = &rest[headers_end + 4..];
    match find(value, close.as_bytes()) {
        Some(end) => Peek::Found(String::from_utf8_lossy(&value[..end]).into_owned()),
        None => Peek::NeedMore,
    }
}

fn multipart_boundary(content_type: &str) -> Option<String> {
    if !content_type
        .to_ascii_lowercase()
        .starts_with("multipart/form-data")
    {
        return None;
    }
    content_type
        .split(';')
        .skip(1)
        .find_map(|p| {
            let (k, v) = p.trim().split_once('=')?;
            k.eq_ignore_ascii_case("boundary")
                .then(|| v.trim().trim_matches('"').to_string())
        })
        .filter(|b| !b.is_empty())
}

// ───────────────────────── helpers ─────────────────────────

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Minimal `a=b&c=d` parser (form-urlencoded: `+` means space, then percent-decode).
fn find_param(encoded: &str, key: &str) -> Option<String> {
    encoded.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| {
            percent_decode_str(&v.replace('+', " "))
                .decode_utf8_lossy()
                .into_owned()
        })
    })
}
