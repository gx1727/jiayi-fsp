# JIAYI-FSP
（[中文文档](README.zh-CN.md)）

JIAYI File Sync Protocol (JIAYI-FSP) is a hybrid file synchronization protocol
designed for trusted local networks.

- Control Plane: MQTT
- Data Plane: UDP

✅ Status: v1.0.0 (Production Ready)

This release aligns the specification with the reference implementation
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
docs/         GitHub Pages deployment (spec + protocol website build)
doc/          Single-file consolidated specification (generated, do not edit)
examples/     Broker ACL and reference implementation examples
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
