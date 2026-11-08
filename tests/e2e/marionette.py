"""Minimal Firefox Marionette client for a headless UI check (no extra dependencies)."""
import base64, json, os, socket, subprocess, sys, time

class Firefox:
    def __init__(self, profile, port):
        os.makedirs(profile, exist_ok=True)
        with open(os.path.join(profile, "user.js"), "w") as f:
            f.write(f'user_pref("marionette.port", {port});\n')
            # No HTTP cache: offline starts must come from the app's service worker.
            f.write('user_pref("browser.cache.disk.enable", false);\n')
            f.write('user_pref("browser.cache.memory.enable", false);\n')
        self.proc = subprocess.Popen(["firefox", "--headless", "--no-remote", "--marionette",
                                      "--profile", profile], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(100):
            try:
                self.sock = socket.create_connection(("127.0.0.1", port)); break
            except OSError:
                time.sleep(0.2)
        self.buf = b""; self.id = 0
        self._read()  # greeting
        self.cmd("WebDriver:NewSession", {"capabilities": {}})
        self.cmd("WebDriver:SetWindowRect", {"width": 1200, "height": 1000})

    def _read(self):
        while b":" not in self.buf:
            self.buf += self.sock.recv(65536)
        n, rest = self.buf.split(b":", 1); n = int(n)
        while len(rest) < n:
            rest += self.sock.recv(65536)
        self.buf = rest[n:]
        return json.loads(rest[:n])

    def cmd(self, name, params=None):
        self.id += 1
        data = json.dumps([0, self.id, name, params or {}]).encode()
        self.sock.sendall(str(len(data)).encode() + b":" + data)
        while True:
            msg = self._read()
            if msg[0] == 1 and msg[1] == self.id:
                if msg[2]: raise RuntimeError(f"{name}: {msg[2]}")
                return msg[3]

    def js(self, script, *args):
        return self.cmd("WebDriver:ExecuteScript", {"script": script, "args": list(args)}).get("value")

    def go(self, url):
        self.cmd("WebDriver:Navigate", {"url": url})

    def shot(self, path, full=True):
        """Screenshot of the whole page, or (full=False) of what the viewport shows, as a
        user sees it; fixed bars appear where they really are."""
        end = time.time() + 5
        while time.time() < end and self.js("return document.getAnimations().length") > 0:
            time.sleep(0.05)
        png = self.cmd("WebDriver:TakeScreenshot", {"full": full})["value"]
        open(path, "wb").write(base64.b64decode(png))

    def keys(self, *chords):
        """Real key presses (with the browser's default actions, unlike synthetic events).
        Each chord is a string of keys pressed together, e.g. CTRL + "k", or text to type."""
        actions = []
        for chord in chords:
            if chord and chord[0] in MODIFIERS:
                down = [{"type": "keyDown", "value": k} for k in chord]
                up = [{"type": "keyUp", "value": k} for k in reversed(chord)]
                actions += down + up
            else:
                for k in chord:
                    actions += [{"type": "keyDown", "value": k}, {"type": "keyUp", "value": k}]
        self.cmd("WebDriver:PerformActions", {"actions": [{"type": "key", "id": "keyboard", "actions": actions}]})
        self.cmd("WebDriver:ReleaseActions")

    def quit(self):
        try: self.cmd("Marionette:Quit", {"flags": ["eForceQuit"]})
        except Exception: pass
        self.proc.wait(timeout=20)

# WebDriver key codes.
CTRL, ENTER, ESCAPE, DOWN, BACKSPACE = "\ue009", "\ue007", "\ue00c", "\ue015", "\ue003"
MODIFIERS = {CTRL}

# Helpers that drive the app like a user (input events, clicks).
FILL = """
const [id, value] = arguments;
const el = document.getElementById(id);
el.value = value; el.dispatchEvent(new Event('input', {bubbles: true}));
"""
CLICK_TEXT = """
const [text] = arguments;
const el = [...document.querySelectorAll('button, label')].find(b => b.textContent.trim().includes(text));
if (!el) return false; el.click(); return true;
"""
