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
    .with_cc("cc@example.com")
    .with_bcc("bcc@example.com")
    .with_reply_to("support@example.com")
    .with_reply_to_name("Support Team")
    .with_tag("welcome-series")
    .with_substitution("first_name", "John")
    .with_metadata_entry("user_id", "12345")
    .with_header("X-Custom-ID", "abc-123")
    .with_click_tracking(true)
    .with_open_tracking(true)
    .with_transactional(true)
    .with_inline_css(true)
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

// Subject is optional when using a template
let email = CreateEmailOptions::new_with_template(
    "from@example.com",
    ["to@example.com"],
    "welcome-email",
)
.with_substitution("first_name", "John")
.with_substitution("company", "Acme Inc");

client.emails.send(email).await?;
# Ok(())
# }
```

### Schedule Emails

```rust,no_run
use lettr::{Lettr, CreateEmailOptions};
use lettr::emails::ScheduleEmailOptions;

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

let email = CreateEmailOptions::new("from@example.com", ["to@example.com"], "Scheduled!")
    .with_html("<h1>This was scheduled!</h1>");

// Must be at least 5 minutes in the future, within 3 days
let options = ScheduleEmailOptions::new(email, "2025-01-16T10:00:00Z");
let response = client.emails.schedule(options).await?;
println!("Scheduled! ID: {}", response.request_id);

// Get scheduled email details
let scheduled = client.emails.get_scheduled(&response.request_id).await?;
println!("State: {}, Scheduled at: {:?}", scheduled.state, scheduled.scheduled_at);

// Cancel a scheduled email
client.emails.cancel_scheduled(&response.request_id).await?;
# Ok(())
# }
```

### List & Retrieve Emails

```rust,no_run
use lettr::Lettr;
use lettr::emails::{ListEmailsOptions, ListEmailEventsOptions};

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// List recent emails
let options = ListEmailsOptions::new()
    .per_page(10)
    .from_date("2025-01-01");

let response = client.emails.list(options).await?;
for item in &response.events.data {
    println!("{}: {:?} -> {:?}", item.event_id, item.friendly_from, item.rcpt_to);
}

// Get email details by request ID
let detail = client.emails.get("request-id", None, None).await?;
println!("State: {}, Recipients: {}", detail.state, detail.num_recipients);
for event in &detail.events {
    println!("  {} at {}", event.event_type, event.timestamp);
}

// List email events with filters
let events = client.emails.list_events(
    ListEmailEventsOptions::new()
        .events(vec!["delivery".into(), "bounce".into()])
        .per_page(25)
).await?;
for event in &events.events.data {
    println!("{}: {} ({})", event.event_id, event.event_type, event.timestamp);
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

// Get domain details (includes DMARC, SPF, DNS provider info)
let detail = client.domains.get("example.com").await?;
println!("DKIM: {:?}, DMARC: {:?}, SPF: {:?}", detail.dkim_status, detail.dmarc_status, detail.spf_status);

// Verify domain DNS configuration
let verification = client.domains.verify("example.com").await?;
println!("DKIM: {}, CNAME: {}, DMARC: {}, SPF: {}",
    verification.dkim_status, verification.cname_status,
    verification.dmarc_status, verification.spf_status);

// Delete a domain
client.domains.delete("example.com").await?;
# Ok(())
# }
```

### Webhooks

```rust,no_run
use lettr::Lettr;
use lettr::webhooks::{CreateWebhookOptions, UpdateWebhookOptions, WebhookAuthType, WebhookEventsMode};

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// List webhooks
let webhooks = client.webhooks.list().await?;
for webhook in &webhooks {
    println!("{}: {} (enabled: {})", webhook.id, webhook.url, webhook.enabled);
}

// Create a webhook
let options = CreateWebhookOptions::new(
    "Order Notifications",
    "https://example.com/webhook",
    WebhookAuthType::Basic,
    WebhookEventsMode::Selected,
)
.with_events(vec!["message.delivery".into(), "message.bounce".into()])
.with_basic_auth("user", "secret");

let webhook = client.webhooks.create(options).await?;
println!("Created webhook: {}", webhook.id);

// Update a webhook
let options = UpdateWebhookOptions::new()
    .with_name("Updated Webhook")
    .with_active(false);
client.webhooks.update(&webhook.id, options).await?;

// Delete a webhook
client.webhooks.delete(&webhook.id).await?;
# Ok(())
# }
```

### Templates

```rust,no_run
use lettr::Lettr;
use lettr::templates::{ListTemplatesOptions, CreateTemplateOptions, UpdateTemplateOptions};

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

// List templates
let response = client.templates.list(ListTemplatesOptions::new()).await?;
for template in &response.templates {
    println!("{}: {} (slug: {})", template.id, template.name, template.slug);
}

// Create a template
let options = CreateTemplateOptions::new("Welcome Email")
    .with_html("<h1>Hello {{FIRST_NAME}}!</h1>")
    .with_project_id(5);

let result = client.templates.create(options).await?;
println!("Created: {} (slug: {})", result.name, result.slug);

// Get template details
let detail = client.templates.get("welcome-email", None).await?;
println!("Version: {:?}, HTML: {:?}", detail.active_version, detail.html.is_some());

// Update a template
let options = UpdateTemplateOptions::new()
    .with_name("Updated Welcome")
    .with_html("<h1>Hello {{NAME}}!</h1>");
let updated = client.templates.update("welcome-email", options).await?;
println!("Updated version: {}", updated.active_version);

// Get merge tags
let tags = client.templates.get_merge_tags("welcome-email", None, None).await?;
for tag in &tags.merge_tags {
    println!("{}: required={}", tag.key, tag.required);
}

// Get rendered HTML
let html = client.templates.get_html(5, "welcome-email").await?;
println!("HTML length: {}, Subject: {:?}", html.html.len(), html.subject);

// Delete a template
client.templates.delete("welcome-email", None).await?;
# Ok(())
# }
```

### Projects

```rust,no_run
use lettr::Lettr;
use lettr::projects::ListProjectsOptions;

# async fn run() -> lettr::Result<()> {
let client = Lettr::new("your-api-key");

let response = client.projects.list(ListProjectsOptions::new()).await?;
for project in &response.projects {
    println!("{}: {}", project.id, project.name);
}
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

## API Coverage

| Endpoint | Method | Status |
|----------|--------|--------|
| `/health` | `GET` | `client.health()` |
| `/auth/check` | `GET` | `client.auth_check()` |
| `/emails` | `POST` | `client.emails.send()` |
| `/emails` | `GET` | `client.emails.list()` |
| `/emails/{requestId}` | `GET` | `client.emails.get()` |
| `/emails/events` | `GET` | `client.emails.list_events()` |
| `/emails/scheduled` | `POST` | `client.emails.schedule()` |
| `/emails/scheduled/{id}` | `GET` | `client.emails.get_scheduled()` |
| `/emails/scheduled/{id}` | `DELETE` | `client.emails.cancel_scheduled()` |
| `/domains` | `GET` | `client.domains.list()` |
| `/domains` | `POST` | `client.domains.create()` |
| `/domains/{domain}` | `GET` | `client.domains.get()` |
| `/domains/{domain}` | `DELETE` | `client.domains.delete()` |
| `/domains/{domain}/verify` | `POST` | `client.domains.verify()` |
| `/webhooks` | `GET` | `client.webhooks.list()` |
| `/webhooks` | `POST` | `client.webhooks.create()` |
| `/webhooks/{id}` | `GET` | `client.webhooks.get()` |
| `/webhooks/{id}` | `PUT` | `client.webhooks.update()` |
| `/webhooks/{id}` | `DELETE` | `client.webhooks.delete()` |
| `/templates` | `GET` | `client.templates.list()` |
| `/templates` | `POST` | `client.templates.create()` |
| `/templates/{slug}` | `GET` | `client.templates.get()` |
| `/templates/{slug}` | `PUT` | `client.templates.update()` |
| `/templates/{slug}` | `DELETE` | `client.templates.delete()` |
| `/templates/{slug}/merge-tags` | `GET` | `client.templates.get_merge_tags()` |
| `/templates/html` | `GET` | `client.templates.get_html()` |
| `/projects` | `GET` | `client.projects.list()` |

## License

MIT
