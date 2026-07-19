# Lore Server Tunnel

Connect NAP to a remote lore server (or vice versa) using Chisel for TCP + UDP forwarding.

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

## Use Case A: Local NAP → Remote Lore

Your local NAP client talks to a lore server running on a remote machine.

### 1. Start Chisel server (remote machine)

```bash
chisel server -p 8080 --reverse --auth loredev:hunter2
```

### 2. Start Chisel client (local machine)

```bash
chisel client --auth loredev:hunter2 remote-host:8080 \
  41337:127.0.0.1:41337 \
  41337/udp:127.0.0.1:41337/udp \
  41339:127.0.0.1:41339
```

This binds local ports 41337 (TCP+UDP) and 41339 (TCP) and forwards them through the tunnel to the remote lore server.

### 3. Configure NAP (local machine)

```bash
export NAP_LORE_URL_BASE='lore://localhost:41337'
export NAP_WORKSPACE_ID='default'
```

### How It Works (Use Case A)

| Flag | Direction | Protocol | Description |
|------|-----------|----------|-------------|
| `41337:127.0.0.1:41337` | Local → Remote | TCP | gRPC traffic (forward) |
| `41337/udp:127.0.0.1:41337/udp` | Local → Remote | UDP | QUIC traffic (forward) |
| `41339:127.0.0.1:41339` | Local → Remote | TCP | HTTP traffic (forward) |

---

## Use Case B: Remote NAP → Local Lore

A remote NAP client talks to your local lore server (dev machine).

### 1. Start Chisel server (remote machine)

```bash
chisel server -p 8080 --reverse --auth loredev:hunter2
```

### 2. Start Chisel client (local machine)

```bash
chisel client --auth loredev:hunter2 remote-host:8080 \
  41337/udp:127.0.0.1:41337/udp \
  R:41337:127.0.0.1:41337 \
  R:41339:127.0.0.1:41339
```

The `R:` prefix tells the chisel **server** to listen on those ports and forward connections back through the tunnel to the local machine.

### 3. Configure NAP (remote machine)

```bash
export NAP_LORE_URL_BASE='lore://localhost:41337'
export NAP_WORKSPACE_ID='default'
```

### How It Works (Use Case B)

| Flag | Direction | Protocol | Description |
|------|-----------|----------|-------------|
| `41337/udp:127.0.0.1:41337/udp` | Client → Server | UDP | QUIC traffic |
| `R:41337:127.0.0.1:41337` | Server → Client | TCP | gRPC traffic (reverse) |
| `R:41339:127.0.0.1:41339` | Server → Client | TCP | HTTP traffic (reverse) |

---

## Troubleshooting

### `--auth` flag position

The `--auth` flag **must come before** the server URL. This is a common gotcha:

```bash
# ✅ Correct
chisel client --auth loredev:hunter2 remote-host:8080 ...

# ❌ Wrong — will get "Failed to decode remote '--auth': Missing ports"
chisel client remote-host:8080 --auth loredev:hunter2 ...
```

### Go routing issues (Tailscale / VPN)

If `chisel client` fails with `no route to host` but `curl` and `nc` can reach the server, the Go network stack may be routing through the wrong interface (common with Tailscale). Workaround: SSH-tunnel the chisel server port to localhost.

```bash
# 1. SSH tunnel to chisel server port
ssh -f -N -L 18080:127.0.0.1:8080 remote-host

# 2. Point chisel client at the tunnel
chisel client --auth loredev:hunter2 localhost:18080 ...
```

### Port already in use

If the chisel server reports `Server cannot listen on R:41337`, another process (e.g., lore server) is already bound to that port. Either stop the conflicting process, or use a different port:

```bash
# Forward to a different local port instead
R:41338:127.0.0.1:41337
# Then configure NAP to use lore://localhost:41338
```

### SSH alias resolution

If `remote-host` is an SSH alias (e.g., `andresb`), Go won't resolve it. Use the IP address directly. Find it with:

```bash
grep -A 5 "Host <alias>" ~/.ssh/config
```

## Alternatives Considered

- **socat**: Doesn't preserve UDP datagram boundaries — QUIC is sensitive to framing, packets can merge
- **WireGuard**: Overkill for 3 ports; requires root, TUN device setup, and NetworkExtension permissions on macOS
- **SSH `-w`**: Requires root on both sides, painful on macOS with utun devices
- **frp**: More feature-rich than needed; requires config files for both server and client
