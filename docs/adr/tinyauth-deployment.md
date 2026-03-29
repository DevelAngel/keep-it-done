---
status: proposed
date: 2026-03-29
---

# Tinyauth for Public Internet Authentication

## Context and Problem Statement

The family task management system needs public internet access without VPN. Four family members need to reach the browser interface from anywhere. The application was designed for trusted local networks and has no built-in authentication — opening it to the internet without a protection layer is not acceptable. Traefik v3 is already used as reverse proxy. How should authentication be added without building it into the application?

## Decision Drivers

- Authentication must not contaminate the application — `kid-server` stays focused on tasks
- Minimal operational overhead for a home lab with 4 users
- No external identity provider dependency
- Works as a Traefik forward-auth provider
- Lightweight enough to run alongside other containers on home hardware

## Considered Options

- Tinyauth
- Authelia
- Basic Authentication in Traefik
- VPN-only access (Tailscale, WireGuard)
- OAuth2 Proxy (GitHub/Google login)
- Self-implemented authentication in Axum
- Tailscale Funnel

## Decision Outcome

Chosen option: "Tinyauth", because it acts as a Traefik ForwardAuth provider — all authentication is handled before requests reach `kid-server`. The application never handles credentials or sessions. Tinyauth is stateless (bcrypt-hashed user database in an environment variable), extremely lightweight (<20 MB image), and requires no external infrastructure.

Architecture:

```
Internet → Traefik (HTTPS) → Tinyauth (ForwardAuth) → kid-server
```

`tasks.example.com` is protected by the ForwardAuth middleware. `kid-server` receives only authenticated requests and reads `Remote-User` from headers if needed.

### Consequences

- Good, because `kid-server` has zero authentication logic — clean separation of concerns
- Good, because Tinyauth is stateless — no session database required
- Good, because Podman Quadlet + systemd manages the container natively (`systemctl status tinyauth`)
- Good, because a single Tinyauth instance protects all future self-hosted services (SSO across the family's infrastructure)
- Good, because rate limiting and brute-force protection are built into Tinyauth
- Bad, because Tinyauth is a single point of failure for login — if it crashes, no one can log in even though `kid-server` is healthy
- Bad, because no 2FA, no granular per-route permissions, no self-service password reset
- Bad, because memory-based sessions by default — container restart forces re-login (mitigated by mounting a `/data` volume)
- Bad, because less battle-tested than Authelia — smaller community, fewer integration examples

## Pros and Cons of the Options

### Tinyauth

- Good, because minimal attack surface — focused, small codebase
- Good, because zero infrastructure beyond a container
- Good, because bcrypt user management via environment variable
- Bad, because no 2FA
- Bad, because no granular access control

### Authelia

- Good, because 2FA (TOTP, WebAuthn), LDAP, granular ACLs, password reset
- Bad, because enterprise-grade complexity for 4 users — YAML DSL, Redis/database dependency, high setup and maintenance overhead
- Bad, because overkill for a family task list

### Basic Authentication in Traefik

- Good, because no additional container
- Bad, because credentials sent with every request (credential stuffing risk)
- Bad, because no brute-force protection, no session concept

### VPN-only access

- Good, because no internet exposure at all
- Bad, because friction kills adoption — "Enable VPN → wait → open browser" vs "open browser"
- Bad, because cognitive overhead for quick checks ("What's next?")

### OAuth2 Proxy (GitHub/Google)

- Good, because no password management, inherits existing 2FA
- Bad, because external dependency — GitHub outage blocks access to the shopping list
- Bad, because conceptual mismatch — "Log in with GitHub to see household tasks"

### Self-implemented authentication in Axum

- Good, because full control
- Bad, because session management, timing vulnerabilities, secure token generation — security code is not the core competency here
- Bad, because maintenance burden far exceeds the cost of deploying Tinyauth

### Tailscale Funnel

- Good, because hybrid: public endpoint with Tailscale authentication
- Bad, because locks into Tailscale ecosystem
- Bad, because requires Tailscale on all client devices

## More Information

**Deployment** uses Podman Quadlet with systemd units:

- Traefik (ports 80/443, reads container labels)
- Tinyauth (ForwardAuth provider, defines the middleware)
- kid-server (internal only, no public ports)

**User management:** Users are defined in the `USERS` environment variable as `username:$2a$10$bcrypt-hash`. Add a user: generate hash with Tinyauth's built-in command, update the env var, `systemctl restart tinyauth`.

**Session persistence:** Mount `/data` volume so sessions survive container restarts.

**Selective protection:** Protected services include the Tinyauth middleware in their Traefik labels. Public services omit it. This allows `tasks.example.com` to require login while `www.example.com` remains public.
