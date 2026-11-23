// node scripts/check_carta_ui.js — syntax of assets/carta.html's script, 8-language completeness, keys used exist,
// informal address (no formal pronouns) — run in CI with Node >= 24.
const fs = require('fs'), path = require('path');
const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'assets', 'carta.html'), 'utf8');
const js = html.match(/<script>([\s\S]*)<\/script>/)[1];
new Function(js);                                              // throws on a syntax error
const D = eval('({' + js.match(/const D=\{([\s\S]*?)\n\};\nlet LANG/)[1] + '})');
const langs = ['en', 'ro', 'fr', 'it', 'es', 'pt', 'de', 'hu'];
let bad = 0;
for (const [k, v] of Object.entries(D)) for (const l of langs) if (!v[l] || !v[l].trim()) { console.error('missing', k, l); bad++; }
const used = new Set([...html.matchAll(/data-(?:i|ph)="(\w+)"/g)].map(x => x[1]));
[...js.matchAll(/T\('(\w+)'\)/g)].forEach(x => used.add(x[1]));
for (const k of used) if (!D[k]) { console.error('undefined key', k); bad++; }
const formal = /dumneavoastră|\bvous\b|\bSie\b|\bIhre?\b|\bLei\b|\busted\b|\bvocê\b|\bÖn\b/;
for (const [k, v] of Object.entries(D)) for (const l of langs) if (formal.test(v[l])) { console.error('formal address', k, l); bad++; }
const site = fs.readFileSync(path.join(root, 'site-carta', 'index.html'), 'utf8').match(/<script>([\s\S]*)<\/script>/)[1];
new Function(site);
if (bad) process.exit(1);
console.log(`carta.html ok: ${Object.keys(D).length} strings x ${langs.length} languages`);
