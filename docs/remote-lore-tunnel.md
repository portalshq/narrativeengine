# Remote Lore Server Tunnel

Connect NAP on a remote machine to a local lore server using Chisel for TCP + UDP forwarding.

## Lore Server Ports

| Protocol | Port | Purpose |
|----------|------|---------|
| QUIC (UDP) | 41337 | Primary transport |
| gRPC (TCP) | 41337 | API transport |
| HTTP (TCP) | 41339 | Store/health |

## Install

```bash
# macOS dev machine
brew install chisel-tunnel

# Linux remote
curl https://i.jpillora.com/chisel! | bash
```

## Setup

### 1. Start Chisel server (remote machine)

```bash
chisel server -p 8080 --reverse --auth loredev:hunter2
```

### 2. Start Chisel client (dev machine)

```bash
chisel client remote-host:8080 \
  41337/udp:127.0.0.1:41337/udp \
  R:41337:127.0.0.1:41337 \
  R:41339:127.0.0.1:41339 \
  --auth loredev:hunter2
```

### 3. Configure NAP (remote machine)

```bash
export NAP_LORE_URL_BASE='lore://localhost:41337'
export NAP_WORKSPACE_ID='default'
```

## How It Works

| Flag | Direction | Protocol | Description |
|------|-----------|----------|-------------|
| `41337/udp:127.0.0.1:41337/udp` | Client → Server | UDP | QUIC traffic |
| `R:41337:127.0.0.1:41337` | Server → Client | TCP | gRPC traffic (reverse) |
| `R:41339:127.0.0.1:41339` | Server → Client | TCP | HTTP traffic (reverse) |

## Alternatives Considered

- **socat**: Doesn't preserve UDP datagram boundaries — QUIC is sensitive to framing, packets can merge
- **WireGuard**: Overkill for 3 ports; requires root, TUN device setup, and NetworkExtension permissions on macOS
- **SSH `-w`**: Requires root on both sides, painful on macOS with utun devices
- **frp**: More feature-rich than needed; requires config files for both server and client
