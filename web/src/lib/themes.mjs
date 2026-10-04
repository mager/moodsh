export const release = '0.8.0';
export const repo = 'https://github.com/mager/moodsh';
export const themes = [
  {
    name: 'dusk',
    title: 'Dusk',
    description: 'For the last good idea of the day.',
    tone: 'dark',
    backdrop: '#24232D',
    layout: 'compact',
    colors: ['#C4A7E7', '#E0DEF4', '#908CAA', '#EB6F92'],
    author: 'Mood Shell',
    builtIn: true,
  },
  {
    name: 'aurora',
    title: 'Aurora',
    description: 'A little northern light for your next build.',
    tone: 'dark',
    backdrop: '#20282B',
    layout: 'compact',
    colors: ['#A6E3A1', '#89DCEB', '#9399B2', '#F38BA8'],
    author: 'Mood Shell',
    builtIn: true,
  },
  {
    name: 'ember',
    title: 'Ember',
    description: 'Warm amber. Keep the good work glowing.',
    tone: 'dark',
    backdrop: '#302722',
    layout: 'compact',
    colors: ['#FFB454', '#FFD6A5', '#B39B8D', '#FF6B6B'],
    author: 'Mood Shell',
    builtIn: true,
  },
  {
    name: 'ocean',
    title: 'Ocean',
    description: 'Clear blue, a quiet mind, an open terminal.',
    tone: 'dark',
    backdrop: '#22283C',
    layout: 'compact',
    colors: ['#7DCFFF', '#C0CAF5', '#8992B0', '#F7768E'],
    author: 'Mood Shell',
    builtIn: true,
  },
  {
    name: 'paper',
    title: 'Paper',
    description: 'Purple ink for a bright place to think.',
    tone: 'light',
    backdrop: '#F2F0E8',
    layout: 'compact',
    colors: ['#5C3599', '#263238', '#62676B', '#B42338'],
    author: 'Mood Shell',
    builtIn: true,
  },
  {
    name: 'afterhours',
    title: 'Afterhours',
    description: 'Pink and lavender, with a little room to breathe.',
    tone: 'dark',
    backdrop: '#25232F',
    layout: 'two-line',
    colors: ['#FF77CC', '#E0DEF4', '#908CAA', '#EB6F92'],
    author: 'Andrew Mager',
    builtIn: false,
  },
];
export function validateTheme(theme) {
  if (
    typeof theme.name !== 'string' ||
    !/^[A-Za-z0-9_-]{1,48}$/.test(theme.name)
  )
    throw new Error(
      'Use 1–48 letters, numbers, hyphens, or underscores for the theme name.',
    );
  if (!['compact', 'two-line'].includes(theme.layout))
    throw new Error('Choose compact or two-line.');
  if (
    !Array.isArray(theme.colors) ||
    theme.colors.length !== 4 ||
    !theme.colors.every((color) => /^#[0-9a-f]{6}$/i.test(color))
  )
    throw new Error(
      'Each color needs a complete six-digit hex value, like #C4A7E7.',
    );
  return theme;
}
function prose(value, max) {
  return String(value ?? '')
    .slice(0, max)
    .replace(/[\r\n\u0000-\u001f\u007f]/g, ' ')
    .replace(/[<>`\\]/g, '');
}
export function themeDocument(theme) {
  validateTheme(theme);
  const [accent, path, muted, error] = theme.colors;
  return `# ${theme.name}\n\n${prose(theme.description, 300)}\n\nBy ${prose(theme.author || 'a Mood Shell maker', 80)}. Shared under the MIT license.\nStyles the prompt only. Set your terminal background separately.\n\n\`\`\`moodsh\n[mood]\nname = "${theme.name}"\nlayout = "${theme.layout}"\n\n[mood.palette]\naccent = "${accent}"\npath = "${path}"\nmuted = "${muted}"\nerror = "${error}"\n\`\`\`\n\nPreview: \`moodsh theme preview --file ./${theme.name}.md\`\nApply: \`moodsh theme apply ./${theme.name}.md\`\n`;
}
export function submissionUrl(theme) {
  const document = themeDocument(theme);
  const body = `## Theme submission\n\n${document}\n\n## Permission\n\nI created this palette or have permission to share it under the MIT license. Please review it for the gallery.\n`;
  return (
    `${repo}/issues/new?` +
    new URLSearchParams({
      template: 'theme.md',
      title: `Theme: ${theme.name}`,
      body,
    })
  );
}
