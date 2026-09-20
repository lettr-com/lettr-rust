# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.6.0] - 2026-09-15

Requests now time out instead of hanging, and `rust-version` states the toolchain the crate actually needs. No public API changed: code written against 1.5.0 compiles unchanged. Two things to check when upgrading: a call that takes longer than 30 seconds now returns an error, and the declared minimum Rust is 1.85.

### Changed

- **Minimum supported Rust version is now 1.85.** `Cargo.toml` declared 1.70, but the crate had not built on 1.70 for a while: dependencies like `indexmap` 2.14 (pulled in through `reqwest`) use the 2024 edition, which Cargo only supports from 1.85. 1.84 fails and 1.85 passes, with all features, the default features, and `rustls-tls,blocking`. This corrects the manifest; no toolchain that worked before stops working. A new CI job builds on 1.85, so a dependency update that raises the floor fails in CI.

### Fixed

- **`get_scheduled()` failed for every scheduled email that had not been sent yet.** The API now returns `transmission_id: null` until the email is actually handed to the sending provider, and `ScheduledTransmission.transmission_id` was a `String`, so serde rejected the response outright - the call returned `Err` rather than the email. The field is now `Option<String>`.

  The same response also carries states this client did not know: `sending`, `sent` and `cancelled`. Those deserialized into `ScheduledEmailState::Other(_)` rather than failing, so they did not error, but no named variant matched them.

- **Requests now time out after 30 seconds.** The async client had no timeout at all, so a stalled connection or an unresponsive server hung the call forever. Every request - async and `blocking` - now has a 30-second total deadline and fails with `Error::Http`, where `is_timeout()` is true. This matches lettr-go, lettr-python, lettr-php and lettr-java.

  The timeout is not configurable. A call that legitimately runs longer than 30 seconds now fails where it used to wait. `blocking` users see no change, because `reqwest::blocking` already defaulted to 30 seconds.

### Added

- **`client.emails.list_scheduled()`** - lists what is queued, newest delivery time first, with `ListScheduledEmailsOptions` (`status`, `per_page`, `page`) and the usual `pagination`. There was previously no way to ask what was scheduled.
- **`ScheduledEmail::request_id`, `accepted`, `rejected`, `tag` and `failure_reason`**, plus `is_cancellable()`, `is_sent()` and `is_cancelled()`. `ScheduledEmailState` gains `Sending`, `Sent` and `Cancelled`, with `is_cancellable()` and `is_terminal()`.
- **`cancel_scheduled()` returns the cancelled `ScheduledEmail`** instead of `()`, so the state can be confirmed without a second call.

### Changed

- **`ScheduledTransmission` is now `ScheduledEmail`.** The old name remains as a deprecated type alias, so code that names the type keeps compiling. It was never a transmission: the provider does not know the email exists until it is sent.
- **`schedule()` returns `ScheduledEmail`** instead of `SendEmailResponse`, and `schedule_with_quota()` returns the new `ScheduledEmailWithQuota`. The old types reported `accepted`/`rejected` as though the email had been sent, which it has not been; those counts are still there, alongside the state and the delivery time.

  **The id `schedule()` gives you changed meaning.** `request_id` is Lettr's own id, prefixed `sch_`, and it is what `get_scheduled()` and `cancel_scheduled()` take. The provider's id is the separate `transmission_id`, which is `None` until the email is sent - and it is the value that appears on **webhook events**. Code that stored the id from `schedule()` to correlate webhooks needs `transmission_id` from a later read.
- **The scheduling window is 5 minutes to 30 days**, up from 3 days. This client never validated it, so longer schedules simply work.

## [1.5.0] - 2026-09-10

Brings this client level with lettr-php: template modules, the folders endpoint, preparation status, and idempotent sends. Everything is additive - code written against 1.4.1 keeps compiling and sends identical requests.

### Added

- **`client.folders.list()`** - the folders templates are filed into, each with its `purpose` and `templates_count`. This is what `CreateTemplateOptions::with_folder_id` was missing: nothing else returned a folder id, so a caller either omitted it and accepted whichever folder the API picked, or hardcoded an integer read out of an app URL. Read-only, because deleting a folder moves or deletes the templates inside it.
- **`TemplatePurpose`** (`Transactional`, `Campaign`) on `CreateTemplateOptions::with_purpose`, on every template response, and as a `ListTemplatesOptions` filter.
- **`TemplatePreparationStatus`** (`Pending`, `Ready`, `Failed`) on every template response, with `is_settled()`.

  `is_settled()` rather than `is_ready()` on purpose: it answers "is what I sent what will go out", which is not the same question as "can I send this". After an *update* the previous render stays in place, so a pending template is still sendable - it is serving the old content.

  Both fields default when the API omits them - `Transactional` and `Ready` - because on a deployment that predates them every template with HTML was simply usable. Defaulting to `Pending` would make an older API look like a stalled queue. Both enums carry an `Unknown(String)` variant, so a value added server-side deserializes rather than failing.
- **`ListTemplatesOptions::folder_id`** - one `per_page(100)` call reconciles a whole bulk import instead of a detail call per template, each dragging the full HTML payload against the same rate limit. A folder outside the resolved project is a 404, not an empty list, so a typo cannot be misread as "nothing is there yet".
- **`CreateEmailOptions::with_idempotency_key`** - reuse the key when you retry and the API returns the original result instead of delivering a second email. `SendEmailResponse::replayed` says when that happened.

  The key lives on the options rather than as a `send` argument, so both `send` and `send_with_quota` pick it up and neither signature changed. It is `#[serde(skip)]`, so the request body is byte-identical to what you were already sending.

  You choose the key; the SDK never generates one. It only works if both attempts use the same value, and the SDK does not retry - one `send` is one HTTP request - so the retry is yours. A malformed key returns `Error::Validation` **before any request goes out**; `is_valid_idempotency_key` is public for callers deriving keys from their own ids.
- **`Error::is_idempotency_in_progress()`** and **`Error::is_idempotency_conflict()`**, because one is safe to retry and the other is not. The first should be retried with the *same* key after `Error::retry_after()` seconds; the second means that key was used with a different payload and will fail identically forever.
- **`Error::retry_after()`** and `ApiError::retry_after` - the `Retry-After` header in seconds, when the API sent one.
- Two new `ErrorCode` variants: `IdempotencyKeyConflict` and `IdempotencyInProgress`.

### Notes

- Keys are scoped per team **and** API key, so the same string through a different API key is a different key. The provider retains one for 24 hours.
- `RawErrorResponse::into_error` now takes the parsed `Retry-After`. It is `pub(crate)`, so this is not a public API change.

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

[Unreleased]: https://github.com/lettr-com/lettr-rust/compare/v1.6.0...HEAD
[1.6.0]: https://github.com/lettr-com/lettr-rust/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/lettr-com/lettr-rust/compare/1.4.1...v1.5.0
[1.4.1]: https://github.com/lettr-com/lettr-rust/compare/1.4.0...1.4.1
[1.4.0]: https://github.com/lettr-com/lettr-rust/compare/1.3.0...1.4.0
[1.3.0]: https://github.com/lettr-com/lettr-rust/compare/v1.2.0...1.3.0
[1.2.0]: https://github.com/lettr-com/lettr-rust/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/lettr-com/lettr-rust/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/lettr-com/lettr-rust/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/lettr-com/lettr-rust/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/lettr-com/lettr-rust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/lettr-com/lettr-rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lettr-com/lettr-rust/releases/tag/v0.1.0
