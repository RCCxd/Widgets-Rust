const T = window.__TAURI__;
const DEF = { theme: 'auto', accent: '#0a84ff', opacity: .62, radius: 24, scale: 1, hour12: false, seconds: false, weekStart: 0, onTop: false, autostart: false, show: { clock: true, cal: true, music: true } };
let S = DEF;
function apply(s) {
  S = { ...DEF, ...s, show: { ...DEF.show, ...(s.show || {}) } };
  const r = document.documentElement, dark = S.theme === 'dark' || (S.theme === 'auto' && matchMedia('(prefers-color-scheme:dark)').matches);
  r.dataset.theme = dark ? 'dark' : 'light';
  r.style.setProperty('--acc', S.accent); r.style.setProperty('--op', S.opacity); r.style.setProperty('--rad', S.radius + 'px');
  if (window.__W) r.style.zoom = S.scale;
  if (window.onS) window.onS();
}
T.core.invoke('get_settings').then(j => apply(JSON.parse(j || '{}')));
T.event.listen('settings', e => apply(JSON.parse(e.payload)));
matchMedia('(prefers-color-scheme:dark)').addEventListener('change', () => apply(S));
let d0 = null;
addEventListener('mousedown', e => { if (e.button === 0 && !e.target.closest('button,input,select,.nodrag')) d0 = [e.clientX, e.clientY]; });
addEventListener('mouseup', () => d0 = null);
addEventListener('mousemove', e => { if (d0 && Math.hypot(e.clientX - d0[0], e.clientY - d0[1]) > 4) { d0 = null; if (window.__W) T.window.getCurrentWindow().startDragging(); } });
