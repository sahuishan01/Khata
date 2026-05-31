#!/data/data/com.termux/files/usr/bin/bash
# Expose the backend via Cloudflare Tunnel.
# Before running: pkg install cloudflared
# Update .env: BIND_ADDR=0.0.0.0:8080  CORS_ORIGINS=https://your-tunnel.trycloudflare.com
set -e
exec cloudflared tunnel --url http://localhost:8080
