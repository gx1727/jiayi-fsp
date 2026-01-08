# JIAYI-FSP
（[中文文档](README.zh-CN.md)）

JIAYI File Sync Protocol (JIAYI-FSP) is a hybrid file synchronization protocol
designed for trusted local networks.

- Control Plane: MQTT
- Data Plane: UDP

⚠️ Status: Experimental (v0.1)

## Goals

- Efficient large file sync on LAN
- Disk-backed UDP receiving (no large memory buffering)
- Client-isolated MQTT topics via client_id

## Non-Goals

- Internet-scale transfer
- Background mobile transfer
- Untrusted peer-to-peer networks

## Documentation Quick Links

- Overview: docs/spec/00-overview.md
- Architecture: docs/spec/01-architecture.md
- Terminology: docs/spec/02-terminology.md
- Protocol Stack: docs/spec/05-protocol-stack.md
- Control Plane (MQTT): docs/spec/06-control-plane-mqtt.md
- Data Plane (UDP/TCP): docs/spec/07-data-plane-udp.md
- Packet Format: docs/spec/08-packet-format.md
- Session State Machine: docs/spec/10-state-machine.md
- Flow Control: docs/spec/11-flow-control.md
- Reliability Model: docs/spec/12-reliability-model.md
- Resume & Recovery: docs/spec/13-resume-and-recovery.md
- Download Flow: docs/spec/14-download-flow.md
- NAT & Network Limits: docs/spec/15-nat-and-network-limits.md
- Mobile Background Limits: docs/spec/16-mobile-background-limits.md
- Memory & Disk Model: docs/spec/17-memory-and-disk-model.md
- Bitmap & Chunk Map: docs/spec/18-bitmap-and-chunk-map.md
- Error Codes: docs/spec/19-error-codes.md
- Security Model: docs/spec/20-security-model.md
- MQTT Topic Spec: docs/spec/22-topic-spec.md
- Client Identity: docs/spec/23-client-identity.md
- Sequence Diagrams: docs/spec/24-sequence-diagrams.md
- Implementation Guide: docs/spec/25-implementation-guide.md
- Reference MVP: docs/spec/26-reference-mvp.md
 
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
