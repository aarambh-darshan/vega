# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.9.x   | ✅ Current         |
| < 0.9   | ❌ Not supported   |

## Reporting a Vulnerability

If you discover a security vulnerability in Vega, please report it responsibly:

1. **Do NOT** open a public GitHub issue
2. Email security concerns to the maintainers
3. Include a detailed description of the vulnerability
4. Include steps to reproduce if possible
5. Allow reasonable time for a fix before public disclosure

## Security Considerations

Vega handles HTTP requests and renders HTML. Key security areas:

- **SSR HTML injection**: All user input must be escaped before rendering. Use `vega::server::esc()` or your own escaping.
- **Cookie security**: Session cookies use `HttpOnly`, `SameSite=Lax`, and optionally `Secure` flags.
- **CORS**: Default is permissive. Configure restrictively for production.
- **Rate limiting**: Available via `vega::middleware::rate_limit()`.
- **Security headers**: HSTS, X-Frame-Options, X-Content-Type-Options are applied by the secure headers middleware.

## Best Practices for Vega Applications

1. Always escape user input in HTML output
2. Use `Secure` cookie flag in production (HTTPS)
3. Configure CORS to allow only your domain
4. Enable rate limiting on auth endpoints
5. Keep dependencies updated
6. Use environment variables for secrets, never hardcode them
