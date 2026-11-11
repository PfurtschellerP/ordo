// Runs before the app (index.html): the service worker, and the chosen colour theme before
// the first paint (no flash of the wrong theme). A file, not inline, for the Content
// Security Policy.
if ("serviceWorker" in navigator) navigator.serviceWorker.register("/sw.js");
// Apply the chosen colour theme before the first paint (no flash of the wrong theme).
try { const t = localStorage.getItem("ordo.theme"); if (t) document.documentElement.dataset.theme = t; } catch (e) {}
