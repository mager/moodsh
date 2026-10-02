import { readdir, mkdtemp, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const binary = resolve(process.argv[2] || 'target/debug/moodsh');
const directory = fileURLToPath(new URL('../dist/themes/', import.meta.url));
const scratch = await mkdtemp(join(tmpdir(), 'moodsh-web-'));
try {
  const files = (await readdir(directory)).filter((name) =>
    name.endsWith('.md'),
  );
  if (!files.length)
    throw new Error('Build the website before checking its themes.');
  for (const name of files) {
    const result = spawnSync(
      binary,
      ['theme', 'preview', '--file', join(directory, name)],
      {
        encoding: 'utf8',
        env: { ...process.env, MOODSH_CONFIG: join(scratch, 'config.toml') },
      },
    );
    if (result.error || result.status !== 0) {
      throw new Error(`${name}: ${result.error || result.stderr}`);
    }
    console.log(`CLI accepted ${name}`);
  }
} finally {
  await rm(scratch, { recursive: true, force: true });
}
