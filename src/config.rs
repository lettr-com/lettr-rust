use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Method;

const BASE_URL: &str = "https://app.lettr.com/api";

// Use the correct reqwest types based on blocking feature.
#[cfg(feature = "blocking")]
use reqwest::blocking::Client as HttpClient;
#[cfg(not(feature = "blocking"))]
use reqwest::Client as HttpClient;

#[cfg(not(feature = "blocking"))]
pub(crate) type RequestBuilder = reqwest::RequestBuilder;
#[cfg(feature = "blocking")]
pub(crate) type RequestBuilder = reqwest::blocking::RequestBuilder;

#[cfg(not(feature = "blocking"))]
pub(crate) type Response = reqwest::Response;
#[cfg(feature = "blocking")]
pub(crate) type Response = reqwest::blocking::Response;

/// Internal configuration for the Lettr HTTP client.
#[derive(Debug, Clone)]
pub(crate) struct Config {
    http: HttpClient,
    base_url: String,
}

impl Config {
    /// Creates a new [`Config`] with the given API key.
    pub fn new(api_key: &str) -> Self {
        Self::with_base_url(api_key, BASE_URL)
    }

    /// Creates a new [`Config`] with a custom base URL.
    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .expect("API key must be valid ASCII"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(concat!("lettr-rust/", env!("CARGO_PKG_VERSION"))),
        );

        let http = HttpClient::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: base_url.to_owned(),
        }
    }

    /// Build an HTTP request for the given method and path.
    pub fn build(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{path}", self.base_url);
        self.http.request(method, url)
    }

    /// Percent-encode a value for safe use as a single URL path segment.
    ///
    /// Encodes every byte that is not an RFC 3986 unreserved character
    /// (ALPHA / DIGIT / "-" / "." / "_" / "~"). This is conservative: it
    /// also encodes sub-delims like ":" and "@" that are legal in path
    /// segments, but doing so is harmless and avoids surprises if a caller
    /// passes a value containing structural characters like "/", "?", or "#".
    pub(crate) fn encode_path_segment(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        for byte in value.as_bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    out.push(*byte as char);
                }
                other => {
                    out.push('%');
                    out.push_str(&format!("{other:02X}"));
                }
            }
        }
        out
    }

    /// Send a built request and handle non-success status codes.
    ///
    /// Returns the raw response on success, or an appropriate error.
    #[maybe_async::maybe_async]
    pub async fn send(&self, request: RequestBuilder) -> crate::Result<Response> {
        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let body = response.text().await.unwrap_or_default();

            match serde_json::from_str::<crate::error::RawErrorResponse>(&body) {
                Ok(raw) => Err(raw.into_error()),
                Err(_) => Err(crate::Error::Parse(format!("HTTP {status}: {body}"))),
            }
        }
    }
}
