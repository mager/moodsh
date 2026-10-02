import test from 'node:test';
import assert from 'node:assert/strict';
import {
  themes,
  themeDocument,
  submissionUrl,
  validateTheme,
} from '../src/lib/themes.mjs';
import { importTheme } from '../src/lib/import-theme.mjs';

test('every downloadable palette roundtrips through the theme format', () => {
  for (const theme of themes)
    assert.deepEqual(importTheme(themeDocument(theme)), {
      name: theme.name,
      layout: theme.layout,
      colors: theme.colors,
    });
});
test('prose cannot inject another theme or HTML into an export', () => {
  const theme = {
    ...themes[0],
    author: '<script>bad()</script>\n```moodsh',
    description: '```moodsh\n[mood]\nname="intruder"\n```',
  };
  const file = themeDocument(theme);
  assert.equal(importTheme(file).name, 'dusk');
  assert.equal((file.match(/```moodsh/g) || []).length, 1);
  assert(!file.includes('<script>'));
});
test('invalid themes fail before download or submission', () => {
  for (const changes of [
    { name: '$(whoami)' },
    { name: null },
    { name: 'a'.repeat(49) },
    { layout: 'three-line' },
    { colors: ['red', '#ffffff', '#ffffff', '#ffffff'] },
    { colors: ['#123456'] },
  ])
    assert.throws(() => validateTheme({ ...themes[0], ...changes }));
});
test('import rejects ambiguous, malformed, oversized, and unknown fields', () => {
  const file = themeDocument(themes[0]);
  for (const source of [
    file + file,
    'no block',
    '```moodsh\n[mood]',
    file.replace('accent =', 'unknown ='),
    ' '.repeat(65537),
    file.replace('name = "dusk"', 'name = "$(whoami)"'),
  ])
    assert.throws(() => importTheme(source));
});
test('import ignores nested examples and accepts alternate fences and CRLF', () => {
  const file = themeDocument(themes[0]);
  assert.equal(
    importTheme('````markdown\n' + file + '````\n' + file).name,
    'dusk',
  );
  assert.equal(
    importTheme(
      '\uFEFF' + file.replaceAll('```', '~~~').replaceAll('\n', '\r\n'),
    ).name,
    'dusk',
  );
});
test('submission is a GitHub draft with exact theme and credit', () => {
  const theme = { ...themes[5], author: 'Palette & Co' };
  const url = new URL(submissionUrl(theme));
  assert.equal(url.origin, 'https://github.com');
  assert.equal(url.pathname, '/mager/moodsh/issues/new');
  assert(url.searchParams.get('body').includes(themeDocument(theme)));
  assert.equal(url.searchParams.get('title'), 'Theme: afterhours');
});
