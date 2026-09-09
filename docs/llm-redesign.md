# LLM workspace implementation audit

> Historical implementation record. Current OAuth behavior, interruption audit,
> and empty-configuration acceptance are documented in [oauth-login-audit.md](oauth-login-audit.md).
> Earlier requirements to configure a backend before login/import no longer apply.

Source of requirements: user-provided `image-1.png`, September 8, 2026.

## Acceptance requirements

- Three distinct modes: OAuth subscription, API subscription, local open weights.
- OAuth provider selector: ChatGPT OAuth and Anthropic OAuth; real KPI cards.
- OAuth account file refresh, edit, upload and download, with icon actions.
- Per-account model editor: disabled models, aliases and default reasoning effort.
- Model settings must affect proxy requests and survive restart.
- API provider selector: ChatGPT API and Anthropic API.
- API KPIs: requests, total tokens, input/output tokens, cached input,
  reasoning tokens and cost/spend.
- Local KPIs: TTFT, tokens/second, requests/second and context.
- Frontend and backend must work together; no invented observations.
- Check error, empty, loading and small-screen states, persistence and secrets.

## Evidence and execution log

1. Inspected screenshot and current checkout; both repositories started clean.
2. Traced `ui/ui.html::renderLlm`, `crates/ui/src/lib.rs`,
   `crates/proxy/src/server/{mod,llm_flow}.rs`, `state.rs` and core backend types.
   Existing UI contains example accounts and hard-coded KPI fallbacks; current
   UI server has no OAuth account CRUD endpoints. Consequently a visual-only
   replacement cannot satisfy the screenshot.
3. Inspected CLIProxyAPI `internal/auth/{codex,claude}/token.go` and refresh
   implementations. Import/export should retain the original credential JSON;
   model rules and backend binding belong to separate gateway metadata.
4. Added `crates/proxy/src/accounts.rs`: preserve imported provider JSON separately
   from backend/model metadata; validate IDs and rules; atomically persist files
   with revision conflict detection. Refresh uses each provider's token endpoint
   and preserves a refresh token when the response omits its replacement.
5. Added authenticated management routes in `crates/ui/src/llm.rs` for read,
   save, download and refresh. `/debug/llm` only returns redacted summaries.
   HTTP tests prove authentication, conflict handling, export and redaction.
6. Connected account selection and model rules to proxy forwarding. Codex uses
   the Responses protocol; Claude uses Anthropic messages and thinking settings.
   HTTP integration tests inspect actual upstream bodies and authorization,
   including aliases, disabled models and caller/default reasoning precedence.
7. Extended usage accounting for cached input, cache creation, reasoning and
   price estimates. Added stream-content timing in `llm_timing.rs`; unit tests
   distinguish first generated content from metadata and cancelled streams.
8. Replaced the old LLM renderer with `ui/llm.js` and `ui/llm.css`: three modes,
   provider selectors, KPI cards, five account icon actions and model dialogs.
   Management tokens remain in browser memory. Upload/edit require an explicit
   save, and download exports the original provider document.
9. Updated the fixture references to `tests/ui-fake.json`; the source scan finds
   no remaining old fixture filename references. Added `tests/llm-workspace.cjs`
   using an actual gateway process, browser and deterministic HTTP upstream.
10. Verified the current implementation on September 8, 2026:
    - `cargo test --workspace`: 212 passed, 2 ignored, 21 suites.
    - `cargo build --bin dxgate`: passed.
    - `node tests/llm-workspace.cjs`: passed upload/edit/download/model rules,
      real forwarding, restart persistence, API usage/cost and local timings.
    - `node tests/ui-smoke.cjs`: passed all eight pages, provider filtering,
      empty/error/recovery states, paused polling, keyboard behavior, light/dark
      themes and 1440/1024/768/390px widths with no JavaScript errors.
    - `git diff --check`: passed; CLIProxyAPI working tree remains clean.
    The two ignored tests are existing machine-dependent throughput/latency
    benchmarks, not skipped LLM acceptance tests. No live provider credentials
    were used; refresh wire formats and token rotation use local HTTP tests.
11. Cleaned build/test artifacts after verification: `cargo clean` removed
    33,039 files (11.9 GiB); confirmed `target`, `debug`, temporary screenshots
    and the accidental condense training log are absent. Stopped the remaining
    dxgate process after verifying its working directory was this repository.
    Source files, `Cargo.lock`, generated source, UI assets and user `AGENTS.md`
    remain. Rebuilding is necessary before running the browser integration again.

## Requirement-by-requirement audit

| Screenshot requirement | Implementation and evidence |
| --- | --- |
| Three modes and OAuth/API provider selectors | `renderLlmWorkspace`; browser mode/filter tests |
| OAuth KPI cards | Redacted `/debug/llm` aggregates; actual account count and process usage |
| Refresh icon | Authenticated refresh route; both provider refresh HTTP tests and persisted rotation |
| Edit/upload/download icons | Actual browser upload/edit/download and restart persistence test |
| Disabled model, alias, default reasoning | Account validation/selection and upstream request integration tests |
| API requests/tokens/input/output/cache/reasoning/spend | Actual upstream usage fixture asserted against `/debug/llm` |
| Local TTFT/tok/s/req/s/context | Delayed content stream and measured dashboard assertions; cancellation tests |
| Frontend/backend integration | Real compiled gateway browser test, not solely a mocked UI server |

## Enable account management

Configure an LLM backend with `account_type: "subscription"` (or `"oauth"`),
the matching provider kind (`open-ai`/`open-ai-compatible` for Codex or
`anthropic` for Claude), and a route to that backend. Use the provider kind
spelling already used by your runtime configuration; see `tests/ui-fake.json`.
Set `XGATE_LLM_ADMIN_TOKEN` (or `DXGATE_LLM_ADMIN_TOKEN`) to your management token, then start the gateway
with `--llm-accounts-dir=/absolute/private/accounts` and your normal static
configuration or control-plane flags. Bind the management UI appropriately for
your environment. The UI prompts for this token before managing credentials.

Import a CLIProxyAPI Codex/Claude OAuth JSON document, select the configured
subscription backend and save. Model edits persist alongside the document;
downloads retain the provider file format. API/local backends continue to use
gateway configuration, with `account_type` of `api-key`/`self-hosted` respectively.
`tests/llm-workspace.cjs` contains a runnable minimal configuration example.

## Risks requiring explicit verification

- OAuth tokens use provider-specific refresh and request protocols; a bearer
  header alone is insufficient for the ChatGPT Codex endpoint.
- Credential endpoints must not expose tokens through account lists, errors or
  unauthenticated management requests. Refresh must not overwrite concurrent edits.
- Aliases and disabled models must participate in selection as well as rewriting.
- Cached/reasoning tokens are subsets of input/output and must not be double-counted.
- Response-header latency is not TTFT; token rate needs actual stream observations.
- An absent upstream usage, quota, price or context limit is unknown, not zero.
- Configured API backends, OAuth accounts and local usage logs must not be mixed
  into the same KPI totals.
- CLIProxyAPI is a reference only; its source and local credentials remain unchanged.

## Remaining operational limits

- Real provider account eligibility and future upstream protocol changes cannot
  be proven by deterministic tests. The file workflow and browser OAuth login
  cover Codex and Claude, not CLIProxyAPI's entire provider catalog.
- Refresh is an explicit account action. No background refresh scheduler is added.
- Metrics cover this gateway process since startup; they are not provider billing
  history. Estimated spend excludes unpriced requests. Cached/reasoning subsets
  must not be added a second time to total tokens. Anthropic reasoning breakdown
  is unavailable and displayed as `—`.
- Local context is explicitly labeled **observed peak**, derived from reported
  input plus output; it is not a hardware/model context capacity declaration.
  Missing timing observations remain `—`. Quotas are shown only when configured
  or reported; refresh-token responses do not establish subscription quota.
- Management files contain credentials (directory mode 0700, file mode 0600).
  Redacted lists and authenticated routes do not replace transport/access controls
  for a UI exposed beyond localhost.

## OAuth login addition

The **OAuth 登录** button sits immediately left of **上传 OAuth 文件**. It opens
a provider-specific panel with a start button, copy/open authorization link,
callback URL field and authentication status. It follows the selected ChatGPT
or Anthropic OAuth view and saves into a matching configured subscription backend.

1. Set up management storage/token as above; select a subscription provider.
2. Click OAuth login to open the provider panel directly. If management is not
   configured, the panel explains the missing settings instead of requesting an
   undefined token. Otherwise enter the gateway management token in that panel,
   select the backend, then start login and open the generated link.
3. After provider authorization redirects to localhost, copy the entire address
   into the callback field, even if the localhost page cannot connect. This is
   the screenshot's manual callback flow; no local callback listener is required.
4. Submit the URL. The server exchanges the code and persists the authentication
   file; the panel reports the saved account and refreshes the account list.

Implementation evidence: `oauth_login.rs` follows the authorize/token constants
and request formats in CLIProxyAPI's `internal/auth/codex/openai_auth.go` and
`internal/auth/claude/anthropic_auth.go`. PKCE uses OS randomness and SHA-256.
The verifier stays server-side; sessions expire after ten minutes and code
exchange consumes the session to reject replay. Both routes require management
authentication and validate provider/backend binding. Callback destinations,
state and duplicate query parameters are checked before token exchange.

Verification: proxy library 75 tests and UI service 9 tests passed, including
local HTTP token exchanges for both providers, PKCE challenge matching, persisted
credentials after restart, redacted results, bad state, replay and expiry.
Browser tests exercise the real authorization-start route and wrong-state
callback; successful panel rendering uses a simulated exchange response while
the server exchange is tested against the local HTTP provider. Live provider
login still requires the user's account. Restart login after a token exchange
failure; provider authorization codes cannot safely be reused.

Final regression after the login addition: `cargo test --workspace` passed
214 tests with the two existing performance benchmarks ignored. Both browser
scripts passed. The login panel was visually inspected at desktop and 390px
widths and in both light and dark themes.

For the local static-config preview, enable management before starting:

```sh
export XGATE_LLM_ACCOUNTS_DIR="$HOME/.config/xgate/accounts"
export XGATE_LLM_ADMIN_TOKEN="$(openssl rand -hex 32)"
# Copy this value into the gateway management-token field; keep it private.
printf '%s\n' "$XGATE_LLM_ADMIN_TOKEN"
cargo run --bin xgate -- --http-addr 127.0.0.1:8080 --ui-addr 127.0.0.1:15021 --static-config tests/ui-fake.json --xds-enabled false
```

This token protects the gateway's credential-management API. It is separate from
the provider OAuth login; the provider authorization link is generated only after
gateway management authentication succeeds. Browser regression covers missing
configuration, empty/invalid tokens, inline retry and successful link display.
