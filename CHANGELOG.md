# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2026-05-25

### Added
- Full `/audience` namespace as nested sub-services under `client.audience.*`, covering all 28 audience endpoints from the OpenAPI spec:
  - `audience.lists` — `list`, `create`, `get`, `update`, `delete`, `bulk_delete`
  - `audience.contacts` — `list`, `create`, `bulk_create`, `get`, `update`, `delete`, `attach_to_list`, `detach_from_list`, `bulk_attach_to_lists`, `bulk_detach_from_lists`, `subscribe_to_topic`, `unsubscribe_from_topic`
  - `audience.topics` — `list`, `create`, `get`, `update`, `delete`
  - `audience.properties` — `list`, `create`, `get`, `update`, `delete`
  - `audience.segments` — `list`, `create`, `get`, `update`, `delete`
- New public types re-exported under `lettr::types::*` and services under `lettr::services::*` (`AudienceList`, `AudienceContact`, `AudienceTopic`, `AudienceProperty`, `AudienceSegment`, `SegmentCondition`, `SegmentOperator`, `DoubleOptInConfig`, builders, response wrappers, and enums for status/visibility/property type).
- `clear_description()` on `UpdateAudienceTopicOptions`, `clear_fallback_value()` on `UpdateAudiencePropertyOptions`, and `clear_list_id()` on `UpdateAudienceSegmentOptions` for sending JSON `null` to clear nullable fields.
- `Config::encode_path_segment()` — RFC 3986 percent-encoding helper applied to every audience path-interpolation site so IDs containing `/`, `?`, `#`, or other reserved characters are safely encoded.

### Fixed
- `AudienceContact.properties` now tolerates the PHP `[]` shape that the API returns for contacts with no custom properties (PHP's `json_encode` serializes empty associative arrays as `[]` instead of `{}`). A non-empty array still errors.
- `AudienceTopic.created_at` is now `Option<String>` to match the spec, which marks it nullable.

### Notes
- `BulkContactListMembershipOptions` uses a named-builder pattern (`new().with_contact_ids(...).with_list_ids(...)`) instead of positional arguments, preventing accidental swap of the two `Vec<String>` lists.
- The `/audience/confirm/{token}` endpoint is intentionally excluded — it's a public confirmation flow not meant for SDK callers.

## [1.1.0] - 2026-04-22

### Added
- `UpdateWebhookOptions::with_url()` — sets the webhook destination on `PUT /webhooks/{id}`, matching the field name used by `POST /webhooks`.

### Deprecated
- `UpdateWebhookOptions::with_target()` — use `with_url()` instead. The `target` field is still serialized when set, so pre-1.1 callers keep working until the server drops support.

### Notes
- Webhook event types are sent and received with their namespace prefix (`message.*`, `engagement.*`, `generation.*`, `unsubscribe.*`, `relay.*`). The `event_types::*` constants already emit the namespaced form — no caller-side change required.

## [1.0.1] - 2026-04-20

### Fixed
- README installation snippets updated to reference `lettr = "1.0"` (were still showing `"0.1"`)

## [1.0.0] - 2026-04-20

Promotes the current API surface to a stable `1.0.0` release. No code changes since `0.3.0` — from this point on, breaking changes require a major version bump (see `RELEASING.md`).

## [0.3.0] - 2026-04-18

### Changed
- `ErrorCode`: added `RetrievalError` variant (matches new `retrieval_error` code in the Lettr API)
- **BREAKING**: Webhook engagement event constants (`event_types::CLICK`, `OPEN`, `INITIAL_OPEN`, `AMP_CLICK`, `AMP_OPEN`, `AMP_INITIAL_OPEN`) now emit `engagement.*` instead of `engagament.*` — the API fixed the typo upstream

## [0.2.0] - 2026-04-17

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

[Unreleased]: https://github.com/lettr/lettr-rust/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/lettr/lettr-rust/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/lettr/lettr-rust/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/lettr/lettr-rust/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/lettr/lettr-rust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/lettr/lettr-rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lettr/lettr-rust/releases/tag/v0.1.0
