# lettr

Official Rust SDK for the [Lettr](https://lettr.com) Email API. An async, typed client for emails, templates, domains, webhooks, audience, and campaigns.

[![Crates.io](https://img.shields.io/crates/v/lettr.svg)](https://crates.io/crates/lettr)
[![Documentation](https://docs.rs/lettr/badge.svg)](https://docs.rs/lettr)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/lettr/lettr-rust/blob/main/LICENSE)

## Installation

Add `lettr` to your `Cargo.toml`:

```toml
[dependencies]
lettr = "1.3"
```

Or with the Cargo CLI:

```sh
cargo add lettr
```

## Quick Start

```rust,no_run
use lettr::{Lettr, CreateEmailOptions};

#[tokio::main]
async fn main() -> lettr::Result<()> {
    let client = Lettr::new("your-api-key");

    let email = CreateEmailOptions::new(
        "sender@example.com",
        ["recipient@example.com"],
        "Hello from Lettr!",
    )
    .with_html("<h1>Welcome!</h1>");

    let response = client.emails.send(email).await?;
    println!("Email sent! Request ID: {}", response.request_id);

    Ok(())
}
```

`Lettr::from_env()` reads the key from `LETTR_API_KEY` instead.

## Error Handling

Methods return `lettr::Result<T>` with a unified `Error` you can match on:

```rust,no_run
# use lettr::{Lettr, CreateEmailOptions, Error};
# async fn run() {
# let client = Lettr::new("key");
# let email = CreateEmailOptions::new("f@e.com", ["t@e.com"], "Hi");
match client.emails.send(email).await {
    Ok(response) => println!("Sent! ID: {}", response.request_id),
    Err(Error::Validation(e)) => eprintln!("Validation: {} {:?}", e.message, e.errors),
    Err(Error::Api(e)) => eprintln!("API error: {} ({:?})", e.message, e.error_code),
    Err(e) => eprintln!("Error: {e}"),
}
# }
```

See [Error Handling](https://docs.lettr.com/quickstart/rust/advanced#error-handling) for the full set of variants.

## Feature Flags

| Feature      | Default | Description                        |
|-|-|-|
| `native-tls` | Yes     | Use the system's native TLS stack  |
| `rustls-tls` | No      | Use rustls for TLS                 |
| `blocking`   | No      | Enable the synchronous (blocking) API |

With `blocking`, drop the `.await` — methods return `Result` directly.

## Documentation

Full guides for every service, with complete request/response details, live in the docs:

📚 **[docs.lettr.com/quickstart/rust](https://docs.lettr.com/quickstart/rust/quickstart)**

| Topic | Guide |
|-|-|
| Install, client, sending | [Quickstart](https://docs.lettr.com/quickstart/rust/quickstart) |
| Async patterns, batch sending, error handling | [Advanced](https://docs.lettr.com/quickstart/rust/advanced) |
| Manage Lettr templates & merge tags | [Templates](https://docs.lettr.com/quickstart/rust/templates) |
| Add, verify, and manage sending domains | [Domains](https://docs.lettr.com/quickstart/rust/domains) |
| Webhook endpoints for delivery & engagement events | [Webhooks](https://docs.lettr.com/quickstart/rust/webhooks) |
| Lists, contacts, topics, properties, segments | [Audience](https://docs.lettr.com/quickstart/rust/audience) |
| List, send, and schedule campaigns | [Campaigns](https://docs.lettr.com/quickstart/rust/campaigns) |
| Endpoint reference (params & schemas) | [API Reference](https://docs.lettr.com/api-reference/introduction) |

## License

MIT
