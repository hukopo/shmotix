// Картинки для README из спрайтов ui/index.html: docs/cat.svg (анимированный котик).
// Запуск: node scripts/make-readme-art.js [sit.svg — статичный кадр для соцпревью]
const fs = require('fs');
const s = fs.readFileSync('ui/index.html', 'utf8');
const code = s.slice(s.indexOf('const SIT'), s.indexOf('const ctx'));
const { F, COLORS } = eval(code + ';({ F, COLORS })');
const W = 29, H = 34;

// строка → полоски одного цвета, чтобы SVG был маленьким
const frame = name => F[name].trim().split('\n').map((r, y) => {
  let out = '';
  for (let x = 0; x < r.length;) {
    let e = x; while (e < r.length && r[e] === r[x]) e++;
    if (COLORS[r[x]]) out += `<rect x="${x}" y="${y}" width="${e - x}" height="1" fill="${COLORS[r[x]]}"/>`;
    x = e;
  }
  return out;
}).join('');

const seq = [['sit', 1.2], ['blink', .15], ['sit', .8], ['tail', .4], ['sit', .4], ['tail', .4], ['happy', .8], ['meow', .5],
  ['happy', .4], ['look', 1], ['sit', .6], ['lick1', .25], ['lick2', .25], ['lick1', .25], ['lick2', .25], ["sleep", 2.2], ["blink", .15], ["ear", .3], ["sit", .3], ["ear", .3]];
const T = seq.reduce((a, [, d]) => a + d, 0);
let t = 0, css = '', g = '';
seq.forEach(([n, d], i) => {
  const a = (t / T * 100).toFixed(2), b = ((t + d) / T * 100).toFixed(2); t += d;
  css += `#f${i}{animation:k${i} ${T}s step-end infinite}@keyframes k${i}{0%{opacity:0}${a}%{opacity:1}${b}%{opacity:0}}`;
  g += `<g id="f${i}" opacity="0">${frame(n)}</g>`;
});
const svg = (body, extra = '') => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${W} ${H}" ${extra} shape-rendering="crispEdges">${body}</svg>`;
fs.writeFileSync('docs/cat.svg', svg(`<style>${css}</style>${g}`, `width="${W * 6}" height="${H * 6}"`));
if (process.argv[2]) fs.writeFileSync(process.argv[2], svg(frame('sit')));
