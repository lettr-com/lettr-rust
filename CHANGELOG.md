# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `GET /emails/events` — `emails.list_events()` with filters (events, recipients, date range, transmissions, bounce_classes)
- `POST /emails/scheduled` — `emails.schedule()` with `ScheduleEmailOptions`
- `GET /emails/scheduled/{id}` — `emails.get_scheduled()`
- `DELETE /emails/scheduled/{id}` — `emails.cancel_scheduled()`
- `POST /domains/{domain}/verify` — `domains.verify()` with DMARC/SPF validation results
- `POST /webhooks` — `webhooks.create()` with `CreateWebhookOptions` (basic auth, OAuth2 support)
- `PUT /webhooks/{id}` — `webhooks.update()` with `UpdateWebhookOptions`
- `DELETE /webhooks/{id}` — `webhooks.delete()`
- `GET /templates/{slug}` — `templates.get()` returning `TemplateDetail`
- `PUT /templates/{slug}` — `templates.update()` with `UpdateTemplateOptions`
- `DELETE /templates/{slug}` — `templates.delete()`
- `GET /templates/{slug}/merge-tags` — `templates.get_merge_tags()`
- `GET /templates/html` — `templates.get_html()`
- `GET /projects` — new `projects` module with `projects.list()`
- New `CreateEmailOptions` fields: `cc`, `bcc`, `reply_to_name`, `amp_html`, `tag`, `headers`
- New `EmailOptions` fields: `inline_css`, `perform_substitutions`
- `CreateEmailOptions::new_with_template()` constructor for template-based sending
- `Lettr::with_base_url()` constructor for testing against mock servers
- New types: `EmailEvent` (flat struct covering all 17 event types), `UserAgentParsed`, `GeoIp`, `DnsProvider`, `DmarcValidationResult`, `SpfValidationResult`, `MergeTagChild`
- Integration test suite using `wiremock` covering all endpoints

### Changed
- **BREAKING**: `CreateEmailOptions::reply_to` changed from `Option<Vec<String>>` to `Option<String>` (matches API spec — single address)
- **BREAKING**: `CreateEmailOptions` `subject` is now optional (may be omitted when using `template_slug`)
- **BREAKING**: `CreateEmailOptions::metadata` value type changed from `HashMap<String, serde_json::Value>` to `HashMap<String, String>` (matches API spec)
- **BREAKING**: `ListEmailsResponse` restructured to match API — now exposes `response.events.data` instead of `response.results`; adds `from`, `to` date range fields
- **BREAKING**: `GetEmailResponse` restructured to match API — now exposes `transmission_id`, `state`, `from`, `subject`, `recipients`, `num_recipients`, `events` (previously `results`, `total_count`)
- **BREAKING**: `EmailsSvc::get()` signature changed to `get(request_id, from, to)` to accept optional date filters
- **BREAKING**: Renamed old `EmailEvent` to `SentEmailListItem` (list endpoint's simpler view); `EmailEvent` now refers to the full event type used by `get()` and `list_events()`
- **BREAKING**: `EmailEventDetail` removed — use `EmailEvent` instead
- `DomainDetail`: added `dmarc_status`, `spf_status`, `is_primary_domain`, `dns_provider`
- `DkimInfo`: added `signing_domain`
- `DkimDnsRecord`: added `headers`
- `MergeTag`: added `merge_tag_type` (`type` in JSON), `children`
- `Template`: `folder_id` changed from `Option<u64>` to `u64` (always present per spec)
- `CreateTemplateResponse`: `folder_id` changed from `Option<u64>` to `u64`
- `ErrorCode`: added `RetrievalError` variant (matches new `retrieval_error` code in the Lettr API)
- **BREAKING**: Webhook engagement event constants (`event_types::CLICK`, `OPEN`, `INITIAL_OPEN`, `AMP_CLICK`, `AMP_OPEN`, `AMP_INITIAL_OPEN`) now emit `engagement.*` instead of `engagament.*` — the API fixed the typo upstream

## [0.1.0] - 2024

### Added
- Initial release
- `POST /emails` — send transactional emails
- `GET /emails` — list sent emails
- `GET /emails/{id}` — get email details
- `GET /domains`, `POST /domains`, `GET /domains/{domain}`, `DELETE /domains/{domain}` — domain management
- `GET /webhooks`, `GET /webhooks/{id}` — webhook read access
- `GET /templates`, `POST /templates` — template listing and creation
- `GET /health`, `GET /auth/check` — health and auth endpoints
- `native-tls`, `rustls-tls`, `blocking` feature flags

[Unreleased]: https://github.com/lettr/lettr-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lettr/lettr-rust/releases/tag/v0.1.0
