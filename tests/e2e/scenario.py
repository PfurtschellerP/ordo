"""End-to-end UI scenario in headless Firefox against a real `ordo serve`.

Usage:  python3 tests/e2e/scenario.py <ordo binary> <app dist dir> <work dir>

Scenarios (tests/acceptance/features/): register patients, required triage choice,
patient record with photo, keyboard command palette, offline registration with reload
and resend, command sees counts but no patient data, phone layout with bottom
navigation. Screenshots are written to the work dir. Exit code 1 on failure.
"""
import base64, os, re, shutil, subprocess, sys, time, urllib.request

sys.path.insert(0, os.path.dirname(__file__))
from audit import audit  # noqa: E402
from marionette import BACKSPACE, CLICK_TEXT, CTRL, DOWN, ENTER, ESCAPE, FILL, Firefox  # noqa: E402

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
rows = lambda ff: ff.js("return [...document.querySelectorAll('.patient-card')].map(c => c.dataset.tag + ' ' + c.querySelector('.triage').innerText.split('\\n')[0] + ' ' + c.querySelector('.patient-card__state').innerText)")
title = lambda ff: ff.js("return document.querySelector('.app-header__title')?.innerText || ''")
announced = lambda ff: ff.js("return document.querySelector('body > [role=status]').innerText")
hash_ = lambda ff: ff.js("return decodeURIComponent(location.hash)")
facts = lambda ff, section: ff.js("return [...document.querySelectorAll(`[aria-labelledby=section-${arguments[0]}] .facts`)].map(f => f.innerText.replace(/\\n/g, ' ')).join(' / ')", section)


def stored_replica(ff):
    """The replica as stored on the device (bytes; Base64 with prefix, older: hex)."""
    text = ff.js("return localStorage.getItem('ordo.replica') || ''")
    return base64.b64decode(text[4:]) if text.startswith("b64:") else bytes.fromhex(text)


def open_record(ff, tag):
    ff.js("document.querySelector(`.patient-card[data-tag='${arguments[0]}']`).click()", tag)
    assert wait_for(lambda: ff.js("return document.getElementById('record-title')?.innerText") == f"Patient {tag}"), f"record {tag} missing"


def fill_form(ff, form, values, choices=()):
    """Fills an open editor: text values by field ID, choices by option label."""
    for field, value in values.items():
        ff.js("const el = document.getElementById(arguments[0]); el.value = arguments[1]; el.dispatchEvent(new Event('input', {bubbles: true}));",
              f"f-{form}-{field}", value)
    for field, label in choices:
        ff.js("[...document.querySelectorAll(`#${arguments[0]} label`)].find(l => l.innerText.trim() === arguments[1]).click()",
              f"f-{form}-{field}", label)


def save_form(ff, form):
    ff.js("document.getElementById(arguments[0]).closest('section').querySelector('form.editor button[type=submit]').click()", f"section-{form}")


SELECT = """
const [id, text] = arguments;
const el = document.getElementById(id) || document.querySelector(`select[aria-label="${id}"]`);
const option = [...el.options].find(o => o.text.startsWith(text));
el.value = option.value; el.dispatchEvent(new Event('change', {bubbles: true}));
"""


def open_workspace(ff, label):
    """Opens the launcher and chooses a workspace by its label."""
    ff.js("document.getElementById('launcher-button').click()")
    ok = ff.js("""
      const item = [...document.querySelectorAll('.launcher__item')].find(b => b.innerText.trim() === arguments[0]);
      if (!item || item.getAttribute('aria-disabled') === 'true') return false;
      item.click(); return true;""", label)
    assert ok, f"workspace {label} not available"


def open_view(ff, label):
    assert wait_for(lambda: ff.js("""
      const b = [...document.querySelectorAll('.views__item')].find(b => b.innerText.trim() === arguments[0]);
      if (!b) return false; b.click(); return true;""", label)), f"view {label} missing"


def launcher_item(ff, label):
    """(aria-disabled, reason) of a launcher item."""
    return ff.js("""
      const b = [...document.querySelectorAll('.launcher__item')].find(b => b.innerText.trim() === arguments[0]);
      return b ? [b.getAttribute('aria-disabled'), document.getElementById(b.getAttribute('aria-describedby')).innerText] : null;""", label)


def create_uhs(ff, name, capacity):
    assert wait_for(lambda: ff.js("return !!document.getElementById('uhs-name')"), 15), "UHS form missing"
    ff.js(FILL, "uhs-name", name)
    ff.js(FILL, "uhs-capacity", str(capacity))
    ff.js(CLICK_TEXT, "UHS anlegen")
    time.sleep(0.5)


def register(ff, tag, index=None, uhs=None):
    # The form appears once the server's welcome granted access (slow CI runners!).
    assert wait_for(lambda: ff.js("return !!document.getElementById('tag')"), 15), "registration form missing"
    if uhs:
        ff.js(SELECT, "uhs", uhs)
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
    check(wait_for(lambda: "Sanität – UHS" in title(anna)), "a medic starts in the workspace Sanität – UHS")
    open_view(anna, "UHS verwalten")
    create_uhs(anna, "UHS Bühne Nord", 10)
    create_uhs(anna, "UHS Eingang", 5)
    # [itest->dsn~uhs-registry~1]
    check(wait_for(lambda: anna.js("return document.querySelectorAll('.load-list__item').length") == 2),
          "two UHS are set up and listed with their load")
    open_view(anna, "Patienten")
    for tag, i, uhs in (("DE-0042", 0, "UHS Bühne Nord"), ("DE-0043", 1, "UHS Bühne Nord"), ("DE-0044", 1, "UHS Eingang")):
        register(anna, tag, i, uhs)
    check(wait_for(lambda: "Alles gesendet" in status(anna)), "registrations are sent")
    check(len(rows(anna)) == 3, f"UHS board lists 3 patients: {rows(anna)}")

    register(anna, "DE-0099")  # no triage category chosen
    check(len(rows(anna)) == 3, "registration without a triage category is not possible")
    anna.js(FILL, "tag", "")

    # Patient record (EPCR-1…4, PAT-8), opened from the list; the URL names the record.
    open_record(anna, "DE-0044")
    check(hash_(anna) == "#/med.uhs/patients/DE-0044", f"the record has its own address: {hash_(anna)}")
    anna.js("document.querySelector('button[aria-label=\"Nachsichtung DE-0044: SK I\"]').click()")
    check(wait_for(lambda: "nachgesichtet" in announced(anna)), "re-triage is announced to screen readers")
    check(wait_for(lambda: "Alles gesendet" in status(anna)), "re-triage is sent")

    # [itest->dsn~patient-care-record~1]
    anna.js("document.getElementById('edit-identity').click()")
    fill_form(anna, "identity", {"family_name": "Mustermann", "given_name": "Max", "age_estimate": "40"}, [("sex", "männlich")])
    save_form(anna, "identity")
    check(wait_for(lambda: "Mustermann" in facts(anna, "identity") and "männlich" in facts(anna, "identity")),
          f"identity is documented: {facts(anna, 'identity')}")
    anna.js("document.getElementById('new-vitals').click()")
    fill_form(anna, "vitals", {"heart_rate": "abc"})
    save_form(anna, "vitals")
    check(wait_for(lambda: anna.js("return document.getElementById('f-vitals-heart_rate').getAttribute('aria-invalid')") == "true"),
          "an invalid number is marked and explained")
    fill_form(anna, "vitals", {"heart_rate": "112", "spo2": "97", "gcs_eye": "4", "gcs_verbal": "5", "gcs_motor": "6"})
    save_form(anna, "vitals")
    check(wait_for(lambda: "GCS gesamt 15" in facts(anna, "vitals") and "112 /min" in facts(anna, "vitals")),
          f"vital signs are documented with the GCS total: {facts(anna, 'vitals')}")
    anna.js("document.querySelector('[aria-labelledby=section-vitals] .entry button').click()")  # Korrigieren
    fill_form(anna, "vitals", {"heart_rate": "92"})
    save_form(anna, "vitals")
    check(wait_for(lambda: "92 /min" in facts(anna, "vitals") and anna.js(
          "return document.querySelector('[aria-labelledby=section-vitals] .corrected summary')?.innerText") == "1 korrigierter Eintrag"),
          "a correction replaces the entry and keeps the old one visible")

    # Unsaved input survives switching views (UX-5).
    anna.js("document.getElementById('edit-history').click()")
    fill_form(anna, "history", {"allergies": "Penicillin"})
    open_view(anna, "UHS verwalten")
    open_view(anna, "Patienten")
    open_record(anna, "DE-0044")
    check(wait_for(lambda: anna.js("return document.getElementById('f-history-allergies')?.value") == "Penicillin"),
          "unsaved input is still there after switching views")
    save_form(anna, "history")

    # [itest->dsn~patient-photo~1]
    anna.js("""
      const canvas = document.createElement('canvas'); canvas.width = 3000; canvas.height = 2000;
      const ctx = canvas.getContext('2d');
      for (let i = 0; i < 400; i++) { ctx.fillStyle = `hsl(${i * 7 % 360} 70% 50%)`; ctx.fillRect(Math.random() * 3000, Math.random() * 2000, 300, 200); }
      canvas.toBlob((blob) => {
        const dt = new DataTransfer(); dt.items.add(new File([blob], 'photo.png', {type: 'image/png'}));
        const input = document.getElementById('photo-upload'); input.files = dt.files;
        input.dispatchEvent(new Event('change', {bubbles: true}));
      }, 'image/png');""")
    check(wait_for(lambda: anna.js("const i = document.querySelector('img.photo'); return !!i && i.complete && i.naturalWidth > 0"), 20),
          "a photo is taken for identification and shown in the record")
    size = anna.js("const i = document.querySelector('img.photo'); return [i.naturalWidth, i.naturalHeight, atob(i.src.split(',')[1]).length, i.src.slice(0, 23)]")
    check(size[0] <= 800 and size[1] <= 800 and size[2] <= 160 * 1024 and size[3] == "data:image/jpeg;base64,",
          f"the photo is downscaled and re-encoded as JPEG: {size}")
    check(wait_for(lambda: "Alles gesendet" in status(anna)), "record entries and photo are sent")
    audit(anna, "Patient record", check)
    anna.shot(f"{WORK}/record.png")

    anna.js("history.back()")
    check(wait_for(lambda: hash_(anna) == "#/med.uhs/patients" and not anna.js("return !!document.getElementById('record-title')")),
          "the back button returns from the record to the list")
    check(wait_for(lambda: any(r.startswith("DE-0044") for r in rows(anna))) and "Mustermann, Max" in anna.js(
          "return document.querySelector(`.patient-card[data-tag='DE-0044']`).innerText"),
          "the list shows the documented name")
    stored = stored_replica(anna)
    check(b"Mustermann" not in stored and b"Penicillin" not in stored,
          "clinical entries are stored sealed on the medic's device, not in plain text")

    # [itest->dsn~command-palette~1]
    anna.js("document.getElementById('tag').focus()")
    anna.keys(CTRL + "k")
    check(wait_for(lambda: anna.js("return document.getElementById('palette').open")), "Ctrl+K opens the command palette")
    check(anna.js("return document.activeElement.id") == "palette-input", "the search field has the focus")
    anna.keys("0043")
    check(wait_for(lambda: anna.js("return document.querySelector('#palette-list [aria-selected=true] .palette__label')?.innerText") == "DE-0043"),
          "typing part of a triage tag number finds the patient")
    audit(anna, "Command palette", check, widths=(1200,))
    anna.keys(ENTER, "sk iii")
    check(wait_for(lambda: anna.js("return document.querySelector('#palette-list [aria-selected=true] .palette__label')?.innerText") == "Sichtung ändern: SK III"),
          "the patient's quick actions are offered")
    anna.keys(ENTER)
    check(wait_for(lambda: "DE-0043 nachgesichtet: SK III" in announced(anna)), "a re-triage runs from the palette without leaving the keyboard")
    check(not anna.js("return document.getElementById('palette').open"), "the palette closes after the action")
    anna.keys(CTRL + "k", "vitalwerte")
    check(wait_for(lambda: anna.js("return document.getElementById('palette-list').children.length") == 0),
          "patient commands are only offered for a chosen patient")
    anna.keys(ESCAPE)
    check(wait_for(lambda: not anna.js("return document.getElementById('palette').open")), "Escape closes the palette")
    check(anna.js("return document.activeElement.id") == "tag", "focus returns to where it was")

    # [itest->dsn~offline-app-shell~1]
    check(wait_for(lambda: anna.js("return !!navigator.serviceWorker.controller"), 15),
          "the app shell is installed for offline use")
    server.terminate(); server.wait()
    check(wait_for(lambda: "Offline" in status(anna)), "loss of connection is shown")
    register(anna, "DE-0045", 2, "UHS Eingang")
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
    anna.js("document.getElementById('launcher-button').click()")
    audit(anna, "Launcher (medic)", check)
    anna.js("document.getElementById('launcher-button').click()")
    anna.shot(f"{WORK}/med-day.png")

    # Phone: navigation at the bottom, menu included (ADR-0025).
    anna.cmd("WebDriver:SetWindowRect", {"width": 390, "height": 844})
    time.sleep(0.3)
    bar = anna.js("""const n = document.querySelector('.navigation').getBoundingClientRect();
      const m = document.getElementById('launcher-button').getBoundingClientRect();
      return [Math.round(n.bottom), innerHeight, m.top >= n.top && m.bottom <= n.bottom]""")
    check(bar[0] == bar[1] and bar[2], f"on phones the views and the menu are in a bar at the bottom: {bar}")
    anna.js("window.scrollTo(0, 0)")
    anna.shot(f"{WORK}/med-phone.png", full=False)
    anna.js("document.getElementById('launcher-button').click()")
    sheet = anna.js("""const p = document.getElementById('launcher-panel').getBoundingClientRect();
      const n = document.querySelector('.navigation').getBoundingClientRect(); return [Math.round(p.bottom), Math.round(n.top), Math.round(p.top)]""")
    check(sheet[0] <= sheet[1] + 1 and sheet[2] > 0, f"the menu opens upwards above the bar: {sheet}")
    anna.shot(f"{WORK}/launcher-phone.png", full=False)
    anna.js("document.getElementById('launcher-button').click()")
    open_record(anna, "DE-0042")
    anna.shot(f"{WORK}/record-phone.png", full=False)
    check(not anna.js("return document.querySelector('.master').offsetParent"), "on phones the record is a page of its own")
    anna.js("history.back()")
    anna.cmd("WebDriver:SetWindowRect", {"width": 1200, "height": 1000})
    anna.js("document.documentElement.dataset.theme = 'night'")
    anna.shot(f"{WORK}/med-night.png")
finally:
    anna.quit()

ben = Firefox(f"{WORK}/profile-ben", 2829)
try:
    pair(ben, "ben@localhost", keys["ben"], incident)
    # [itest->dsn~workspace-navigation~2]
    check(wait_for(lambda: "Einsatzleitung – Lage" in title(ben)),
          "the commander starts in the first workspace they may use: Einsatzleitung – Lage")
    ben.js("document.getElementById('launcher-button').click()")
    check(launcher_item(ben, "UHS") == ["true", "Kein Zugriff in diesem Einsatz"],
          f"the UHS workspace stays in the menu, disabled, with the reason: {launcher_item(ben, 'UHS')}")
    audit(ben, "Launcher (commander)", check)
    ben.shot(f"{WORK}/launcher.png")
    ben.js("document.querySelector('.launcher').dispatchEvent(new KeyboardEvent('keydown', {key: 'Escape', bubbles: true}))")
    check(wait_for(lambda: ben.js("return document.getElementById('launcher-panel').hidden")), "Escape closes the launcher")
    check(ben.js("return document.activeElement.id") == "launcher-button", "focus returns to the launcher button")
    # DE-0043 was re-triaged to SK III from the palette.
    cards = lambda: ben.js("return [...document.querySelectorAll('.count-card')].map(c => [c.querySelector('h3').innerText, ...[...c.querySelectorAll('.count-card__n')].map(n => n.innerText), c.querySelector('p').innerText].join(' '))")
    check(wait_for(lambda: cards() == ["UHS Bühne Nord 1 0 1 0 0 Gesamt: 2 · Belegung: 2 / 10", "UHS Eingang 1 0 1 0 0 Gesamt: 2 · Belegung: 2 / 5"]),
          f"command sees counts and load per UHS: {cards()}")
    ben.cmd("WebDriver:SetWindowRect", {"width": 1700, "height": 1000})
    table = lambda: ben.js("return [...document.querySelectorAll('#main tbody tr')].filter(r => r.offsetParent).map(r => [...r.cells].map(c => c.innerText.split('\\n')[0]).join(' '))")
    check(wait_for(lambda: table() == ["UHS Bühne Nord 1 0 1 0 0 2 2 / 10", "UHS Eingang 1 0 1 0 0 2 2 / 5"]),
          f"where there is room, the counts are a table: {table()}")
    audit(ben, "Command workspace (wide)", check, widths=(1700,))
    stored = stored_replica(ben)
    check(len(stored) > 0 and b"DE-0042" not in stored and b"Mustermann" not in stored and b"\xff\xd8\xff" not in stored,
          "command device stores no triage tag number, no name and no photo")
    audit(ben, "Command workspace", check)
    ben.shot(f"{WORK}/command.png")

    # Resources: add a vehicle and set its status (UNIT-1, UNIT-3).
    open_view(ben, "Einsatzmittel")
    assert wait_for(lambda: ben.js("return !!document.getElementById('unit-call-sign')")), "resources form missing"
    ben.js(FILL, "unit-call-sign", "RTW 83/1")
    ben.js(FILL, "unit-type", "RTW")
    ben.js(FILL, "unit-organisation", "DRK Musterstadt")
    ben.js(CLICK_TEXT, "Einsatzmittel hinzufügen")
    assert wait_for(lambda: ben.js("return !!document.querySelector('select[aria-label=\"Status von RTW 83/1 setzen\"]')")), "unit row missing"
    ben.js(SELECT, "Status von RTW 83/1 setzen", "3 ")
    units_row = lambda: ben.js("return [...document.querySelectorAll('.unit-card')].map(c => [...c.querySelectorAll('h3, p')].map(e => e.innerText).join(' | ')).join('; ')")
    check(wait_for(lambda: units_row() == "RTW 83/1 | RTW · DRK Musterstadt | Status: 3 – Einsatz übernommen"),
          f"a unit is added and its status is set: {units_row()}")
    check(wait_for(lambda: "Alles gesendet" in status(ben)), "unit and status are sent")
    # Quick status change from the keyboard (UX-10).
    ben.keys(CTRL + "k", "rtw")
    check(wait_for(lambda: ben.js("return document.querySelector('#palette-list [aria-selected=true] .palette__label')?.innerText") == "RTW 83/1"),
          "the palette finds a unit by call sign")
    ben.keys(ENTER, "am einsatzort", ENTER)
    check(wait_for(lambda: "Status: 4 – Am Einsatzort" in units_row()), f"a status is set from the palette: {units_row()}")
    ben.keys(CTRL + "k", "patienten")
    check(wait_for(lambda: ben.js("return [...document.querySelectorAll('#palette-list [role=option]')].map(o => o.getAttribute('aria-disabled') + ' ' + o.innerText.replace(/\\n/g, ' '))").count(
          "true Ansicht Patienten Kein Zugriff in diesem Einsatz") == 1),
          "in the palette, views without access are shown disabled with the reason")
    ben.keys(ESCAPE)
    audit(ben, "Resources workspace", check)
    ben.shot(f"{WORK}/resources.png")
    ben.js("document.documentElement.dataset.theme = 'high-contrast'")
    ben.shot(f"{WORK}/command-high-contrast.png")
finally:
    ben.quit()
    server.terminate()

print(f"\n{len(failures)} failure(s)")
sys.exit(1 if failures else 0)
