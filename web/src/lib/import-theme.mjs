import { parse } from 'smol-toml';
import { validateTheme } from './themes.mjs';
export function importTheme(document) {
  if (new TextEncoder().encode(document).length > 65536)
    throw new Error('Theme files must be 64 KiB or smaller.');
  let fence = null,
    active = false,
    blocks = [],
    lines = [];
  for (const line of document.replace(/^\uFEFF/, '').split(/\r?\n/)) {
    if (fence) {
      const close = line.match(/^ {0,3}(`{3,}|~{3,})\s*$/);
      if (
        close &&
        close[1][0] === fence[0] &&
        close[1].length >= fence.length
      ) {
        if (active) blocks.push(lines.join('\n'));
        fence = null;
        active = false;
        lines = [];
      } else if (active) lines.push(line);
    } else {
      const open = line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
      if (open) {
        fence = open[1];
        active = open[2].trim() === 'moodsh';
      }
    }
  }
  if (active || blocks.length !== 1)
    throw new Error(
      'Use exactly one complete fenced moodsh block. Export a theme with moodsh theme export my-theme.md.',
    );
  let parsed;
  try {
    parsed = parse(blocks[0]);
  } catch {
    throw new Error(
      'The moodsh block contains invalid TOML. Check its field names and quoted values.',
    );
  }
  if (
    Object.keys(parsed).join() !== 'mood' ||
    !parsed.mood ||
    typeof parsed.mood !== 'object'
  )
    throw new Error('The theme must contain a [mood] table.');
  const { mood } = parsed;
  if (
    Object.keys(mood).sort().join() !== 'layout,name,palette' ||
    !mood.palette ||
    typeof mood.palette !== 'object' ||
    Object.keys(mood.palette).sort().join() !== 'accent,error,muted,path'
  )
    throw new Error(
      'Include only name, layout, and the four palette colors in the moodsh block.',
    );
  const theme = {
    name: mood.name,
    layout: mood.layout,
    colors: ['accent', 'path', 'muted', 'error'].map(
      (role) => mood.palette[role],
    ),
  };
  if (
    typeof theme.name !== 'string' ||
    !theme.colors.every((color) => typeof color === 'string')
  )
    throw new Error('Theme names and colors must be text values.');
  return validateTheme(theme);
}
