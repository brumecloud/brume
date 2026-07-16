# Architecture

The Rust CLI embeds the React and MDX renderer.

| Component | Responsibility |
| --- | --- |
| CLI | Build and upload plans |
| Server | Authenticate, store, and serve plans |
| MCP | Expose plan operations to agents |

<Decision status="accepted">
Renderer assets are built from this monorepo and embedded in Brume binaries.
</Decision>

