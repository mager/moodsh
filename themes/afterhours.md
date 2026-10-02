# Afterhours

Pink and lavender for a dark terminal, with a little room to breathe.
Made for Mood Shell by Andrew Mager. Released under the repository's MIT license.

The two-line layout gives the directory its own row. Lavender keeps the path
readable, pink marks a successful command, and red makes failures stand out.
Choose a dark background in your terminal settings; this file changes only the prompt.

```moodsh
[mood]
name = "afterhours"
layout = "two-line"

[mood.palette]
accent = "#FF77CC"
path = "#E0DEF4"
muted = "#908CAA"
error = "#EB6F92"
```

Preview with `moodsh theme preview --file themes/afterhours.md`.
Apply with `moodsh theme apply themes/afterhours.md`.
Edit the four colors to make it your own, then share this document.
