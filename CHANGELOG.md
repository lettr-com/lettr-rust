# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.4.1] - 2026-08-15

### Fixed

- Corrected the segment condition documentation on `SegmentConditionGroup` and `SegmentConditionsInput`: conditions **within a group** are joined by `OR`, and **groups** are joined by `AND` — i.e. `(A OR B) AND (C OR D)`. The previous doc comments stated the inverse. No behaviour change — the API has always evaluated segments this way, and no code paths were touched. Worth a read if you built a segment against the old description, since it may target a wider or narrower audience than you intended.

## [1.4.0] - 2026-08-14

Covers the reworked bulk contact import (TPL-2105) and the duplicate-create fix. Everything here is additive — code written against 1.3.0 keeps compiling and sends the exact same payloads.

### Added

- **Per-contact bulk create.** `BulkCreateAudienceContactsOptions::for_contacts` takes one `BulkAudienceContactRow` per contact, each with its own properties, lists and topic subscriptions — the alternative to the flat `::new(emails)` shape, which is unchanged

  ```rust
  let options = BulkCreateAudienceContactsOptions::for_contacts(vec![
      BulkAudienceContactRow::new("cara@example.com")
          .with_properties(props),
      BulkAudienceContactRow::new("dan@example.com")
          .with_topics(vec![AudienceTopicSubscription::opt_out("01h-promos")]),
  ])
  .with_list_ids(vec!["01h-everyone".into()])
  .with_update_existing(true);
  ```
- `BulkAudienceContactRow` (`new`, `with_properties`, `with_list_ids`, `with_topics`) and `AudienceTopicSubscription` (with the `opt_in(id)` / `opt_out(id)` constructors), plus the `AudienceTopicSubscriptionState` enum
- `AudienceTopicSubscriptionState` says what a request should *do* with a topic and is deliberately separate from a topic's `default_subscription`, which describes how the topic behaves for a contact that says nothing. An `opt_out` on a topic whose default is opt-out suppresses the auto-subscription in the same request instead of needing a second call
- **Batch-wide `with_list_ids` and `with_topics`,** plus `with_update_existing`, on `BulkCreateAudienceContactsOptions`. Batch-wide lists and topics are unioned into every row; a row-level property key or opt-out wins over the batch-wide value. `with_update_existing(true)` merges properties (submitted keys overwrite, absent keys are preserved) and allows dropping a subscription. It is skipped when `false`, so a legacy payload stays byte-identical
- **Bulk create now reports what happened per row.** `BulkCreateAudienceContactsResponse` gains `updated`, `error_count`, `errors` (`BulkAudienceContactError` — `index`, `email`, `error_code`, `error`) and `contacts` (`BulkAudienceContactRef` — `id`, `email`, `created`), plus the `has_errors()`, `contact_ids()` and `id_for(email)` methods. `created` and `already_existed` keep their exact meaning, and the new fields are `#[serde(default)]`, so the response also parses a pre-TPL-2105 body

  A bulk create can **partially succeed**: rows that fail validation are skipped and returned in `errors` while the rest of the batch commits, and the call still returns HTTP 201. Check `has_errors()` — an `Ok` result does not mean every row landed

  Note that `already_existed` and `updated` overlap by design. They answer different questions ("was the address already in the audience?" vs "did this request change the contact?"), so they do not sum to the row count: a contact that already existed and got attached to a list is counted in both
- `BulkAudienceContactErrorCode` enum (`missing_email`, `invalid_email`, `invalid_property_value`, `unknown_property_key`, `unknown_list`, `unknown_topic`, `invalid_topic_subscription`), with an `Unknown(String)` variant so a code added server-side deserializes instead of failing — the same shape as the existing `ErrorCode` and `AudienceContactStatus` enums
- **Bulk topic subscribe/unsubscribe** — 2 new methods on `client.audience.contacts`, mirroring the existing `bulk_attach_to_lists` / `bulk_detach_from_lists` pair:
  - `bulk_subscribe_to_topics(BulkContactTopicMembershipOptions)` — `POST /audience/contacts/topics/bulk`, returns `BulkSubscribeContactsToTopicsResponse` (`subscribed`, `already_subscribed`, `total_pairs`)
  - `bulk_unsubscribe_from_topics(BulkContactTopicMembershipOptions)` — `DELETE /audience/contacts/topics/bulk` with a request body, returns `BulkUnsubscribeContactsFromTopicsResponse` (`unsubscribed`, `total_pairs`). Pairs that do not exist are ignored

  Both process every `contact_ids` × `topic_ids` combination (up to 1000 × 50). Feed them `contact_ids()` from a bulk create — no ID lookup needed
- `Error::error_code()` — the API's `error_code` across both the `Api` and `Validation` variants, so callers can discriminate without matching on the variant first
- `Error::is_contact_already_exists()` — the 409 that `audience.contacts.create` returns when the email is already in the team's audience. A client-correctable condition, **not** an outage: do not retry it; update the existing contact, or use `bulk_create` with `with_update_existing(true)`

### Changed

- Creating a contact whose email already exists now comes back as `Error::Api` with `ErrorCode::ResourceAlreadyExists` (HTTP 409). The API previously let this escape as HTTP 500 with the misleading `send_error` code, which names email delivery — not involved unless double opt-in is supplied. **If your retry policy retries 5xx, duplicate creates are no longer retried** — which was pointless anyway. Any error mapping or docs of yours that name `send_error` for this endpoint should be corrected

  No new `Error` variant was introduced, so existing `match` arms over `Error` keep compiling
- `BulkCreateAudienceContactsOptions` no longer always serializes `emails` — it is skipped when the `contacts` shape is used. Options built with `::new(emails)` serialize exactly as before

## [1.3.0] - 2026-05-28

### Added
- Full `/campaigns` namespace as a top-level service under `client.campaigns.*`, covering all 6 campaign endpoints from the OpenAPI spec:
  - `GET /campaigns` — `campaigns.list()` with optional `status` filter and pagination
  - `GET /campaigns/{id}` — `campaigns.get()` returning `CampaignDetail` (campaign + rendered HTML)
  - `GET /campaigns/{id}/events` — `campaigns.list_events()` with cursor-based pagination and filters (event type, email, date range)
  - `POST /campaigns/{id}/send` — `campaigns.send()` to dispatch a draft immediately
  - `POST /campaigns/{id}/schedule` — `campaigns.schedule()` with `ScheduleCampaignOptions`
  - `POST /campaigns/{id}/unschedule` — `campaigns.unschedule()` to cancel a scheduled send
- New public types re-exported under `lettr::types::*` and the service under `lettr::services::CampaignsSvc`: `Campaign`, `CampaignDetail`, `CampaignStats`, `CampaignStatus`, `CampaignEvent`, `CampaignEventType`, `CampaignPagination`, request builders (`ListCampaignsOptions`, `ListCampaignEventsOptions`, `ScheduleCampaignOptions`), and response types (`ListCampaignsResponse`, `ListCampaignEventsResponse`).

### Notes
- The action endpoints (`send`, `schedule`, `unschedule`) return `Option<Campaign>` because the API may omit the `data` field if the campaign can't be re-read after the action (e.g. concurrent deletion).
- `list_events` uses cursor-based pagination — keep requesting with the returned `next_cursor` until it is `None`. When a filter is applied, an empty `events` page with a non-`None` `next_cursor` is normal mid-stream; continue paginating.
- The `send`, `schedule`, and `unschedule` endpoints are not available to sandbox API keys (server-side enforcement; no client-side change).

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

[Unreleased]: https://github.com/lettr/lettr-rust/compare/v1.4.1...HEAD
[1.4.1]: https://github.com/lettr/lettr-rust/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/lettr/lettr-rust/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/lettr/lettr-rust/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/lettr/lettr-rust/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/lettr/lettr-rust/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/lettr/lettr-rust/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/lettr/lettr-rust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/lettr/lettr-rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lettr/lettr-rust/releases/tag/v0.1.0
