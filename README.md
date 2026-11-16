# JIAYI-FSP
（[中文文档](README.zh-CN.md)）

JIAYI File Sync Protocol (JIAYI-FSP) is a hybrid file synchronization protocol
designed for trusted local networks.

- Control Plane: MQTT
- Data Plane: UDP

✅ Status: v1.1.0 (Draft — spec under revision)

This release completes the control-plane timing semantics and fixes several
definition errors carried over from v1.0.0. It is still **under revision** —
the wire format is not frozen yet.

v1.0.0 aligned the specification with the reference implementation
`jiayi-fsp-core` 0.2.0 / `jiayi-client` 1.0.0.

## Goals

- Efficient large file sync on LAN
- Disk-backed UDP receiving (no large memory buffering)
- Client-isolated MQTT topics via client_id

## Non-Goals

- Internet-scale transfer
- Background mobile transfer
- Untrusted peer-to-peer networks

## Repository Structure

This repository is dedicated to the **protocol itself** — specification,
reference implementation and developer tooling. End-user product applications
are out of scope.

```
proto/        Protocol data structures (C / Rust)
docs/spec/    Normative specification — single source of truth
docs/         Spec sources served by GitHub Pages (build artifacts not committed)
doc/          Single-file consolidated specification (generated, do not edit)
examples/     Broker ACL and reference implementation examples
  rust/       Executable conformance suite (deterministic simulation + real UDP)
tools/        Spec tooling (compilation generator / consistency checker) + debug tools
site/         Protocol website source
```

### Specification Workflow

`docs/spec/` is the single source of truth. The consolidated document under `doc/`
is **generated** — never edit it by hand.

```bash
python3 tools/build-spec-compilation.py           # regenerate the compilation
python3 tools/check-spec-consistency.py           # endianness / types / topics / proto alignment
python3 tools/build-spec-compilation.py --check   # CI mode
```

## Website Deployment

The protocol website is published to the **`gh-pages` branch**; build artifacts
are **not** committed to `main` (hashed asset names would otherwise churn
hundreds of files on every change).

```bash
cd site
npm run build        # tsc -b && vite build
npm run copy:spec    # copy docs/spec + docs/diagrams into dist/ (else /spec/*.md 404s)
npm run deploy       # publish dist/ to the gh-pages branch
```

CI (`.github/workflows/deploy-pages.yml`) does this automatically on push to
`main`.

> **One-time repository setting**: Settings → Pages → Source = *Deploy from a
> branch*, branch `gh-pages`, directory `/(root)`.

## Reference Implementation

`examples/rust/` turns the specification's **normative requirements** into
executable assertions — a deterministic simulation (virtual clock + virtual
broker + fault-injectable virtual UDP) plus a real-UDP loopback example.

```bash
cd examples/rust
cargo test            # 27 core unit tests + 9 conformance scenarios
cargo run -p fsp-sim  # run every scenario and print the observations
```

Each scenario pins one normative rule — zero-loss must not deadlock, holes must
be repaired, `next_chunk_id` must be the *first missing* chunk, and so on.

## Documentation Quick Links

 - [Overview](docs/spec/00-overview.md)
 - [Architecture](docs/spec/01-architecture.md)
 - [Terminology](docs/spec/02-terminology.md)
 - [Protocol Stack](docs/spec/05-protocol-stack.md)
 - [Control Plane (MQTT)](docs/spec/06-control-plane-mqtt.md)
 - [Data Plane (UDP/TCP)](docs/spec/07-data-plane-udp.md)
 - [Packet Format](docs/spec/08-packet-format.md)
 - [Session State Machine](docs/spec/10-state-machine.md)
 - [Flow Control](docs/spec/11-flow-control.md)
 - [Reliability Model](docs/spec/12-reliability-model.md)
 - [Resume & Recovery](docs/spec/13-resume-and-recovery.md)
 - [Download Flow](docs/spec/14-download-flow.md)
 - [NAT & Network Limits](docs/spec/15-nat-and-network-limits.md)
 - [Mobile Background Limits](docs/spec/16-mobile-background-limits.md)
 - [Memory & Disk Model](docs/spec/17-memory-and-disk-model.md)
 - [Bitmap & Chunk Map](docs/spec/18-bitmap-and-chunk-map.md)
 - [Error Codes](docs/spec/19-error-codes.md)
 - [Security Model](docs/spec/20-security-model.md)
 - [MQTT Topic Spec](docs/spec/22-topic-spec.md)
 - [Client Identity](docs/spec/23-client-identity.md)
 - [Sequence Diagrams](docs/spec/24-sequence-diagrams.md)
 - [Implementation Guide](docs/spec/25-implementation-guide.md)
 - [Reference MVP](docs/spec/26-reference-mvp.md)
