# Mood Shell theme documents

A theme is a UTF-8 Markdown file containing exactly one standalone fenced block
whose info string is `moodsh`. Everything outside that block is documentation.
The block contains TOML using the existing `[mood]` and `[mood.palette]` schema.

Start with `moodsh theme new my-mood.md --from dusk`, or export your saved picker
colors with `moodsh theme export my-mood.md --name my-mood`. Both commands create
new `.md` or `.markdown` files and refuse to overwrite existing files.
The output directory must already exist. Exporting does not rename your active mood.

## Fields

All fields are required. Unknown fields, invalid values, and duplicate TOML keys
are errors, so typos do not silently change your prompt.

| Field | Accepted values | Purpose |
| --- | --- | --- |
| `mood.name` | 1–48 ASCII letters, digits, hyphens, underscores | Name displayed in two-line layout |
| `mood.layout` | `compact` or `two-line` | Prompt arrangement |
| `mood.palette.accent` | `#RRGGBB` | Success arrow |
| `mood.palette.path` | `#RRGGBB` | Directory text |
| `mood.palette.muted` | `#RRGGBB` | Mood name |
| `mood.palette.error` | `#RRGGBB` | Failed-command status and arrow |

## Fences and file limits

Use at least three backticks or tildes followed by `moodsh`. The closing fence
must use the same character, at least as many characters, and no trailing text.
Both fences may have up to three leading spaces. Use standalone blocks, outside
lists, blockquotes, HTML comments, and other containers. Blocks inside a larger
fenced example are ignored. A second moodsh block is an error, even if identical.

LF and CRLF line endings and a UTF-8 byte-order mark are accepted. Documents are
limited to 64 KiB. This is a small fenced-block reader, not a general Markdown
renderer; it does not interpret HTML, frontmatter, links, or embedded commands.
Keep the theme block outside HTML comments: those comments are not parsed.

## Preview, apply, and share

```sh
moodsh theme preview --file my-mood.md
moodsh theme apply my-mood.md
```

Preview does not write any files. Apply validates the document and saves the
result as the normal TOML config. A malformed existing config is not overwritten.
The source document is never changed, and the shell hook never reads Markdown.
Reapply after editing a document, or customize the saved palette and export a new file.

Share the file through Git, a gist, or any file transfer. The receiver saves a
local copy before previewing or applying it. Mood Shell does not fetch URLs or
execute anything in a theme file. Markdown viewers may independently render
links or images; Mood Shell ignores them.

[Afterhours](afterhours.md) is a complete example. Theme documents customize the
prompt only. Terminal backgrounds, blinking cursors, syntax highlighting, and
autosuggestions belong to other tools or terminal settings.
