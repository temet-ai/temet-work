---
session_id:
  - 7bb2fb4e-ea63-41fd-bb34-aad02f0a2882
---

# Extending temet-work from outside the repository

This document records how someone outside this repository adds a tracker, a runner, an
agent runtime or a check, and why. Inside the repository an adapter is a crate
(`docs/architecture.md`).

## What was decided on 26 November 2026

Both transports from the start, not the recommendation below: trackers and the parsing half
of checks are WebAssembly components, with network hosts, paths, time and memory enforced
by the runtime; runners and agent runtimes are subprocesses speaking JSON-RPC lines over
stdio. One contract per family, the install preview, grants, `acts`, the conformance tests
and discovery are as recommended below. What this costs:

- two transports and two bridges to build and test now;
- a Python author cannot write a tracker until componentize-py supports WASI 0.3;
- Wasmtime is a heavy dependency, so it lives only in the crates that load Wasm
  extensions, each behind its own feature.

The sources were fetched on 26 November 2026. The page reader summarised each page through
a small model, so quotes here are its extracts, not raw page text.

## What other systems do

| System | Contract and versions | Permissions | Testing and install |
|---|---|---|---|
| [Zed extensions](https://zed.dev/docs/extensions/developing-extensions) | WIT files per API version (`since_v0.0.1` to `since_v0.8.0`, [source](https://github.com/zed-industries/zed/tree/main/crates/extension_api/wit)); code compiles to `wasm32-wasip2`. A language server extension returns a `Command` and Zed starts it ([languages](https://zed.dev/docs/extensions/languages)). | `process:exec`, `download_file`, `npm:install`, narrowed by the user in `granted_extension_capabilities` ([capabilities](https://zed.dev/docs/extensions/capabilities)). | A pull request to one registry repository, reviewed by Zed ([publishing](https://zed.dev/docs/extensions/publishing/publishing-guide)). |
| [WASI 0.3](https://bytecodealliance.org/articles/WASI-0.3) | Async functions, `stream<T>` and `future<T>` in the component model; announced 11 June 2026. Wasmtime 46 is slated to run it by default; jco supports it; Python bindings for async are still in progress ([componentize-py](https://github.com/bytecodealliance/componentize-py)). | Nothing is reachable unless the host grants it. | |
| [Extism](https://extism.org/docs/concepts/manifest) | A manifest names the Wasm file and its hash. | `allowed_hosts`, `allowed_paths`, memory limits and `timeout_ms`, enforced by the runtime. | No way to test a plugin found in its docs. |
| [Zellij plugins](https://zellij.dev/documentation/plugin-api-permissions) | | 17 permission kinds, requested in `load()` and granted by the user at a prompt. | |
| [Nushell plugins](https://www.nushell.sh/contributor-book/plugin_protocol_reference.html) | A subprocess over stdio, JSON or msgpack. Both sides send `Hello` with a semver version and a feature list; unknown features are ignored; new fields have defaults. | None: it runs as the user. | |
| [MCP 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/changelog) | No session or handshake: each request carries its version and capabilities in `_meta`; `server/discover` advertises versions and identity; results carry `resultType`; a deprecation window of at least 12 months. | None in the protocol. | |
| [Terraform](https://developer.hashicorp.com/terraform/plugin/terraform-plugin-protocol) | gRPC; a major version is a compatibility boundary, a minor one adds. A release declares `protocol_versions` in a registry manifest ([publishing](https://developer.hashicorp.com/terraform/registry/providers/publishing)). | None. | A registry picks a release the CLI can run. |
| ask-kit kits | A folder with `ask-kit.toml`; any language. | None: a kit "runs as you". Calls that change things are listed in `acts` and held back unless a change was asked for. | `ask-kit kit install owner/repo` previews what the kit reaches, changes and runs; a bench of real questions ships with the kit; a GitHub topic is the gallery. |
| Herdr mods | A folder with `mod.json`; the host rewrites the type files on every start. | Stated at install: network, programs, files outside the folder. | `herdr-app check <mod>` loads a mod without the app; an invalid file keeps the last good version and writes the reason to `<name>.error`. |

## Recommendation

Runners and agent runtimes start terminals and processes, and checks run commands, so a
WebAssembly sandbox would protect nothing for them; Zed's own language servers are started
by Zed, not by the Wasm code. A tracker is the one kind that is mostly network calls, and
the only kind where Wasm adds real enforcement. WASI 0.3 is ready for Rust and nearly
ready for TypeScript, and not yet for Python.

1. **One contract per family, independent of transport.** The family crate's Rust types
   are the source. The build writes `schema/<family>/v<major>.json` from them, and later
   WIT in Zed's per-version layout.
2. **One transport now:** a subprocess speaking stateless JSON-RPC lines over stdio, shaped
   after MCP 2026-07-28: version and capabilities in each request's `_meta`, a
   `discover` method, a features list, unknown features ignored, new fields defaulted.
   Each family has its own major version.
3. **WebAssembly later,** as a second transport for trackers and for the parsing half of
   checks (the host runs the declared command; the component reads its output), once
   Python can build WASI 0.3 components.

| Kind | Now | Later | Typical grants |
|---|---|---|---|
| Tracker | subprocess | Wasm component | network hosts, one secret |
| Runner | subprocess | subprocess | running the terminal program |
| Agent runtime | subprocess | subprocess | running the agent program, paths to the work folder |
| Check | subprocess | Wasm for parsing | running the declared command |

**Permissions.** The manifest declares network hosts, commands with argument patterns,
paths, and secrets by name; a secret is passed per call. Installing shows them, and the
user can narrow them in config. Calls that change something are listed in `acts` and held
back in read-only runs. A subprocess runs as the user: the host passes only the granted
secrets and environment and sets the folder, and confines it further only where the
operating system allows (Landlock on Linux). The docs say so plainly.

**The same contract tests.** A `temet-work-conformance` crate runs each family's contract
tests against any adapter: in process for adapters in this repository, and through the
subprocess bridge for an outside extension. `temet-work ext test ./my-ext` runs them
against recorded fixtures with no daemon and no live service, the way `herdr-app check`
works.

**Install and discovery.** `temet-work ext install owner/repo[/subdir]` shows the kinds,
grants, acts and the command it runs before it installs into
`~/.config/temet-work/ext/<id>/`. The GitHub topic `temet-work-ext` is the gallery. An
extension whose major version the host does not speak is refused, the last good version
stays, and the reason goes to `<id>.error`.

**What an author writes:**

```toml
# temet-ext.toml
id = "acme.linear"
version = "0.1.0"
kinds = { tracker = "1" }
protocol_versions = ["tracker/1.2"]
run = ["linear-tracker"]
[grants]
net = ["api.linear.app"]
secrets = ["LINEAR_TOKEN"]
acts = ["tracker.comment", "tracker.set_state"]
```

```json
→ {"jsonrpc":"2.0","id":7,"method":"tracker.list_issues","params":{"project":"ENG","_meta":{"protocol":"tracker/1.2"}}}
← {"jsonrpc":"2.0","id":7,"result":{"resultType":"complete","issues":[{"id":"ENG-41","title":"…","state":"open"}]}}
```

A Rust author implements the same trait as the adapters in this repository and calls
`serve`; TypeScript and Python authors get generated types and a `serve` helper.

**What it costs.** A subprocess tracker is not sandboxed until Wasm arrives: its grants are
shown at install and confined only as far as the operating system allows. Three thin
author kits (Rust, TypeScript, Python) have to follow the generated schemas.

## Still unknown

- Whether Wasmtime 46 shipped with WASI 0.3 on by default, and whether jco's default-on
  release is out.
- When componentize-py supports WASI 0.3 async.
- How to generate both WIT and JSON Schema from one set of Rust types; WIT may have to be
  the source instead.
- How Zellij stores grants, and how a host tests an Extism plugin.
- Confinement on macOS, which has no Landlock.
- Who builds and signs extension binaries.
