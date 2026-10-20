# Third-party notices

Devdesk uses the following direct dependencies. Their individual licenses remain applicable; consult package metadata and upstream repositories for full terms.

| Dependency | Purpose | License |
| --- | --- | --- |
| Vue | UI framework | MIT |
| Vite and `@vitejs/plugin-vue` | Frontend build tooling | MIT |
| TypeScript and `vue-tsc` | Type checking | Apache-2.0 |
| Tauri and `@tauri-apps/api` | Desktop runtime and frontend bridge | Apache-2.0 OR MIT |
| `lucide-vue-next` | Interface icons | ISC |
| `reqwest` | Rust HTTP client | MIT OR Apache-2.0 |
| `keyring` | Operating system credential storage | MIT OR Apache-2.0 |
| `serde` and `serde_json` | Serialization | MIT OR Apache-2.0 |
| `tokio` | Async runtime | MIT |
| `uuid` | Profile identifiers | Apache-2.0 OR MIT |

The application also loads DM Sans, DM Mono, and Space Grotesk from Google Fonts at runtime. Their font files are distributed under the SIL Open Font License. A network connection is required to load these fonts; system fallbacks are used if unavailable.

Transitive dependencies are not exhaustively listed here. Their license information is available from `package-lock.json` and `src-tauri/Cargo.lock` after dependency installation/build.