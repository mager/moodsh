# Mood Shell website

The public landing page, curated theme gallery, and browser theme maker at
[https://moodsh.vercel.app](https://moodsh.vercel.app). Built with Astro and static
HTML. Theme editing runs in the browser; no account, database, or server is
needed to preview, import, remix, or download a theme.

## Develop and verify

Use Node 22.19 or newer:

```sh
npm ci --prefix web
npm run dev --prefix web
npm test --prefix web
npm run build --prefix web
```

To verify every gallery download against the Rust parser:

```sh
cargo build --locked
node web/scripts/check-cli.mjs target/debug/moodsh
```

In development, append `?audit=1` to any page to run axe's WCAG A/AA checks.
The report appears as `MOODSH_A11Y` in the browser console. The auditor is
excluded from production. Check the landing page, gallery filters and empty
state, imports, color editing, downloads, and narrow mobile layouts before
shipping UI changes.

## Review a community submission

The theme maker opens a prefilled GitHub issue. It never posts automatically.
A GitHub account is required to submit; there are no website accounts or instant
public uploads. The contributor must confirm permission to share the palette
under MIT and provide a name or handle for credit.

1. Check the issue's credit and permission. Preview the Markdown with
   `moodsh theme preview --file submitted-theme.md`; never execute its prose.
2. Add the reviewed theme to `src/lib/themes.mjs` with a unique filename-safe
   name, four hex colors, a supported layout, short description, and author.
   Use `builtIn: false`. Preserve attribution when reviewing remixes.
3. Choose `tone` and `backdrop` for the web preview only. These are not exported
   or applied to the contributor's terminal.
4. Run the tests and build, then the CLI compatibility check above. Commit and
   push to `main` to publish the theme through Vercel.

Gallery Markdown files are generated at build time from the same catalog used
by the previews. Imported files are limited to 64 KiB, read as UTF-8, and parsed
as TOML from one fenced `moodsh` block. Their prose is never rendered as HTML or
executed. The maker imports palette settings only; credit and story are entered
separately.

## Deployment

Vercel project: `moodsh`, framework Astro, root directory `web`, Node 22.x.
GitHub `mager/moodsh` is connected; pushes to `main` deploy production.
No environment variables or secrets are required. For a manual production
release, run `vercel --prod` from the repository root after linking that project.

Update `release` in `src/lib/themes.mjs` when a new CLI release is available.
Website releases are independent of the Rust crate's Semantic Versioning.
