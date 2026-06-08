# Security model

This document describes the framing-integrity decisions `mavlink-codec` makes while
parsing a byte stream, and the reasoning behind them. The behaviors below are exercised by
the exploit suites under [`tests/exploits`](tests/exploits): `packet_in_packet`,
`desync_liveness`, and `forgery_drops`.

## Resync policy: discard the whole declared frame

When a frame is rejected — bad CRC, zeroed system/component id under the drop policies,
unsupported incompatibility flags, or a failed signature — the decoder discards the entire
declared frame (`CodecState::Discarding { remaining: packet_size }`) before looking for the
next start-of-frame (STX) byte.

It deliberately does **not** rescan for the next STX starting at the byte right after the
rejected marker. That "forward rescan" is what makes most MAVLink parsers vulnerable to
**packet-in-packet injection**: an attacker sends an outer frame with a deliberately bad CRC
whose declared payload contains a fully valid inner frame. A parser that resyncs into the
rejected bytes will happily emit the attacker-chosen inner frame. By skipping the whole
declared length, we refuse to resync inside a rejected frame and the embedded inner frame is
discarded along with its outer (see `tests/exploits/packet_in_packet`).

Reference: rust-mavlink PR #508, "discard rejected mavlink frames entirely"
(<https://github.com/mavlink/rust-mavlink/pull/508>).

### Trade-off and the rejected `RESYNC_FROM_NEXT_STX` toggle

The cost of whole-frame discard is liveness: if the declared `len` byte itself is corrupt,
we may over-skip and drop a legitimate frame that followed the rejected one. The
`desync_liveness` suite pins this behavior.

An opt-in `RESYNC_FROM_NEXT_STX` const-generic (resync from the byte after the STX) was
considered to match the robustness of other implementations, but was **not** added: enabling
it reintroduces the packet-in-packet injection vector above. Whole-frame discard remains the
only resync strategy, as a security-first default.

## Unknown message ids (`ACCEPT_UNKNOWN_MSGID`)

By default, a frame whose message id is absent from the compiled dialect cannot be
CRC-validated (its `extra_crc` is unknown), so it fails CRC and is discarded like any other
rejected frame.

Routers need to forward messages they do not understand. The opt-in `ACCEPT_UNKNOWN_MSGID`
const-generic changes the CRC-failure path: if the message id is genuinely unknown
(`is_known_msgid` returns `false`), the frame is **forwarded unvalidated** instead of being
dropped. Genuinely corrupt frames carrying a *known* message id are still rejected.

Security implications of enabling this mode:

- Forwarded unknown frames carry **no integrity guarantee** — by definition we cannot check
  their CRC. Downstream consumers must validate them.
- It does **not** weaken packet-in-packet protection. The unknown outer frame is forwarded as
  a single opaque frame; the decoder still never resyncs into its payload, so an embedded
  inner frame is not extracted as a separate packet.

## Signature verification (`VERIFY_SIGNATURE`)

When enabled, accepted v2 frames must carry a valid MAVLink 2 signature, and all v1 frames are
rejected (v1 cannot be signed). A message authentication code is the only mechanism that closes
the residual packet-in-packet cases where the forged outer declares a tiny length
(see `tests/exploits/packet_in_packet/len1.rs`).

Verification is zero-copy: the SHA-256 signature is computed in place over the buffered frame
bytes (`src/signing.rs`), without materializing rust-mavlink's fixed-size `MAVLinkV2MessageRaw`.
Because the codec verifies through `Decoder::decode(&mut self)`, the per-stream timestamp replay
state is mutated through exclusive `&mut` access and needs no locking. The cost of this is that
the codec carries its own copy of the signing/replay logic, which must track the MAVLink signing
specification.
