# Sv2 Protocol Coverage

This document formally describes how `interoperability-tests-sv2` covers the [Stratum V2 Protocol Specification](https://github.com/stratum-mining/sv2-spec).

§ indicates spec sections.

## How scenarios are organized

A **Scenario** is a single testable Sv2 Protocol behavior — each describes a failure mode, its spec basis, and how a client detects it.

The catalog is organized **one table per spec message**, mirroring the spec's own structure (§3.2–§3.6 common, §5.3 mining, §6 job declaration, §7 template distribution), plus a **General** section for cross-cutting invariants that have no home message (framing, error codes, channel/job namespaces, liveness) and a **Protocol Extensions** section covering §3.4/§3.4.3 machinery and the registered extensions of §9.

**Anchor rule:** each scenario is filed under the message whose spec section carries the normative requirement it tests. Scenarios frequently involve a longer session with other messages; the `Involves` column lists them so the scenario remains discoverable from every message it touches.

**Scenario IDs** are `<PROTO>-<MSG>-<n>` where `PROTO` ∈ `GEN` (cross-cutting), `C` (common), `M` (mining), `JD` (job declaration), `TD` (template distribution). Extension scenarios use `X<extid>-<MSG>-<n>` (e.g. `X0001-REQS-1` for extension `0x0001`). IDs are unique, stable references: new scenarios append; existing IDs are never renumbered or reused. IDs prefixed `JD-`/`TD-` are reserved for future suite expansion (see [Protocol expansion](#protocol-expansion)).

Scenarios that apply symmetrically to a standard/extended message pair are separate scenarios with separate IDs: they exercise different server code paths and are covered independently per suite.

## Suites

A **Suite** is an application context (e.g. Solo Pool, Pool, JDS, TP, Proxy) that maps scenarios to real-world deployments. The [**Suite Coverage**](#suite-coverage) table tracks which scenarios are implemented per suite.

Note: at the protocol level, a solo pool is indistinguishable from any other non-JD pool — the Mining Protocol has no notion of "solo". The Solo Pool label denotes the initial deployment target (trivial local setup; low-difficulty endpoints make Θ2 share grinding practical; no JDP/TDP involvement), not a different protocol profile. The same scenarios apply verbatim to any non-JD pool; a future Pool column would only differ in *expected-behavior* scenarios (e.g. M-SSS-5/M-SSE-6, where duplicate-share dedup is accounting-critical for pools but harmless for solo deployments).

## Compliance Tiers

**Compliance Tiers** classify scenarios by the normative strength of their spec basis, making tier assignment mechanical:

- **Ω1 — spec violation.** The scenario tests a verbatim MUST/MUST NOT — including field-level "must" statements and field-definition identity semantics (e.g. `request_id` echo).
- **Ω2 — derived requirement.** The requirement follows from spec semantics, but no verbatim MUST exists. The Spec basis column makes the derived status self-documenting.
- **Ω3 — expected behavior / robustness.** No spec basis at all (e.g. duplicate-share dedup, liveness).

Consequence severity does not drive tiers; the Failure mode column carries that context. Scenarios whose tier is blocked by a spec ambiguity are listed in [Spec Gaps and Ambiguities](#spec-gaps-and-ambiguities).

This list might grow as the project matures.

## Testability Tiers

**Testability Tiers** classify scenarios by the type of assertion required.

- **Θ1 — Protocol conformance.** Pure message-level assertions (no valid-share grinding): framing, negotiation, channel open, message ordering, invalid-share rejection, channel lifecycle.
- **Θ2 — Valid-share accounting.** The test client grinds real shares meeting the *channel* target and asserts acceptance / rejection and batch-accounting semantics.

This list might grow as the project matures.

## Scenario Catalog

### General ([§3.1–3.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#31-data-types-mapping), [§5.1–5.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#51-job))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| GEN-1 | Ω1 | Θ1 | `channel_msg` bit wrong per message type (e.g. `NewMiningJob` 0x15 with bit unset, `OpenStandardMiningChannel.Success` 0x11 with bit set) | §3.2.1, §8 | Assert bit on every received frame | all messages |
| GEN-2 | Ω1 | Θ1 | Core message sent with non-zero `extension_type` | §3.4.1, §3.4 | Mask the `channel_msg` bit first, then assert `extension_type & !0x8000 == 0` | all messages |
| GEN-3 | Ω1 | Θ1 | `error_code` containing control characters | §3.5 (MUST NOT) | Validate `error_code` on every error message | all `*.Error` messages |
| GEN-4 | Ω2 | Θ1 | `error_code` containing non-printable, non-control bytes | §3.5 (SHOULD — recommendation, not MUST) | Same validation as GEN-3, lower tier | all `*.Error` messages |
| GEN-5 | Ω2 | Θ1 | Server sends messages for a `channel_id` never opened (or already closed) on this connection | §5.2 | Track open channel set; assert all inbound channel messages reference it | all channel messages; see M-CC-1 |
| GEN-6 | Ω3 | Θ1 | Job starvation: no fresh jobs as the search space exhausts | §5.1 ("provide new and unique Jobs quickly enough") | Soft liveness/timing metric | `NewMiningJob`, `NewExtendedMiningJob` |
| GEN-7 | Ω2 | Θ1 | Message with unknown `extension_type` that terminates locally is not discarded and ignored (server errors, hangs up, or crashes) | §3.4.1 ("Messages with an unknown `extension_type` which are to be processed locally MUST be discarded and ignored") | Send a frame with an unallocated `extension_type`; assert the session continues unaffected | all messages |
| GEN-8 | Ω2 | Θ2 | Server chokes on unnegotiated/unknown TLV fields appended to a core message | §3.4.3 ("unknown fields can be safely ignored"); ext `0x0002` §1.3 ("MUST ignore any unexpected TLV fields") | Append an unknown TLV to `SubmitSharesExtended`; assert the share is processed normally | `SubmitSharesExtended` |
| GEN-9 | Ω2 | Θ1 | Server-generated TLV fields violate placement/ordering rules (not at end of payload, not ordered by `extension_type`) | §3.4.3 (MUSTs) | Parse TLV region on server messages; only applicable once the server negotiates a TLV extension | all TLV-extended messages |
| GEN-10 | Ω2 | Θ1 | TLV field exceeding its extension's maximum length is accepted | §3.4.3 ("the recipient MUST reject the message") | Send over-length TLV (e.g. 33-byte `user_identity`); assert the message is rejected | `SubmitSharesExtended`, `SubmitShares.Error` |
| GEN-11 | Ω2 | Θ1 | Proxy drops or modifies an unknown-extension channel message instead of forwarding it unmodified to the channel endpoint | §3.4.1 ("MUST forward that message... MUST NOT be otherwise modified") | Proxy suites only: inject unknown-`extension_type` channel message; assert byte-identical forwarding | all channel messages |
| GEN-12 | Ω1 | Θ1 | Server sends extension-specific messages for an extension that was never negotiated | §3.4.2 ("Extensions MUST require negotiation... before sending") | Assert no extension-typed frames arrive without a prior `RequestExtensions.Success` | `RequestExtensions.Success` |
| GEN-13 | Ω2 | Θ1 | Server sends contradictory responses to a single request (e.g. both `.Success` and `.Error` for the same `request_id`) | §5.3.3/§5.3.6, §5.3.19/§5.3.20, ext `0x0001` §2 (derived: one request, one response) | Assert exactly one response per `request_id` | `OpenMiningChannel.Error`, `RequestExtensions.Error`, `SetCustomMiningJob.Error` |

### Common Protocol Messages

#### `SetupConnection` ([§3.6.1](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#361-setupconnection-client---server), mining flags [§5.3.1](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#531-setupconnection-flags-for-mining-protocol))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| C-SC-1 | Ω2 | Θ1 | Server sends *any* protocol message before responding to `SetupConnection` | §3.6.1 (derived — the verbatim MUST covers the *client's* first message; server-side silence is inferred) | Message-order assertion | `SetupConnection.Success`, `SetupConnection.Error` |
| C-SC-2 | Ω2 | Θ1 | Server crashes, hangs up, or responds with an error to in-range boundary device-info / endpoint strings. Per-field rules: empty `device_id` MUST be accepted (clients SHOULD send 0-length without telemetry); empty `vendor` may be rejected (clients MUST set it); 255-byte `STR0_255` bounds on all fields | §3.6.1, §3.1 | Send 0-length and 255-byte values per field; assert the connection proceeds normally | — |
| C-SC-3 | Ω1 | Θ1 | Server sets `REQUIRES_FIXED_VERSION` in `.Success.flags` even though client set `REQUIRES_VERSION_ROLLING` | §5.3.1 ("MUST NOT be set") | Flag cross-check | `SetupConnection.Success` |
| C-SC-4 | Ω2 | Θ1 | Server rejects `REQUIRES_STANDARD_JOBS` (bit 0) — fails the most basic client class | §5.2.2 ("MUST also support Standard Channels"; held at Ω2 due to §5.3.1 tension — see Spec Gaps) | Minimal-flags `SetupConnection` must succeed | `SetupConnection.Error` |
| C-SC-5 | Ω2 | Θ1 | Server sets `REQUIRES_EXTENDED_CHANNELS` but then accepts `OpenStandardMiningChannel` (or rejects it despite not setting the flag) | §5.3.1 bit 1 | Open standard channel; assert coherent behavior | `OpenStandardMiningChannel`, `OpenMiningChannel.Error` |
| C-SC-6 | Ω2 | Θ1 | Server sets `REQUIRES_FIXED_VERSION` but later sends `NewExtendedMiningJob` with `version_rolling_allowed = True` | §5.3.1 ("extended jobs MUST NOT indicate support for version rolling") | Cross-message consistency | `NewExtendedMiningJob` |
| C-SC-7 | Ω2 | Θ1 | Server accepts, crashes, or hangs up on a `SetupConnection` with an unknown `protocol` value (not 0/1/2) | §3.6.1 (derived — enumerated values only; no explicit rejection rule; see Spec Gaps) | Send `protocol = 3`; assert framed `SetupConnection.Error` | `SetupConnection.Error` |

#### `SetupConnection.Success` ([§3.6.2](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#362-setupconnectionsuccess-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| C-SCS-1 | Ω1 | Θ1 | Server silently ignores `SetupConnection` or responds with a different message type, leaving the connection open (for connection-close-without-response, see C-SCE-2) | §3.6.1 ("Server MUST respond with either .Success or .Error") | Timeout / wrong-type assertion | `SetupConnection.Error` |
| C-SCS-2 | Ω2 | Θ1 | `used_version` outside client's `[min_version, max_version]` | §3.6.2 (derived — no verbatim MUST; the only logical reading of negotiation) | Send `min_version = max_version = 2`; assert `used_version == 2` | — |

#### `SetupConnection.Error` ([§3.6.3](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#363-setupconnectionerror-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| C-SCE-1 | Ω1 | Θ1 | All-flags feature probe: server errors but returns wrong/empty `flags` (note: `flags = 0` legitimately means the error is not flag-related) | §3.6.3 ("MUST provide the full set of flags which it does not support") | All-flags probe; assert error flags exactly equal the unsupported set | — |
| C-SCE-2 | Ω2 | Θ1 | Server closes the connection without any framed response on unsupported version (e.g. `min_version = 3`) | §3.6.3 (error sent "prior to closing the connection") | Assert framed error, not bare TCP close | — |

#### `ChannelEndpointChanged` ([§3.6.4](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#364-channelendpointchanged-server---client))

No scenarios yet — relevant to proxy contexts, not solo-pool deployments.

#### `Reconnect` ([§3.6.5](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#365-reconnect-server---client))

No scenarios yet.

### Mining Protocol Messages

#### `OpenStandardMiningChannel` ([§5.3.2](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#532-openstandardminingchannel-client---server))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-OSMC-1 | Ω1 | Θ1 | Returned `target` is *above* client's `max_target` and no `OpenMiningChannel.Error` sent | §5.3.2 ("Server MUST accept the target or respond by sending OpenMiningChannel.Error") | Assert `target <= max_target` | `OpenStandardMiningChannel.Success`, `OpenMiningChannel.Error` |

#### `OpenStandardMiningChannel.Success` ([§5.3.3](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#533-openstandardminingchannelsuccess-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-OSMCS-1 | Ω1 | Θ1 | `request_id` doesn't echo the request | §5.3.3 | Random `request_id`s; assert pairing | — |
| M-OSMCS-2 | Ω1 | Θ1 | `channel_id` reused for a second channel, or collides with a `group_channel_id` | §5.2, §5.2.3 (shared namespace) | Open N channels; assert uniqueness | `SetGroupChannel` |
| M-OSMCS-3 | Ω1 | Θ1 | Same `extranonce_prefix` assigned to two standard channels → overlapping search space, duplicate work | §5.1 ("MUST assign a unique subset of the search space to each mining device") | Open 2+ channels; compare prefixes | — |

#### `OpenExtendedMiningChannel` ([§5.3.4](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#534-openextendedminingchannel-client---server))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-OEMC-1 | Ω2 | Θ1 | Server doesn't support Extended channels at all | §5.2.2 ("upstream servers which accept connections and provide work MUST support Extended Channels"; see Spec Gaps) | `OpenExtendedMiningChannel` must not error | `OpenMiningChannel.Error` |
| M-OEMC-2 | Ω1 | Θ1 | Returned `target` is *above* client's `max_target` and no `OpenMiningChannel.Error` sent (extended open) | §5.3.2 via §5.3.4 ("All fields from OpenStandardMiningChannel") | Assert `target <= max_target` | `OpenExtendedMiningChannel.Success`, `OpenMiningChannel.Error` |

#### `OpenExtendedMiningChannel.Success` ([§5.3.5](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#535-openextendedminingchannelsuccess-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-OEMCS-1 | Ω1 | Θ1 | `request_id` doesn't echo the request | §5.3.5 | Random `request_id`s; assert pairing | — |
| M-OEMCS-2 | Ω1 | Θ1 | `channel_id` collision (shared namespace with standard/group channels) | §5.2, §5.2.3 | Open N channels; assert uniqueness | `SetGroupChannel` |
| M-OEMCS-3 | Ω2 | Θ1 | `extranonce_size` smaller than requested `min_extranonce_size`, without error | §5.3.4 / §5.3.5 | Assert `extranonce_size >= min_extranonce_size` (or explicit error) | `OpenMiningChannel.Error` |
| M-OEMCS-4 | Ω1 | Θ1 | `extranonce_prefix` length + `extranonce_size` inconsistent with group constraints | §5.2.3 ("MUST have the exact same size for the full Extended Extranonce"), §5.1.2.1 | Cross-check full extranonce sizes within group | `SetGroupChannel`, `NewExtendedMiningJob` |
| M-OEMCS-5 | Ω1 | Θ1 | Same extranonce space assigned to two channels → overlapping search space | §5.1 | Open 2+ channels; compare prefixes | — |

#### `OpenMiningChannel.Error` ([§5.3.6](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#536-openminingchannelerror-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-OMCE-1 | Ω1 | Θ1 | `request_id` not echoed on error, or error referencing a `request_id` never sent → response cannot be paired with a request | §5.3.6 (field-definition identity semantics) | Force an open error; assert pairing; assert no unsolicited errors | — |

#### `UpdateChannel` ([§5.3.7](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#537-updatechannel-client---server))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-UC-1 | Ω1 | Θ1 | `UpdateChannel` with a *smaller* `maximum_target` ignored | §5.3.7 ("upstream node MUST reflect the client's request (and send appropriate SetTarget message)"; note permitted delay) | Send `UpdateChannel`; await matching `SetTarget` | `SetTarget` |

#### `UpdateChannel.Error` ([§5.3.8](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#538-updatechannelerror-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-UCE-1 | Ω2 | Θ1 | `UpdateChannel` for a never-opened or closed `channel_id` gets no `UpdateChannel.Error` (crash, silence, or acceptance instead) | §5.3.8 ("Sent only when `UpdateChannel` message is invalid") | Send `UpdateChannel` with bogus `channel_id`; assert `UpdateChannel.Error` | — |
| M-UCE-2 | Ω2 | Θ1 | `UpdateChannel.Error` sent for a valid `UpdateChannel` (false rejection) | §5.3.8 ("When it is accepted by the server, no response is sent back") | Send valid `UpdateChannel`; assert no error response | — |

#### `CloseChannel` ([§5.3.9](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#539-closechannel-client---server-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-CC-1 | Ω1 | Θ1 | Server keeps sending jobs/`SetTarget` for a channel after `CloseChannel` | §5.3.9 ("The server MUST stop sending messages for the channel") | Close channel; assert silence | `NewMiningJob`, `NewExtendedMiningJob`, `SetTarget` |
| M-CC-2 | Ω3 | Θ1 | Server-initiated `CloseChannel` mid-session with unusable `reason_code`; client cannot re-open cleanly | §5.3.9 (spec gap: `reason_code` has no content constraints analogous to §3.5) | Resilience test | — |
| M-CC-3 | Ω3 | Θ1 | `CloseChannel` addressing a group channel doesn't close member channels (or closes unrelated ones) | §5.3.9 ("all channels belonging to such group MUST be closed") | Group topology probe (only if pool uses groups) | `SetGroupChannel` |

#### `SetExtranoncePrefix` ([§5.3.10](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5310-setextranonceprefix-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SEP-1 | Ω3 | Θ2 | New prefix applied retroactively to jobs sent before the change (validation mismatch) | §5.3.10 ("applicable for all jobs sent after this message") | Submit share on a pre-change job with the old prefix; assert acceptance | `NewExtendedMiningJob`, `SubmitSharesExtended` |

#### `SubmitSharesStandard` ([§5.3.11](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5311-submitsharesstandard-client---server))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SSS-1 | Ω1 | Θ2 | Valid share (meets channel target, correct job/ntime/extranonce) rejected — e.g. endianness bugs in U256 target comparison | §5.3.21 converse: hashes at-or-below target must be accepted | Grind real share; assert `SubmitShares.Success` | `SubmitShares.Success` |
| M-SSS-2 | Ω1 | Θ2 | Invalid share accepted (hash above target) — breaks difficulty accounting | §5.3.21 ("hashes higher than the specified target will be rejected") | Submit share just above target; assert `SubmitShares.Error` | `SubmitShares.Error` |
| M-SSS-3 | Ω2 | Θ1 | Out-of-bounds `ntime` accepted: `< min_ntime` of latest `SetNewPrevHash`, or too far past its receipt time | §5.3.11 (MUST; note spec text says "header_timestamp" but the field is `min_ntime`) | Boundary `ntime` submissions | `SetNewPrevHash`, `SubmitShares.Error` |
| M-SSS-4 | Ω2 | Θ2 | Share with rolled `nTime` inside the valid window rejected. Window: `ntime` ≥ `min_ntime` of latest `SetNewPrevHash` and ≤ that value + seconds since its receipt (outer bound: `MAX_FUTURE_BLOCK_TIME`); converse rejection covered by M-SSS-3 | §5.3.11 bounds | Submit with `ntime = min_ntime + k` within window; assert acceptance | `SetNewPrevHash` |
| M-SSS-5 | Ω3 | Θ2 | Duplicate share counted twice — no dedup on `(job_id, nonce, ntime, version)` | Expected pool behavior (spec-permissive; see caveats) | Resubmit identical valid share; assert error or unchanged accounting | `SubmitShares.Success` |
| M-SSS-6 | Ω2 | Θ1 | Share for unknown/closed/never-opened `channel_id` or `job_id` causes crash, silence, or acceptance | §5.2 scoping | Probe with bogus IDs | `SubmitShares.Error` |
| M-SSS-7 | Ω2 | Θ1 | `SubmitSharesStandard` sent on an extended channel mishandled | §5.3.11, §5.3.12 ("Only relevant for extended channels") | Wrong-type probe | `SubmitSharesExtended` |

#### `SubmitSharesExtended` ([§5.3.12](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5312-submitsharesextended-client---server))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SSE-1 | Ω1 | Θ2 | Valid share rejected (extended channel) | §5.3.21 converse | Grind real share; assert `SubmitShares.Success` | `SubmitShares.Success` |
| M-SSE-2 | Ω1 | Θ2 | Invalid share accepted (extended channel) | §5.3.21 | Submit share just above target; assert `SubmitShares.Error` | `SubmitShares.Error` |
| M-SSE-3 | Ω2 | Θ1 | Wrong `extranonce` length accepted (or correct length rejected) | §5.3.12 ("MUST be equal to the negotiated extranonce size") | Off-by-one extranonce lengths | `SubmitShares.Error` |
| M-SSE-4 | Ω2 | Θ1 | Out-of-bounds `ntime` accepted (extended channel) | §5.3.11 | Boundary `ntime` submissions | `SetNewPrevHash` |
| M-SSE-5 | Ω2 | Θ2 | Rolled `nTime` inside the valid window rejected (extended channel). Window as defined in M-SSS-4; converse rejection covered by M-SSE-4 | §5.3.11 bounds | Boundary-valid `ntime` share; assert acceptance | `SetNewPrevHash` |
| M-SSE-6 | Ω3 | Θ2 | Duplicate share counted twice (extended channel) | Expected pool behavior (spec-permissive; see caveats) | Resubmit identical valid share | `SubmitShares.Success` |
| M-SSE-7 | Ω2 | Θ1 | Bogus `channel_id`/`job_id` mishandled (extended channel) | §5.2 scoping | Probe with bogus IDs | `SubmitShares.Error` |
| M-SSE-8 | Ω2 | Θ1 | `SubmitSharesExtended` sent on a standard channel mishandled | §5.3.12 ("Only relevant for extended channels") | Wrong-type probe | `SubmitSharesStandard` |

#### `SubmitShares.Success` ([§5.3.13](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5313-submitsharessuccess-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SSOK-1 | Ω1 | Θ2 | Batch semantics broken: `last_sequence_number` ≠ client's last accepted seq; `new_submits_accepted_count` ≠ batch size; `new_shares_sum` ≠ sum of batch share difficulties; counters not reset between batches | §5.3.13 ("their respective counters MUST be reset when a new batch starts being processed") | Submit known-difficulty shares in controlled batches; verify all three fields arithmetically | `SubmitSharesStandard`, `SubmitSharesExtended` |
| M-SSOK-2 | Ω2 | Θ2 | `.Success` acknowledging shares never submitted (`last_sequence_number` ahead of what the client sent) | §5.3.13 | Sequence audit | — |

#### `SubmitShares.Error` ([§5.3.14](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5314-submitshareserror-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SSERR-1 | Ω1 | Θ1 | Errors silently dropped: incorrect submit never gets a `SubmitShares.Error` | §5.3.14 ("An error is immediately submitted for every incorrect submit attempt"; delayed validation allowed "as soon as the result is known") | Timeout assertion on every bad share | — |
| M-SSERR-2 | Ω2 | Θ1 | `SubmitShares.Error` referencing the wrong `sequence_number` or `channel_id` (cannot be paired with the offending submit) | §5.3.14 | Pairing assertion | — |

#### `NewMiningJob` ([§5.3.15](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5315-newminingjob-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-NMJ-1 | Ω1 | Θ1 | Server sends nothing (or another message) after standard channel open — the first message MUST be a `NewMiningJob` | §5.3.15 (verbatim MUST, standard channels) | Message-order assertion | `OpenStandardMiningChannel.Success`, `SetNewPrevHash` |
| M-NMJ-2 | Ω1 | Θ1 | First job arrives with `min_ntime` **set** (active job before any `SetNewPrevHash` exists) | §5.3.15 ("This first message will have min_ntime unset") | Assert first job is a future job | `SetNewPrevHash` |
| M-NMJ-3 | Ω1 | Θ1 | `NewMiningJob.channel_id` references an extended/group channel | §5.3.15 ("this must be a standard channel") | Type/channel-kind cross-check | — |
| M-NMJ-4 | Ω2 | Θ1 | `job_id` collisions across simultaneously valid jobs, or jobs referencing IDs never announced | §5.3.15–5.3.17 pairing semantics | Track `job_id` space | `SetNewPrevHash` |
| M-NMJ-5 | Ω2 | Θ2 | Server rejects shares with rolled BIP323 general-purpose version bits | §5.3.15 version field (bits "can be freely manipulated by the downstream node"; "MUST NOT rely on the upstream node to set the BIP323 bits") | Submit valid share with rolled version bits; assert acceptance | `SubmitSharesStandard`, `SubmitShares.Error` |

#### `NewExtendedMiningJob` ([§5.3.16](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5316-newextendedminingjob-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-NEMJ-1 | Ω2 | Θ1 | First message after extended channel open is not a `NewExtendedMiningJob` | Derived: §5.3.16 `min_ntime` semantics + §5.3.17 future-job pairing (no verbatim MUST for extended channels; see Spec Gaps) | Message-order assertion | `OpenExtendedMiningChannel.Success`, `SetNewPrevHash` |
| M-NEMJ-2 | Ω2 | Θ1 | First extended job arrives with `min_ntime` **set** (active job before any `SetNewPrevHash` exists) | Derived, as above: an active job "uses the prev_hash from the last received SetNewPrevHash message", which does not exist yet | Assert first job is a future job | `SetNewPrevHash` |
| M-NEMJ-3 | Ω1 | Θ1 | `NewExtendedMiningJob` targets a standard channel | §5.3.16 (extended/group channels only) | Type/channel-kind cross-check | — |
| M-NEMJ-4 | Ω1 | Θ1 | `NewExtendedMiningJob` sent on a connection where client set `REQUIRES_STANDARD_JOBS` | §5.3.16, §5.2.1 | Assert absence | `SetupConnection` |
| M-NEMJ-5 | Ω1 | Θ1 | BIP141 stripping violation: `coinbase_tx_prefix`/`coinbase_tx_suffix` still contain marker/flag/witness data → client computes wtxid-based merkle root, every share built on the job is garbage | §5.3.16 ("MUST be stripped of BIP141 fields"), §3.7.1 | Reconstruct coinbase on extended channel; assert txid (not wtxid) semantics | — |
| M-NEMJ-6 | Ω2 | Θ1 | `job_id` collisions across simultaneously valid jobs (extended) | §5.3.16–5.3.17 pairing | Track `job_id` space | `SetNewPrevHash` |
| M-NEMJ-7 | Ω2 | Θ2 | Server rejects shares with rolled BIP323 bits despite `version_rolling_allowed = True` | §5.3.16 | Submit valid share with rolled bits; assert acceptance | `SubmitSharesExtended`, `SubmitShares.Error` |

#### `SetNewPrevHash` ([§5.3.17](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5317-setnewprevhash-server---client-broadcast))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SNPH-1 | Ω2 | Θ1 | `SetNewPrevHash` activates a `job_id` never sent as a future job | §5.3.17, §5.3.15 | Cross-reference announced future jobs | `NewMiningJob`, `NewExtendedMiningJob` |
| M-SNPH-2 | Ω2 | Θ2 | Server keeps accepting shares for jobs invalidated by a `SetNewPrevHash` (stale acceptance → wrong accounting, misleading hashrate) | §5.3.17 ("only the job referenced by Job ID is valid") | Submit share for invalidated job; expect `SubmitShares.Error` | `SubmitShares.Error`, `SubmitShares.Success` |
| M-SNPH-3 | Ω2 | Θ1 | `min_ntime` wildly in the future (beyond `MAX_FUTURE_BLOCK_TIME`) → any block found would be invalid | §5.1 | Sanity-bound assertion | — |

#### `SetCustomMiningJob` ([§5.3.18](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5318-setcustomminingjob-client---server))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SCMJ-1 | Ω2 | Θ1 | `SetCustomMiningJob` accepted on a connection that never declared `REQUIRES_WORK_SELECTION` | §5.3.18 ("SetupConnection.flags MUST contain REQUIRES_WORK_SELECTION") | Send without declaring; expect `SetCustomMiningJob.Error` | `SetupConnection`, `SetCustomMiningJob.Error` |

#### `SetCustomMiningJob.Success` ([§5.3.19](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5319-setcustomminingjobsuccess-server---client))

No scenarios yet — custom-job flows are exercised by JDC suites.

#### `SetCustomMiningJob.Error` ([§5.3.20](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5320-setcustomminingjoberror-server---client))

No scenarios yet.

#### `SetTarget` ([§5.3.21](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5321-settarget-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-ST-1 | Ω2 | Θ1 | `SetTarget.maximum_target` above the client's declared `max_target` | §5.3.2 / §5.3.7 (client's max is a hard bound) | Assert bound on every `SetTarget` | `OpenStandardMiningChannel`, `UpdateChannel` |
| M-ST-2 | Ω1 | Θ2 | New target applied retroactively to active jobs that had `min_ntime` set | §5.3.21 ("not applicable for already received jobs with min_ntime=nTime") | Submit share valid under old target on a pre-`SetTarget` job; assert old target applies | `NewMiningJob`, `SubmitSharesStandard` |

#### `SetGroupChannel` ([§5.3.22](https://github.com/stratum-mining/sv2-spec/blob/main/05-Mining-Protocol.md#5322-setgroupchannel-server---client))

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| M-SGC-1 | Ω2 | Θ1 | `SetGroupChannel` sent on a connection with `REQUIRES_STANDARD_JOBS` | §5.3.22 ("can be sent only to connections that don't have REQUIRES_STANDARD_JOBS") | Assert absence | `SetupConnection` |
| M-SGC-2 | Ω1 | Θ1 | `group_channel_id` collides with an existing mining `channel_id` | §5.2.3 (shared namespace), §5.3.22 ("MUST ensure that a group channel has a unique channel ID") | Cross-check ID spaces | `OpenStandardMiningChannel.Success`, `OpenExtendedMiningChannel.Success` |

### Protocol Extensions ([§3.4](https://github.com/stratum-mining/sv2-spec/blob/main/03-Protocol-Overview.md#34-protocol-extensions), [§9](https://github.com/stratum-mining/sv2-spec/blob/main/09-Extensions.md))

One subsection per registered extension (see the §9 registry); future registrations append new subsections. Generic extension machinery (unknown `extension_type` handling, TLV invariants, negotiation-before-use) lives in the [General](#general-3135-5152) table as GEN-7…GEN-12.

**Applicability caveat:** a server that implements no extensions is compliant — per §3.4.1 and ext `0x0001` §4.3 it simply ignores `RequestExtensions`. Therefore *no response* to `RequestExtensions` is not a failure; the `X0001-*` scenarios below only apply once a server engages with the negotiation protocol.

#### Extension `0x0001` — [Extensions Negotiation](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/0x0001-extensions-negotiation.md)

##### `RequestExtensions`

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| X0001-REQ-1 | Ω2 | Θ1 | Server requires extensions the client didn't request, but keeps the connection open after the client doesn't retry | ext `0x0001` §4 ("the server MUST disconnect the client") | Omit a server-required extension from the request; do not retry; assert disconnect | `RequestExtensions.Error` |

##### `RequestExtensions.Success`

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| X0001-REQS-1 | Ω1 | Θ1 | `request_id` doesn't echo the request | ext `0x0001` §2 (field-definition identity semantics) | Random `request_id`s; assert pairing | — |
| X0001-REQS-2 | Ω1 | Θ1 | Extension-defined message framed with the wrong `extension_type` (must be `0x0001`, not `0x0000`) | §3.4.1, ext `0x0001` §3 (MUST) | Assert `extension_type == 0x0001` (after masking the `channel_msg` bit) on every negotiation frame | `RequestExtensions.Error` |

##### `RequestExtensions.Error`

| ID | Compliance | Testability | Failure mode | Spec basis | Detection | Involves |
|----|-----------|-------------|--------------|-----------|-----------|----------|
| X0001-REQE-1 | Ω2 | Θ1 | None of the requested extensions are supported, but server doesn't respond with `RequestExtensions.Error` | ext `0x0001` §4.1 ("MUST respond with `RequestExtensions.Error` if none of the requested extensions are supported") | Request only unallocated extension IDs; assert `Error` | — |
| X0001-REQE-2 | Ω2 | Θ1 | `Error` contents wrong: `unsupported_extensions` ≠ the requested-but-unsupported set, or `required_extensions` omits extensions the server requires | ext `0x0001` §1.1, §4.1 (see Spec Gaps for a field-name inconsistency in §4.1) | Request a mix of supported and bogus IDs; assert exact sets | — |
| X0001-REQE-3 | Ω1 | Θ1 | `request_id` doesn't echo the request | ext `0x0001` §2 (field-definition identity semantics) | Random `request_id`s; assert pairing | — |

#### Extension `0x0002` — [Worker-Specific Hashrate Tracking](https://github.com/stratum-mining/sv2-spec/blob/main/extensions/0x0002-worker-specific-hashrate-tracking.md)

No additional protocol-observable scenarios: its wire-level requirements (ignore unexpected TLVs, 32-byte `user_identity` limit) are covered by GEN-8/GEN-10, and per-worker hashrate accounting is not observable over the protocol. Reserved for future rows.

### Job Declaration Protocol Messages ([§6](https://github.com/stratum-mining/sv2-spec/blob/main/06-Job-Declaration-Protocol.md))

Reserved for future suites (JDS/JDC contexts). No scenarios yet.

#### `AllocateMiningJobToken` / `AllocateMiningJobToken.Success`
#### `ProvideMissingTransactions` / `ProvideMissingTransactions.Success`
#### `DeclareMiningJob` / `DeclareMiningJob.Success` / `DeclareMiningJob.Error`
#### `PushSolution`

Expansion seeds: §3.7.2 — unlike `NewExtendedMiningJob`, `DeclareMiningJob`'s coinbase fields MUST **NOT** be stripped of BIP141 fields (opposite polarity of M-NEMJ-5); token lifecycle; missing-transaction round-trips.

### Template Distribution Protocol Messages ([§7](https://github.com/stratum-mining/sv2-spec/blob/main/07-Template-Distribution-Protocol.md))

Reserved for future suites (TP contexts). No scenarios yet.

#### `CoinbaseOutputConstraints`
#### `NewTemplate`
#### `SetNewPrevHash` (TDP)
#### `RequestTransactionData` / `RequestTransactionData.Success` / `RequestTransactionData.Error`
#### `SubmitSolution`

Expansion seeds: §3.7.4 BIP141 handling on `NewTemplate`; `CoinbaseOutputConstraints` vs template coinbase outputs.

---

## Protocol expansion

The solo-pool scope exercises only the common and mining protocols. The JD/TD tables above are intentionally empty: expanding to JDS/TP/JDC suites means filling rows in existing tables, not restructuring. IDs are protocol-namespaced (`JD-`, `TD-`) from day one so mining and TDP messages with identical names (e.g. `SetNewPrevHash`) never collide. The same applies to extensions: each new entry in the §9 registry appends a subsection with `X<extid>-` IDs, without touching existing scenarios.

---

## Caveats

- **M-SSS-5 / M-SSE-6 (duplicate-share rejection)** are areas where the spec is deliberately permissive; the tests
  assert *expected pool behavior*, not a spec MUST. Suites should label them as such to avoid
  false "non-compliant" verdicts.
- **GEN-7…GEN-12 / X0001-\* (extensions)** are conditionally applicable: a server implementing no
  extensions is compliant if it silently ignores `RequestExtensions` (§3.4.1, ext `0x0001` §4.3).
  Suites must treat *no response* as "extension unsupported", never as a failure, and only evaluate
  `X0001-*` rows once the server engages with negotiation. GEN-11 applies to proxy suites only.

---

## Spec Gaps and Ambiguities

Places where the spec is silent, ambiguous, or inconsistent, with the affected scenarios and how the catalog treats them pending clarification. To be filed as `sv2-spec` issues.

| # | Gap | Affected scenarios | Treatment | Suggested spec change |
|---|-----|--------------------|-----------|------------------------|
| 1 | §5.2.2 says work-providing servers MUST support both channel kinds, but §5.3.1's `REQUIRES_EXTENDED_CHANNELS` flag lets a server refuse standard channels | C-SC-4, M-OEMC-1 | Held at Ω2 pending clarification — the only gap that blocks tiering | Clarify when refusing standard channels is compliant |
| 2 | `min_extranonce_size` has no honor-or-error rule (asymmetric with §5.3.2's explicit target rule) | M-OEMCS-3 | Ω2 (derived) | Mirror §5.3.2: satisfy `min_extranonce_size` or respond with `OpenMiningChannel.Error` |
| 3 | `SetNewPrevHash` referencing a never-announced `job_id` is not prohibited | M-SNPH-1 | Ω2 (derived) | Add "`job_id` MUST reference a previously announced future job" |
| 4 | "First message MUST be a job / `min_ntime` unset" exists only for standard channels (§5.3.15) | M-NEMJ-1, M-NEMJ-2 | Ω2 (derived via §5.3.16 `min_ntime` semantics + §5.3.17 future-job pairing) | Add the equivalent sentence to §5.3.16 |
| 5 | Server behavior on unknown `protocol` values (not 0/1/2) is undefined | C-SC-7 | Ω2 (derived) | Define: respond with `SetupConnection.Error` |
| 6 | Server-side handling of stale shares (jobs invalidated by `SetNewPrevHash`) is unspecified; stale acceptance silently inflates reported hashrate | M-SNPH-2 | Ω2 (derived) | Specify whether stale shares MUST/SHOULD be rejected |
| 7 | No content constraints on `CloseChannel.reason_code` (analogous to §3.5 `error_code` rules) | M-CC-2 | Ω3 | Apply §3.5-like constraints to `reason_code` |
| 8 | §5.3.11 refers to a `header_timestamp` field in `SetNewPrevHash`; the actual field is `min_ntime` | M-SSS-3, M-SSE-4 | Wording inconsistency (no tier impact) | Fix field name in §5.3.11 |
| 9 | Ext `0x0001` §4.1 says server-required extensions go in the "`requested_extensions` field"; the actual field is `required_extensions` | X0001-REQE-2 | Wording inconsistency (no tier impact) | Fix field name in ext `0x0001` §4.1 |
| 10 | No explicit rule that the server sends nothing before its `SetupConnection` response (§3.6.1's verbatim MUST covers the *client's* first message) | C-SC-1 | Ω2 (derived) | State that the server MUST NOT send any message before its `SetupConnection` response |

---

## Suite Coverage

✅ = covered; ❌ = not covered

| Scenario | Solo Pool |
|---|---|
| GEN-1 — `channel_msg` bit correctness | ❌ |
| GEN-2 — `extension_type` on core messages | ❌ |
| GEN-3 — error code control characters | ❌ |
| GEN-4 — error code non-printable bytes | ❌ |
| GEN-5 — `channel_id` scoping | ❌ |
| GEN-6 — job starvation (liveness) | ❌ |
| GEN-7 — unknown `extension_type` discarded | ❌ |
| GEN-8 — unexpected TLV fields ignored | ❌ |
| GEN-9 — server TLV placement/ordering | ❌ |
| GEN-10 — over-length TLV rejected | ❌ |
| GEN-11 — proxy forwards unknown-extension frames unmodified | ❌ |
| GEN-12 — no extension messages without negotiation | ❌ |
| GEN-13 — contradictory double response | ❌ |
| C-SC-1 — no messages before `SetupConnection` response | ❌ |
| C-SC-2 — boundary device-info/endpoint strings | ❌ |
| C-SC-3 — `REQUIRES_FIXED_VERSION` vs `REQUIRES_VERSION_ROLLING` | ❌ |
| C-SC-4 — `REQUIRES_STANDARD_JOBS` must be accepted | ❌ |
| C-SC-5 — `REQUIRES_EXTENDED_CHANNELS` coherence | ❌ |
| C-SC-6 — `REQUIRES_FIXED_VERSION` + `version_rolling_allowed` consistency | ❌ |
| C-SC-7 — unknown `protocol` value rejected | ❌ |
| C-SCS-1 — `SetupConnection` response type | ❌ |
| C-SCS-2 — `used_version` within negotiated range | ❌ |
| C-SCE-1 — feature-probing error flags | ❌ |
| C-SCE-2 — `SetupConnection.Error` on unsupported version | ❌ |
| M-OSMC-1 — target vs `max_target` on channel open | ❌ |
| M-OSMCS-1 — `request_id` echo (standard open) | ❌ |
| M-OSMCS-2 — `channel_id` uniqueness (standard) | ❌ |
| M-OSMCS-3 — `extranonce_prefix` uniqueness (standard) | ❌ |
| M-OEMC-1 — Extended channel support | ❌ |
| M-OEMC-2 — target vs `max_target` on extended open | ❌ |
| M-OEMCS-1 — `request_id` echo (extended open) | ❌ |
| M-OEMCS-2 — `channel_id` uniqueness (extended) | ❌ |
| M-OEMCS-3 — `extranonce_size` vs `min_extranonce_size` | ❌ |
| M-OEMCS-4 — group extranonce-size consistency | ❌ |
| M-OEMCS-5 — extranonce space uniqueness (extended) | ❌ |
| M-OMCE-1 — `request_id` echo on open error | ❌ |
| M-UC-1 — `UpdateChannel` target reduction honored | ❌ |
| M-UCE-1 — `UpdateChannel.Error` for invalid channel | ❌ |
| M-UCE-2 — no `UpdateChannel.Error` on valid update | ❌ |
| M-CC-1 — silence after `CloseChannel` | ❌ |
| M-CC-2 — server-initiated `CloseChannel` reason_code | ❌ |
| M-CC-3 — group-channel `CloseChannel` correctness | ❌ |
| M-SEP-1 — `SetExtranoncePrefix` non-retroactivity | ❌ |
| M-SSS-1 — valid-share acceptance (standard) | ❌ |
| M-SSS-2 — invalid-share rejection (standard) | ❌ |
| M-SSS-3 — `ntime` bounds enforcement (standard) | ❌ |
| M-SSS-4 — `nTime`-rolling acceptance (standard) | ❌ |
| M-SSS-5 — duplicate-share rejection (standard) | ❌ |
| M-SSS-6 — bogus IDs in share submission (standard) | ❌ |
| M-SSS-7 — `SubmitSharesStandard` on extended channel | ❌ |
| M-SSE-1 — valid-share acceptance (extended) | ❌ |
| M-SSE-2 — invalid-share rejection (extended) | ❌ |
| M-SSE-3 — extranonce length validation | ❌ |
| M-SSE-4 — `ntime` bounds enforcement (extended) | ❌ |
| M-SSE-5 — `nTime`-rolling acceptance (extended) | ❌ |
| M-SSE-6 — duplicate-share rejection (extended) | ❌ |
| M-SSE-7 — bogus IDs in share submission (extended) | ❌ |
| M-SSE-8 — `SubmitSharesExtended` on standard channel | ❌ |
| M-SSOK-1 — `SubmitShares.Success` batch accounting | ❌ |
| M-SSOK-2 — `last_sequence_number` audit | ❌ |
| M-SSERR-1 — `SubmitShares.Error` always sent for bad submits | ❌ |
| M-SSERR-2 — `SubmitShares.Error` sequence_number pairing | ❌ |
| M-NMJ-1 — first message after standard open is a job | ❌ |
| M-NMJ-2 — first standard job `min_ntime` unset | ❌ |
| M-NMJ-3 — `NewMiningJob` channel-kind cross-check | ❌ |
| M-NMJ-4 — standard `job_id` uniqueness and pairing | ❌ |
| M-NMJ-5 — version-rolling share acceptance (standard) | ❌ |
| M-NEMJ-1 — first message after extended open is a job | ❌ |
| M-NEMJ-2 — first extended job `min_ntime` unset | ❌ |
| M-NEMJ-3 — `NewExtendedMiningJob` channel-kind cross-check | ❌ |
| M-NEMJ-4 — `NewExtendedMiningJob` vs `REQUIRES_STANDARD_JOBS` | ❌ |
| M-NEMJ-5 — BIP141 stripping in coinbase tx data | ❌ |
| M-NEMJ-6 — extended `job_id` uniqueness and pairing | ❌ |
| M-NEMJ-7 — version-rolling share acceptance (extended) | ❌ |
| M-SNPH-1 — `SetNewPrevHash` references known future `job_id` | ❌ |
| M-SNPH-2 — stale-share rejection | ❌ |
| M-SNPH-3 — `min_ntime` sanity bounds | ❌ |
| M-SCMJ-1 — `SetCustomMiningJob` requires declared work selection | ❌ |
| M-ST-1 — `SetTarget` bound vs client `max_target` | ❌ |
| M-ST-2 — `SetTarget` non-retroactivity | ❌ |
| M-SGC-1 — `SetGroupChannel` vs `REQUIRES_STANDARD_JOBS` | ❌ |
| M-SGC-2 — group/channel ID namespace separation | ❌ |
| X0001-REQ-1 — disconnect when required extensions ignored | ❌ |
| X0001-REQS-1 — `request_id` echo on negotiation success | ❌ |
| X0001-REQS-2 — negotiation frames carry `extension_type = 0x0001` | ❌ |
| X0001-REQE-1 — `Error` when nothing requested is supported | ❌ |
| X0001-REQE-2 — `Error` unsupported/required sets exact | ❌ |
| X0001-REQE-3 — `request_id` echo on negotiation error | ❌ |

Note: more columns will be added as the project matures (Pool, JDS, TP, Proxy suites).
