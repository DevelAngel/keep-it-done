# ADR: Tinyauth for Public Internet Authentication

## Status

Proposed

## Context

The family task management system needs public internet access without VPN. The browser interface displays tasks and allows marking them complete. Four family members need access from anywhere.

The challenge: The application was designed for trusted local networks. It has no built-in authentication. Opening it to the internet creates immediate security problems.

We use Traefik v3 as reverse proxy. `kid-server` runs Axum with Leptos in hydrate mode. The RPC interface (TCP `127.0.0.1:9000`) is local-only and not exposed. Only the browser HTTP interface faces the internet.

## Decision

We will use Tinyauth as forward authentication provider, integrated with Traefik.

Tinyauth sits between Traefik and the task server. Before any request reaches our application, Traefik asks Tinyauth: "Is this user authenticated?" If not, redirect to login. If yes, pass through with user identity in headers.

Our Axum application receives only authenticated requests. It never sees the login process. It never handles credentials. It simply reads `Remote-User` from headers when needed (though for this use case, even that is optional—we only care that someone authenticated, not necessarily who).

## Implementation Approach

The deployment uses Podman Quadlet with systemd units:

- Traefik (public-facing, ports 80/443, Docker/Podman labels)
- Tinyauth (Podman Quadlet container with systemd)
- kid-server (Podman Quadlet container with systemd, isolated from internet)

Tinyauth configuration:

- Environment variable-based user database (bcrypt hashes)
- Stateless design: no separate session database needed
- Session cookies with standard security flags
- Rate limiting built-in
- Login page served at dedicated subdomain (e.g., `auth.example.com`)

Access control:

- Protected services (e.g., `tasks.example.com`) require authentication via Traefik middleware
- Other services can remain publicly accessible (e.g., `www.example.com`)
- Per-route protection via Traefik labels

User management:

- Users defined via `USERS` environment variable
- Passwords hashed with bcrypt
- Simple format: `username:$2a$10$hash`
- Add users by updating environment and restarting container

## Consequences

### Positive

**Security becomes someone else's problem.** We don't implement session management, brute-force protection, or CSRF guards. Tinyauth handles all of it. Lightweight, focused implementation with minimal attack surface.

**Our application stays simple.** No authentication logic. No session cookies. No password handling. The Axum code does tasks, nothing else. If we need to know who made a change, we read a header—but we don't validate it, because Tinyauth already did.

**Minimal resource footprint.** Tinyauth is extremely lightweight (single binary, <20MB image). No database dependencies. Sessions stored in memory with configurable persistence options. Ideal for home lab environments.

**Simple deployment model.** Podman Quadlet with systemd integration means standard service management. `systemctl status tinyauth` shows state. Logs via journalctl. Automatic restarts on failure. No Docker Compose orchestration needed.

**Future-proof.** When we add more self-hosted services (Nextcloud, Grafana, documentation wiki), they all use the same Tinyauth instance. Single sign-on across the family's infrastructure. One login, many applications.

**Clean UX.** Modern login page with dark mode support. Users stay logged in across sessions (with security timeouts). Mobile-responsive design. Feels professional despite simplicity.

### Negative

**Another point of failure.** If Tinyauth crashes, nobody can log in—even though the task server is fine. We need monitoring for both services. Recovery procedures must account for Tinyauth state.

**Limited advanced features.** Tinyauth focuses on simplicity. No built-in 2FA. No LDAP integration. No sophisticated access control rules. For basic username/password authentication, this is fine. For complex requirements, it's limiting.

**Less mature ecosystem.** Tinyauth is newer and less battle-tested than solutions like Authelia. Smaller community. Fewer integration examples. Documentation is basic. Edge cases may not be covered.

**No granular access control.** Authentication is binary: logged in or not. No user groups, no per-resource permissions, no time-based access rules. If you need "Admin can access everything, User can only access specific routes", Tinyauth doesn't provide that natively.

**Password management responsibility.** We must securely generate and distribute initial passwords. No self-service password reset. No account recovery flow. Lost passwords require manual hash regeneration and environment update.

**Memory-based sessions by default.** Container restart loses all active sessions. Users must re-login. Can be mitigated with persistent storage, but adds complexity. For home lab with occasional restarts, this is noticeable friction.

### Mitigations

**Simplify deployment with Podman Quadlet.** Systemd manages containers natively. `systemctl start tinyauth` and `systemctl start kid-server`. Dependencies expressed in unit files. Automatic restart on failure. No orchestration layer needed.

**Monitor both services.** Systemd provides built-in status and restart policies. Journalctl for centralized logs. Simple healthcheck scripts via systemd timers. Alert if either service is down.

**Document common operations.** Adding a user: "Generate bcrypt hash, update `USERS` environment variable, `systemctl restart tinyauth`." Password change: regenerate hash, update environment, restart. Simple, repeatable procedures.

**Persistent session storage.** Mount `/data` volume for session persistence. Sessions survive container restarts. Users stay logged in. Backup includes session data if needed (though sessions can be sacrificed—users just re-login).

**Accept the trade-offs.** No 2FA means password security is critical. Enforce strong passwords. Consider password manager for family. Acknowledge that simplicity comes at the cost of advanced features. For home lab with 4 trusted users, this is acceptable.

## Alternatives Considered

### Authelia

Full-featured authentication and authorization server with comprehensive security features.

**Features:** 2FA (TOTP, WebAuthn), granular access control rules, user groups, LDAP integration, brute-force protection, extensive audit logging, password reset flows, session management with multiple storage backends.

**Why rejected for home lab:** Authelia is enterprise-grade, which is both its strength and weakness. For a home lab with 4 users, it's over-engineered. Configuration complexity is high—YAML files with domain-specific access control language. Resource usage is heavier (needs Redis or database for session storage). Setup and maintenance overhead outweighs the benefits for our scale.

The additional features (2FA, LDAP, complex ACLs) aren't requirements. They're nice-to-haves that add operational burden. For a family task manager, Tinyauth's simplicity matches the problem better.

Would reconsider if: we needed 2FA, had multiple user tiers with different permissions, or were protecting truly sensitive data (financial, medical records).

### Basic Authentication in Traefik

Traefik can enforce Basic Auth without an additional service. Add `basicAuth` middleware with bcrypt-hashed passwords.

**Why rejected:** Basic Auth sends credentials with every request. Even over HTTPS, this increases exposure. No 2FA. No brute-force protection beyond Traefik's rate limiting. No account lockout. No session concept—you're perpetually authenticated or perpetually prompted.

For a local network, acceptable. For public internet, insufficient. The credential stuffing risk alone makes it unsuitable. If any family member reuses a password (they will), one breach elsewhere compromises the task system.

### VPN-Only Access

Don't expose the task server to the internet at all. Require VPN connection (Tailscale, WireGuard) to access the local network. Then use Basic Auth or even no auth.

**Why rejected:** VPN adds friction. Installing Tailscale on every device. Remembering to connect before accessing tasks. Mobile access becomes cumbersome. Family members forget, get frustrated, stop using the system.

The cognitive overhead is real. "Check task list" becomes "Enable VPN → wait for connection → open browser → access tasks." That's three steps instead of one. For quick interactions ("What's next?"), this kills adoption.

VPN is excellent for sensitive infrastructure (SSH access, database admin tools). For a family task board? Overkill that creates more problems than it solves.

### OAuth2 Proxy with GitHub/Google

Use `oauth2-proxy` container. Family members log in with their existing GitHub or Google accounts. No password management. Instant 2FA if they already use it.

**Why rejected:** External dependency. If GitHub has an outage, nobody can log in to see their tasks. Privacy concern: GitHub/Google know when family members access the task system (OAuth callbacks are tracked).

More importantly: it's weird. "Log in with GitHub to see the shopping list?" The mismatch between authentication provider and application domain creates cognitive dissonance. Users expect task management to have task management credentials, not tech company accounts.

Good for developer tools. Strange for family life infrastructure.

### Self-Implemented Authentication in Axum

Build session management, password hashing, and brute-force protection directly into the Axum application. Full control, no external dependencies.

**Why rejected:** This is exactly the complexity we want to avoid. Session management has subtle bugs (fixation attacks, timing vulnerabilities, secure token generation). Even basic authentication has implementation pitfalls.

Security code is not our core competency. We're building task management, not an authentication system. Every line of auth code is maintenance burden, potential vulnerability, and distraction from actual features.

The effort to build secure authentication correctly would exceed the effort to deploy Tinyauth by 10x. And our implementation would be less secure, less tested, and less maintained than even a simple dedicated auth service.

### Tailscale Funnel

Tailscale's "Funnel" feature exposes a local service to the internet while requiring Tailscale authentication. Hybrid approach: public endpoint, private authentication.

**Why rejected:** This is actually close to what we want. But it locks us into Tailscale's ecosystem. If we later want to switch VPN providers or authentication methods, we're stuck. It also requires Tailscale on the server and all client devices.

For teams already using Tailscale everywhere, this makes sense. For a home lab starting fresh, it's premature vendor lock-in. Tinyauth + Traefik is portable. We can move providers, change VPNs, or add services without redesigning authentication.

## Implementation Notes

Tinyauth solves a specific problem: protecting applications that weren't designed for hostile environments. Our task server was built for trust. Tinyauth provides the suspicion layer without contaminating the trust layer.

The separation is clean. Authentication lives in Tinyauth and Traefik. Task logic lives in Axum. They communicate through standard HTTP headers. No tight coupling. No shared state. No mixed concerns.

This is the right architecture for public internet deployment. The task server never needed authentication features. Adding them now would be scope creep. Tinyauth lets us stay focused: tasks are tasks, security is security.

For four family members in a home lab, Tinyauth is appropriately sized. It doesn't bring enterprise features we don't need. It doesn't require operational complexity we can't sustain. The security posture matches the actual threat: protect against casual attacks, trust the authenticated users.

We're not under-engineering. We're matching the solution to the actual context—home infrastructure, trusted users, pragmatic security. Tinyauth fits that context perfectly.

## Deployment Example

This example demonstrates secure public internet deployment using Podman Quadlet and Traefik v3 with dynamic Docker labels.

### Architecture Overview

```
Internet
   │
   │ HTTPS (Let's Encrypt)
   ▼
┌─────────────────────────────┐
│ Traefik v3                  │
│ (Podman/Systemd)            │
└──────────┬──────────────────┘
           │
           ├─→ auth.example.com
           │   (Tinyauth login page)
           │
           └─→ tasks.example.com
               (Protected by ForwardAuth)
                   ↓
           ┌──────────────────┐
           │ kid-server       │
           │ (Internal only)  │
           └──────────────────┘
```

### Key Configuration Points

**Tinyauth middleware:** Define ForwardAuth middleware in Tinyauth's Traefik labels. Protected services reference this middleware to enforce authentication.

**Network isolation:** Containers share a common network (e.g., `traefik`). The task-server has no public ports exposed—only accessible via Traefik after authentication.

**User management:** Users are defined via environment variables with bcrypt-hashed passwords. Generate hashes using Tinyauth's built-in command.

**Selective Protection:**

Protected services include the authentication middleware in their Traefik configuration. Public services omit the middleware reference. This allows mixing protected and public endpoints within the same infrastructure.

Example: A task management interface at `tasks.example.com` requires authentication, while a public homepage at `www.example.com` remains openly accessible.

This demonstrates that the task management system can be securely deployed on public internet infrastructure with minimal operational overhead—appropriate for home lab environments while maintaining production-grade security for protected services.
