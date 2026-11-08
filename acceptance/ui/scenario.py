"""End-to-end UI scenario in headless Firefox against a real `ordo serve`.

Usage:  python3 acceptance/ui/scenario.py <ordo binary> <app dist dir> <work dir>

Scenarios (docs/requirements/acceptance/): register patients, required triage choice,
offline registration with reload and resend, command sees counts but no patient data.
Screenshots are written to the work dir. Exit code 1 on failure.
"""
import os, re, shutil, subprocess, sys, time, urllib.request

sys.path.insert(0, os.path.dirname(__file__))
from audit import audit  # noqa: E402
from marionette import CLICK_TEXT, FILL, Firefox  # noqa: E402

ORDO, DIST, WORK = sys.argv[1:4]
PORT = 18090
URL = f"http://127.0.0.1:{PORT}/"
failures = []


def check(condition, message):
    print(("ok   " if condition else "FAIL ") + message)
    if not condition:
        failures.append(message)


def wait_for(predicate, timeout=10.0):
    end = time.time() + timeout
    while time.time() < end:
        if predicate():
            return True
        time.sleep(0.25)
    return False


def setup():
    shutil.rmtree(WORK, ignore_errors=True)
    os.makedirs(WORK)
    run = lambda *a: subprocess.run([ORDO, *a], check=True, capture_output=True, text=True).stdout
    open(f"{WORK}/master.key", "w").write(run("generate-key"))
    keys = {}
    for name in ("anna", "ben"):
        keys[name] = dict(re.findall(r'^(\w+) = "([^"]+)"', run("generate-device-key"), re.M))
    seed = ""
    for name, compartments in (("anna", '"incident", "med.clinical"'), ("ben", '"incident"')):
        seed += f'[[user]]\nid = "{name}@localhost"\ncompartments = [{compartments}]\n\n'
        seed += f'[[device]]\nid = "{keys[name]["id"]}"\nuser = "{name}@localhost"\npublic_key = "{keys[name]["public_key"]}"\n\n'
    open(f"{WORK}/seed.toml", "w").write(seed)
    return keys


def start_server():
    env = dict(os.environ, ORDO_SERVER_NAME="localhost", ORDO_LISTEN=f"127.0.0.1:{PORT}",
               ORDO_DATA_DIR=f"{WORK}/data", ORDO_MASTER_KEY_FILE=f"{WORK}/master.key",
               ORDO_SEED_FILE=f"{WORK}/seed.toml", ORDO_APP_DIR=DIST)
    proc = subprocess.Popen([ORDO, "serve"], env=env, stdout=open(f"{WORK}/server.log", "a"), stderr=subprocess.STDOUT)
    def ready():
        try:
            return urllib.request.urlopen(f"{URL}readyz", timeout=1).status == 200
        except Exception:
            return False
    assert wait_for(ready, 20), "server did not become ready"
    return proc


def pair(ff, user, keys, incident=None):
    ff.go(URL)
    wait_for(lambda: ff.js("return !!document.getElementById('user')"))
    ff.js(FILL, "user", user)
    ff.js(FILL, "device", keys["id"])
    ff.js(FILL, "key", keys["private_key"])
    if incident:
        ff.js(FILL, "incident", incident)
    else:
        ff.js(CLICK_TEXT, "Neuen Einsatz anlegen")
    incident = ff.js("return document.getElementById('incident').value")
    ff.js(CLICK_TEXT, "Koppeln")
    return incident


status = lambda ff: ff.js("return document.querySelector('.status')?.innerText.replace(/\\n/g, ' | ') || ''")
rows = lambda ff: ff.js("return [...document.querySelectorAll('#main tbody tr')].map(r => r.cells[0].innerText + ' ' + r.cells[1].innerText.split('\\n')[0] + ' ' + r.cells[4].innerText)")


def register(ff, tag, index=None):
    # The form appears once the server's welcome granted access (slow CI runners!).
    assert wait_for(lambda: ff.js("return !!document.getElementById('tag')"), 15), "registration form missing"
    ff.js(FILL, "tag", tag)
    if index is not None:
        ff.js("document.querySelectorAll('input[name=category]')[arguments[0]].click()", index)
    ff.js(CLICK_TEXT, "Registrieren")
    time.sleep(0.5)


keys = setup()
server = start_server()
anna = Firefox(f"{WORK}/profile-anna", 2828)
try:
    anna.go(URL)
    wait_for(lambda: anna.js("return !!document.getElementById('user')"))
    audit(anna, "Pairing page", check)
    incident = pair(anna, "anna@localhost", keys["anna"])
    check(wait_for(lambda: "Verbunden" in status(anna)), "medic device connects")
    for tag, i in (("DE-0042", 0), ("DE-0043", 1), ("DE-0044", 1)):
        register(anna, tag, i)
    check(wait_for(lambda: "Alles gesendet" in status(anna)), "registrations are sent")
    check(len(rows(anna)) == 3, f"UHS board lists 3 patients: {rows(anna)}")

    register(anna, "DE-0099")  # no triage category chosen
    check(len(rows(anna)) == 3, "registration without a triage category is not possible")
    anna.js(FILL, "tag", "")

    anna.js("document.querySelector('button[aria-label=\"Nachsichtung DE-0044: SK I\"]').click()")
    check(wait_for(lambda: "nachgesichtet" in anna.js("return document.querySelector('[role=status]').innerText")),
          "re-triage is announced to screen readers")
    check(wait_for(lambda: "Alles gesendet" in status(anna)), "re-triage is sent")

    # [itest->dsn~offline-app-shell~1]
    check(wait_for(lambda: anna.js("return !!navigator.serviceWorker.controller"), 15),
          "the app shell is installed for offline use")
    server.terminate(); server.wait()
    check(wait_for(lambda: "Offline" in status(anna)), "loss of connection is shown")
    register(anna, "DE-0045", 2)
    check("1 Eingabe noch nicht gesendet" in status(anna), f"pending input is shown: {status(anna)}")
    # [itest->dsn~local-persistence~1]
    anna.go(URL)  # reload while the server is down: the app must start from the device
    check(wait_for(lambda: any(r.startswith("DE-0045") and "noch nicht gesendet" in r for r in rows(anna))),
          "offline registration survives a reload")
    server = start_server()
    check(wait_for(lambda: "Alles gesendet" in status(anna), 15), "after reconnecting, everything is sent")
    # [itest->dsn~triage-badge~1]
    badges = anna.js("return [...document.querySelectorAll('.triage')].map(b => !!b.querySelector('.triage__shape') && b.innerText.trim().length > 0)")
    check(len(badges) > 0 and all(badges), f"every triage badge has a shape and a text label ({len(badges)} badges)")
    audit(anna, "Med workspace", check)
    anna.shot(f"{WORK}/med-day.png")
    anna.js("document.documentElement.dataset.theme = 'night'")
    anna.shot(f"{WORK}/med-night.png")
finally:
    anna.quit()

ben = Firefox(f"{WORK}/profile-ben", 2829)
try:
    pair(ben, "ben@localhost", keys["ben"], incident)
    check(wait_for(lambda: "Kein Zugriff" in ben.js("return document.querySelector('#main')?.innerText || ''")),
          "command device has no access to patient data")
    ben.js(CLICK_TEXT, "Einsatzleitung")
    counts = lambda: ben.js("return [...document.querySelectorAll('#main tbody tr')].map(r => [...r.cells].map(c => c.innerText.split('\\n')[0]).join(' '))")
    check(wait_for(lambda: counts() == ["UHS 1 2 1 1 0 0 4"]), f"command sees counts SK I 2, SK II 1, SK III 1: {counts()}")
    stored = ben.js("return localStorage.getItem('ordo.replica') || ''")
    check(stored != "" and "44452d30303432" not in stored, "command device stores no triage tag number")
    audit(ben, "Command workspace", check)
    ben.shot(f"{WORK}/command.png")
    ben.js("document.documentElement.dataset.theme = 'high-contrast'")
    ben.shot(f"{WORK}/command-high-contrast.png")
finally:
    ben.quit()
    server.terminate()

print(f"\n{len(failures)} failure(s)")
sys.exit(1 if failures else 0)
