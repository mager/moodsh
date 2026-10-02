import { themes, themeDocument } from '../../lib/themes.mjs';
export function getStaticPaths() {
  return themes.map((theme) => ({
    params: { name: theme.name },
    props: { theme },
  }));
}
export function GET({ props }) {
  return new Response(themeDocument(props.theme), {
    headers: { 'Content-Type': 'text/markdown; charset=utf-8' },
  });
}
