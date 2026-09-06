---
name: pve
description: Query and inspect the user's Proxmox VE (PVE) server via its REST API using curl — list nodes, VMs (qemu), containers (lxc), storage, tasks, and configs. Use when the user asks about their PVE / Proxmox / homelab virtualization state, VM status, or asks to check their PVE configuration.
---

# pve

Read-only access to the user's Proxmox VE server. All queries are plain `curl` calls — no SDK needed.

## Connection

```bash
BASE="https://192.168.0.9:8006"
TOKEN="root@pam!ai-reader=4a0955f1-f075-4c0f-9d68-a756c9151583"
```

- Auth header format: `Authorization: PVEAPIToken=<user>!<tokenid>=<secret>`
- `-k` is REQUIRED (self-signed certificate)
- Append `/api2/json/...` to the base URL; responses are `{"data": ...}` JSON

Smoke test first when connectivity is unclear:

```bash
curl -sk -H "Authorization: PVEAPIToken=$TOKEN" $BASE/api2/json/version
```

## Token is READ-ONLY

The token has privilege separation and read-only permissions. **GET works; POST/PUT/DELETE will fail with 403.** Never attempt mutations (start/stop VM, create, delete) with this token — if the user wants changes, ask them to do it in the PVE web UI or provide a read-write token.

## Common endpoints

| Purpose | Endpoint (GET) |
|---------|----------------|
| Overall inventory: all nodes + VMs + CTs + storage | `/api2/json/cluster/resources` |
| Node list | `/api2/json/nodes` |
| Single node status (CPU/mem/load/uptime) | `/api2/json/nodes/{node}/status` |
| VMs on a node | `/api2/json/nodes/{node}/qemu` |
| VM status | `/api2/json/nodes/{node}/qemu/{vmid}/status/current` |
| VM config (cores, memory, disks, net) | `/api2/json/nodes/{node}/qemu/{vmid}/config` |
| VM snapshots | `/api2/json/nodes/{node}/qemu/{vmid}/snapshot` |
| Containers on a node | `/api2/json/nodes/{node}/lxc` |
| CT status / config | `/api2/json/nodes/{node}/lxc/{vmid}/status/current` · `.../config` |
| Storage on a node | `/api2/json/nodes/{node}/storage` |
| Network config of a node | `/api2/json/nodes/{node}/network` |
| Recent tasks (backups, migrations, errors) | `/api2/json/nodes/{node}/tasks` |
| Backup jobs | `/api2/json/cluster/backup` |
| Cluster status / quorum | `/api2/json/cluster/status` |

`{node}` = node name from `/api2/json/nodes` (e.g. `pve`), `{vmid}` = numeric VM/CT id.

## Examples

```bash
# Full inventory — best first call: every VM/CT with status, node, cpu/mem usage
curl -sk -H "Authorization: PVEAPIToken=$TOKEN" \
  $BASE/api2/json/cluster/resources | jq '.data[] | {name, vmid, type, status, node, maxmem, maxcpu}'

# Only running VMs
curl -sk -H "Authorization: PVEAPIToken=$TOKEN" \
  $BASE/api2/json/cluster/resources | jq '[.data[] | select(.type=="qemu" and .status=="running")]'

# VM 101 config on node "pve"
curl -sk -H "Authorization: PVEAPIToken=$TOKEN" \
  $BASE/api2/json/nodes/pve/qemu/101/config | jq .data

# Recent tasks, newest first
curl -sk -H "Authorization: PVEAPIToken=$TOKEN" \
  $BASE/api2/json/nodes/pve/tasks | jq '.data[:10] | map({id, type, status, endtime})'
```

## Tips

- Always pipe through `jq` — raw output is verbose and hard to read.
- `/cluster/resources` covers 90% of "what's running" questions; go per-node for configs.
- Task `status` field: a failed task has `"status": "..."` containing `ERRor` — grep for it when diagnosing.
- `maxmem` is bytes; `mem`/`maxmem` in cluster/resources are bytes too (divide by 2^30 for GiB).
- If curl hangs, the server may be unreachable — use `--connect-timeout 5`.
