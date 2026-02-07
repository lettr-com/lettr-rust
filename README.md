# lettr

Official Rust SDK for the [Lettr](https://lettr.com) Email API.

Send transactional emails with tracking, attachments, and template personalization.

[![Crates.io](https://img.shields.io/crates/v/lettr.svg)](https://crates.io/crates/lettr)
[![Documentation](https://docs.rs/lettr/badge.svg)](https://docs.rs/lettr)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/lettr/lettr-rust/blob/main/LICENSE)

## Installation

Add `lettr` to your `Cargo.toml`:

```toml
[dependencies]
lettr = "0.1"
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
    .with_html("<h1>Welcome!</h1><p>Thanks for signing up.</p>")
    .with_text("Welcome! Thanks for signing up.");

    let response = client.emails.send(email).await?;
    println!("Email sent! Request ID: {}", response.request_id);

    Ok(())
}
```

## Features

### Send Emails

```rust,no_run
use lettr::{Lettr, CreateEmailOptions, Attachment};

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// Simple email
let email = CreateEmailOptions::new("from@example.com", ["to@example.com"], "Hello!")
    .with_html("<h1>Hello World!</h1>");

client.emails.send(email).await?;

// With all options
let email = CreateEmailOptions::new("from@example.com", ["to@example.com"], "Welcome!")
    .with_from_name("Acme Inc")
    .with_html("<h1>Hello {{first_name}}!</h1>")
    .with_text("Hello {{first_name}}!")
    .with_reply_to("support@example.com")
    .with_substitution("first_name", "John")
    .with_metadata_entry("user_id", "12345")
    .with_click_tracking(true)
    .with_open_tracking(true)
    .with_attachment(Attachment::new("invoice.pdf", "application/pdf", "base64data..."));

client.emails.send(email).await?;
# Ok(())
# }
```

### Send with Templates

```rust,no_run
use lettr::{Lettr, CreateEmailOptions};

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

let email = CreateEmailOptions::new("from@example.com", ["to@example.com"], "Welcome!")
    .with_template("welcome-email")
    .with_substitution("first_name", "John")
    .with_substitution("company", "Acme Inc");

client.emails.send(email).await?;
# Ok(())
# }
```

### List & Retrieve Emails

```rust,no_run
use lettr::Lettr;
use lettr::emails::ListEmailsOptions;

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// List recent emails
let options = ListEmailsOptions::new()
    .per_page(10)
    .from_date("2025-01-01");

let emails = client.emails.list(options).await?;
for email in &emails.results {
    println!("{} -> {}: {}", email.friendly_from, email.rcpt_to, email.subject);
}

// Get email details by request ID
let details = client.emails.get("request-id").await?;
for event in &details.results {
    println!("Event: {} at {}", event.event_type, event.timestamp);
}
# Ok(())
# }
```

### Manage Domains

```rust,no_run
use lettr::Lettr;

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// List all domains
let domains = client.domains.list().await?;
for domain in &domains {
    println!("{}: {} (can send: {})", domain.domain, domain.status, domain.can_send);
}

// Register a new domain
let result = client.domains.create("example.com").await?;
println!("DKIM selector: {:?}", result.dkim);

// Get domain details
let detail = client.domains.get("example.com").await?;
println!("DKIM status: {:?}", detail.dkim_status);

// Delete a domain
client.domains.delete("example.com").await?;
# Ok(())
# }
```

### Webhooks

```rust,no_run
use lettr::Lettr;

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

let webhooks = client.webhooks.list().await?;
for webhook in &webhooks {
    println!("{}: {} (enabled: {})", webhook.id, webhook.url, webhook.enabled);
}
# Ok(())
# }
```

### Templates

```rust,no_run
use lettr::Lettr;
use lettr::templates::{ListTemplatesOptions, CreateTemplateOptions};

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// List templates
let templates = client.templates.list(ListTemplatesOptions::new()).await?;

// Create a template
let template = CreateTemplateOptions::new("Welcome Email")
    .with_html("<h1>Hello {{FIRST_NAME}}!</h1>")
    .with_project_id(5);

let result = client.templates.create(template).await?;
println!("Created template: {} (slug: {})", result.name, result.slug);
# Ok(())
# }
```

## Configuration

### Environment Variable

```rust,no_run
// Reads from LETTR_API_KEY environment variable
let client = lettr::Lettr::from_env();
```

### Feature Flags

| Feature      | Default | Description                          |
|-------------|---------|--------------------------------------|
| `native-tls` | Yes     | Use the system's native TLS stack   |
| `rustls-tls` | No      | Use rustls for TLS                  |
| `blocking`   | No      | Enable synchronous (blocking) API   |

#### Blocking API

Enable the `blocking` feature for synchronous usage:

```toml
[dependencies]
lettr = { version = "0.1", features = ["blocking"] }
```

```rust,ignore
use lettr::{Lettr, CreateEmailOptions};

fn main() -> lettr::Result<()> {
    let client = Lettr::new("your-api-key");

    let email = CreateEmailOptions::new("from@example.com", ["to@example.com"], "Hello!")
        .with_text("Hello World!");

    let response = client.emails.send(email)?;
    println!("Request ID: {}", response.request_id);

    Ok(())
}
```

## Error Handling

The SDK uses a unified [`Error`] type that covers HTTP errors, API errors, and validation errors:

```rust,no_run
use lettr::{Lettr, CreateEmailOptions, Error};

# async fn run() {
let client = Lettr::new("your-api-key");

let email = CreateEmailOptions::new("from@example.com", ["to@example.com"], "Hello!")
    .with_html("<h1>Hello!</h1>");

match client.emails.send(email).await {
    Ok(response) => println!("Sent! ID: {}", response.request_id),
    Err(Error::Validation(e)) => {
        eprintln!("Validation failed: {}", e.message);
        for (field, messages) in &e.errors {
            eprintln!("  {}: {:?}", field, messages);
        }
    }
    Err(Error::Api(e)) => eprintln!("API error: {} ({:?})", e.message, e.error_code),
    Err(e) => eprintln!("Error: {e}"),
}
# }
```

## License

MIT
