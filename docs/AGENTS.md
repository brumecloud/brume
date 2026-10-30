# Brume docs

The public documentation of Brume: a static [Fumadocs](https://fumadocs.dev) site on TanStack Start. These rules are for anyone writing the docs, human or agent.

## Commands

Run from `docs/`. The pnpm version is pinned by `packageManager`.

| Command | What it does |
|---|---|
| `pnpm install` | Installs dependencies. New versions wait 7 days, and no dependency runs install scripts (`pnpm-workspace.yaml`). |
| `pnpm dev` | Dev server on http://localhost:3000. |
| `pnpm build` | Static build in `dist/client`. **Fails on any internal link to a missing page**: the prerenderer crawls every link. |
| `pnpm types:check` | TypeScript check. |

Set `VITE_SITE_URL` (for example `https://brume.dev`) for a production build: it is used in `llms.txt` and in agent prompts.

## Layout

```
docs/
├── content/docs/           pages (MDX) and sidebar order (meta.json)
│   ├── index.mdx           docs home, a landing page
│   ├── use/                Use Brume: install, deploy, operate
│   ├── how-it-works/       How Brume works: architecture, security, decisions
│   └── customize/          Customize Brume: build, change the code, these docs
├── src/
│   ├── routes/             pages, Markdown twins (.md), llms.txt, search index
│   ├── components/         MDX components
│   └── lib/                content source, frontmatter schema, site config
└── public/                 static files
```

Each folder of `content/docs/` is a sidebar tab (`"root": true` in its `meta.json`). `meta.json` lists the pages in order; `---Title---` starts a sidebar group. A page missing from `pages` is hidden.

## Where a page goes

| Section | For | Answers |
|---|---|---|
| `use/` | People running Brume | How do I install it, deploy, and operate my apps? |
| `how-it-works/` | People evaluating or auditing Brume | How is it built, and why is it safe? |
| `customize/` | People changing Brume | How do I build it, change it, and document it? |

## Frontmatter

Every page starts with this block. The build rejects a page that does not match the schema in `src/lib/source.ts`.

```yaml
---
title: Networking
description: How traffic reaches your apps without any inbound port.
type: concept
status: planned
sources:
  - archi/base/03-reseau.md
---
```

- **`title`:** short, sentence case. It is the sidebar label and the `<h1>`: never repeat it in the body.
- **`description`:** one sentence. It appears under the title, in search results, and in `llms.txt`, where agents use it to pick pages.
- **`type`:** one of the kinds below.
- **`status`:** `planned` while the page describes something not built yet. The page then shows a warning, and its Markdown twin says so. Switch to `available` once the page matches a release.
- **`sources`:** repository paths the page is derived from. When one of them changes, the page must be checked (see [Keeping the docs true](#keeping-the-docs-true)).

## Page types

Pick one type per page and stick to it. Mixing them is the most common reason a page is hard to use.

| `type` | Purpose | Shape |
|---|---|---|
| `overview` | Orient the reader in a section | Short. What it is, who it is for, cards to the next pages. |
| `tutorial` | Teach by doing, from zero to a working result | Numbered `<Steps>`. One path, no options. Ends with what the reader built and where to go next. |
| `guide` | Solve one task for someone who knows the basics | Title is the task ("Add a custom domain"). Prerequisites, steps, how to check it worked. |
| `concept` | Explain how something works and why | Prose and diagrams. Decisions and their reasons, trade-offs, what is out of scope. |
| `reference` | Describe every option, flag or field | Tables, one entry per item, no narrative. Generated from code when the code exists. |

## Writing

- **English,** US spelling. The design notes in `archi/base/` are in French: translate the meaning, not the words.
- **Lead with what matters.** The first sentence of a page or section says the most important thing.
- **Short sentences, plain words.** Address the reader as "you". Present tense: "Brume runs each app in its own network", not "will run".
- **Bold lead-ins for rule lists:** `- **No inbound port.** Public traffic goes through…`. Keep each item to one or two sentences.
- **Say why.** A security rule without its reason gets removed by the next person who finds it inconvenient.
- **No marketing words:** no "seamless", "powerful", "blazing fast", "simply".
- **Code blocks** always have a language, and a `title` when they show a file: ` ```toml title="brume.toml" `.
- **Headings** are sentence case, `##` and `###` only. They feed the table of contents.

## Facts

- **Never invent.** Every command, flag, path, port and behavior must come from the code or from `archi/base/`. If neither says it, leave it out.
- **Planned is not shipped.** Until a page is `available`, describe the design. Do not write commands as if they had been run, and do not make up their output.
- **Open questions stay open.** If the design lists a decision as open, say it is open and link to it. Do not pick an answer.

## Links

- **Internal links are absolute:** `/docs/how-it-works/networking#cloudflare-tunnel`. The build fails on a link to a missing page.
- **Link to a decision, do not repeat it.** One page owns each topic, the others link to it.
- **External links** go to official sources (project docs, RFCs, advisories).

## Components

Available in every page without import (`src/components/mdx.tsx`):

| Component | Use |
|---|---|
| `<Callout type="info \| warn \| error" title="…">` | A warning or a note the reader must not miss. At most two per page. |
| `<Cards>` / `<Card title href>` | Links to the next pages, at the end of overviews. |
| `<Steps>` / `<Step>` | Numbered procedures in tutorials and guides. Each step starts with a `###` heading. |
| `<Tabs items={[…]}>` / `<Tab value="…">` | Alternatives the reader picks one of (OS, package manager). |
| `<Files>` / `<Folder name>` / `<File name>` | A file tree. |
| `<Accordions>` / `<Accordion title>` | FAQs and long optional details. |
| `<TypeTable type={{…}}>` | Fields of a config object, in reference pages. |
| `<Hero>`, `<AgentPrompt>`, `<Features>`, `<Feature>` | Landing pages only. |

Icons come from `lucide-react`: `import { Rocket } from "lucide-react";` after the frontmatter.

In MDX, `{`, `}` and `<` in prose start code: put them in backticks.

## For agents reading the docs

The build generates, from the pages and their frontmatter:

- `/llms.txt`: an index of every page with its description.
- `/llms-full.txt`: every page in one file.
- `/docs/<page>.md`: the Markdown twin of each page, linked from the "Copy Markdown" and "Open" buttons.

So a good `description` and a correct `status` matter as much as the body.

## Keeping the docs true

- **A change to `archi/base/` updates the docs in the same PR.** Find the pages to check with `grep -rl "archi/base/<file>" docs/content`.
- **A change to a command, flag or config field updates its reference page.** Once the code exists, reference pages are generated from it (clap for the CLI, the Rust types for the config) and the build checks they are current.
- **Do not edit generated pages by hand.**

## Security

The docs site publishes the install command. A compromised docs site can hand out a malicious command, so it follows the same rules as the panel (`archi/base/10-developpement.md#frontend`):

- **Static site only.** Nothing runs on a server.
- **No third-party scripts:** no analytics, chat widget, external search, external fonts or embeds. Search runs in the browser on a prerendered index.
- **New dependencies:** well-known packages only, added with pnpm so the 7-day delay and the install-script ban apply.
- **The install command** always points to GitHub Releases, never to a URL served by the docs.

## Before you finish

1. `pnpm build` passes.
2. Each new page has complete frontmatter and is listed in its `meta.json`.
3. Every fact comes from the code or `archi/base/`, and every source is in `sources`.
4. Read the page once as its reader: does the first paragraph tell them what they need?
