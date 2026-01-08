# JIAYI-FSP

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
