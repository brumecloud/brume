# Brume vs Kamal

Checked: 2026-11-01. Official sources only (kamal-deploy.org, github.com/basecamp/kamal and kamal-site, rubygems.org).

**What it is:** Kamal is a free, MIT-licensed Ruby CLI from 37signals that deploys Docker containers to your servers over SSH from your own machine or CI. The only thing it leaves running on each server is a small proxy (kamal-proxy) that handles zero-downtime switches.

## Facts

| Fact | Value | Source URL | Short quote (under 20 words) |
|---|---|---|---|
| License | MIT | https://github.com/basecamp/kamal | "Kamal is released under the MIT License." |
| Pricing | Free. No paid tier, pro version or hosted plan found. | https://github.com/basecamp/kamal | "Kamal is released under the MIT License." |
| Install method | Ruby gem on the developer machine (`gem install kamal`), or run it as a Docker container. Then `kamal init` and edit `config/deploy.yml`. | https://kamal-deploy.org/docs/installation/ | "If you have a Ruby environment available, you can install Kamal globally with: gem install kamal" |
| Server requirements | A plain Linux host reachable over SSH as root by default. Kamal installs Docker with get.docker.com if it is missing. | https://kamal-deploy.org/docs/installation/ | "Install Docker on any server that might be missing it (using get.docker.com): root access is needed via SSH" |
| Server prep | No prep beyond an SSH key | https://kamal-deploy.org/ | "vanilla Ubuntu servers that have seen no prep beyond an added SSH key" |
| Registry requirement | Needs a container registry (Docker Hub by default). A `localhost:` registry makes Kamal run a local registry instead. | https://kamal-deploy.org/docs/configuration/docker-registry/ | "If the registry server starts with `localhost`, Kamal will start a local Docker registry" |
| Admin interface | CLI only. No web UI or dashboard in the docs (derived: none of the 65 doc pages describes one). | https://kamal-deploy.org/ | "You can see everything that's going on, it's just basic Docker commands being called." |
| Web UI port / exposure / 2FA / SSO | Not applicable (no web UI) | https://kamal-deploy.org/docs/commands/view-all-commands/ | not applicable |
| How it reaches Docker | Over SSH: SSHKit runs `docker` commands on each host, as root by default. | https://github.com/basecamp/kamal | "Works seamlessly across multiple servers, using SSHKit to execute commands." |
| SSH defaults | root user, port 22 (both configurable). Jump-host proxy supported. | https://kamal-deploy.org/docs/configuration/ssh/ | "By default, it will attempt to connect to the root user on port 22." |
| Docker socket on the server | kamal-proxy's `docker run` mounts only its config and apps-config directories, not `docker.sock` (derived from reading the source). | https://github.com/basecamp/kamal/blob/main/lib/kamal/commands/proxy.rb | `"--volume", "kamal-proxy-config:/home/kamal-proxy/.config/kamal-proxy"` |
| What an attacker gets by compromising the control interface | **Derived.** The control interface is the deployer's machine or CI: its SSH key (root by default), registry credentials and `.kamal/secrets`. Stealing those gives a root shell on every server in `deploy.yml` plus push access to the registry. The server runs no panel or API to attack, and kamal-proxy has no Docker socket. | https://kamal-deploy.org/docs/configuration/ssh/ | "Kamal uses SSH to connect and run commands on your hosts." |
| Inbound ports by default | SSH (22, open to wherever you deploy from), plus 80 and 443 for kamal-proxy. kamal-proxy publishes them by default; `publish: false` turns that off. | https://kamal-deploy.org/docs/configuration/proxy/ | "It runs on ports 80 and 443 and forwards requests to the application container." |
| Proxy publish option | `publish` defaults to true. `bind_ips` can limit which interfaces it binds. | https://kamal-deploy.org/docs/configuration/proxy/ | "publish: false # Publish ports to the host (default: true)" |
| Cloudflare Tunnel support | Not found for app traffic. The SSH docs only name Cloudflare Access for Infrastructure as an SSH tunnel (agent-forwarding note). | https://kamal-deploy.org/docs/configuration/ssh/ | "tunnel that does not support agent forwarding (for example, Cloudflare Access for Infrastructure with SSH)" |
| Let's Encrypt limitation | Automatic HTTPS needs a single server with `host` set. Multi-host setups need custom certificates or a load balancer. | https://kamal-deploy.org/docs/configuration/proxy/ | "This requires that we are deploying to one server and the host option is set." |
| Multi-server support | Yes: a list of servers, roles (web and workers), tags. With more than one web server you add your own load balancer. | https://kamal-deploy.org/docs/installation/ | "If you're running multiple servers, you need to put a load balancer in front of them." |
| AI agents: official CLI | Yes. Kamal is itself a CLI. | https://kamal-deploy.org/docs/commands/view-all-commands/ | not applicable |
| AI agents: official MCP server | Not found. No MCP in the Kamal docs, and no Kamal MCP repo in the basecamp GitHub org (`basecamp/mcp` covers Basecamp/HEY/Fizzy). | https://github.com/basecamp | not found |
| Database backups (to S3?) | Not found. Databases run as "accessories", with no built-in backup command. | https://kamal-deploy.org/docs/configuration/accessories/ | "they are not updated when you deploy, and they do not have zero-downtime deployments." |
| Templates / plugin ecosystem | No one-click templates. Extension points: accessories (any Docker image), 9 hook types, and secrets adapters (1Password, LastPass, Bitwarden, Bitwarden Secrets Manager, AWS Secrets Manager, Doppler, GCP Secret Manager, Passbolt). | https://kamal-deploy.org/docs/hooks/overview/ | not applicable |
| Secrets managers | 8 built-in adapters: 1Password, LastPass, Bitwarden, Bitwarden Secrets Manager, AWS Secrets Manager, Doppler, GCP Secret Manager, Passbolt | https://kamal-deploy.org/docs/commands/secrets/ | "Use these to read secrets from common password managers (currently 1Password, LastPass, and Bitwarden)." |
| Maturity: first release | 2023. Published as MRSK 0.0.1 on RubyGems on 2023-01-07, renamed Kamal (0.16.0) on 2023-08-22. Latest is v2.12.0 (2026-06-18). | https://rubygems.org/gems/mrsk | "Kamal was formerly known as MRSK." (kamal-deploy.org) |
| Maturity: GitHub stars | basecamp/kamal: 14,698. kamal-proxy: 1,115. (GitHub API, 2026-11-01) | https://github.com/basecamp/kamal | not applicable |
| Production use | 37signals runs HEY and its other apps on it | https://kamal-deploy.org/ | "this is the toolkit we've used at 37signals to bring HEY and all our other formerly cloud-hosted applications home" |

## Where it is better than a young single-server alpha

1. **Multi-server from day one.** One `deploy.yml` deploys to many hosts with roles (web, workers) and tags, across clouds or bare metal. Brume 0.1 is single-server.
2. **Proven in production at scale.** 37signals uses it to run HEY and its other apps on its own hardware. It has been public since January 2023 (as MRSK), is at v2.12, and has about 14.7k GitHub stars.
3. **Small resident footprint.** Nothing runs on the server except kamal-proxy, which has no Docker socket. There is no admin panel to expose, patch or secure.
4. **Deployment ergonomics.** Zero-downtime deploys, rolling restarts, rollback, maintenance mode, remote builds, 9 hook types, and secrets fetched from 8 vaults (1Password, Bitwarden, AWS, GCP, Doppler and others).
5. **Free and MIT-licensed**, with no paid tier.

## Suggested verdict

**Brume vs Kamal:** pick Kamal for a battle-tested, panel-free CLI that pushes from your laptop or CI to many servers over SSH. Pick Brume for push-to-deploy on one server with a web panel, no inbound ports at all (no public SSH, 80 or 443), and a Docker socket the UI never holds.
