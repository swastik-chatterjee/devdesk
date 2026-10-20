# Contributing

Thanks for helping improve Devdesk. Contributions should be focused, clearly explained, and respectful of the project's privacy-first approach.

## Before you start

- Check existing issues and pull requests to avoid duplicated work.
- For substantial changes, open an issue first to discuss the approach.
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md).
- Never include access tokens, credentials, private repository data, or personal information in commits, tests, screenshots, or logs.

## Development setup

Install Node.js 20+, npm, Rust stable, and the platform-specific [Tauri prerequisites](https://tauri.app/start/prerequisites/). Then run `npm install` and `npm run tauri dev`.

Before submitting, run `npm run build`. For Rust changes, run `cargo check --manifest-path src-tauri/Cargo.toml` on a machine with the required Tauri system libraries.

## Security-sensitive changes

GitHub authentication and all authenticated API requests belong in Rust commands. Do not return credentials to Vue, serialize them into app configuration, write them to logs, or place them in command-line arguments. Store secrets with the OS credential store and add tests that do not require real credentials. Report vulnerabilities privately to the maintainers rather than opening a public issue with exploit details.

## Pull requests

Use a clear title and describe the behavior changed, the reason, and how it was verified. Include screenshots for meaningful UI changes at desktop and mobile widths. Keep unrelated formatting and generated files out of the diff. All checks in GitHub Actions must pass before merge.