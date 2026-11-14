// Starts and stops a real `ordo serve` for the browser tests.
import { ChildProcess, execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { DEX_API, DEX_URL } from "./dex";

export const ORDO = path.resolve(process.env.ORDO_BIN ?? path.join(__dirname, "../../../target/debug/ordo"));
export const DIST = path.resolve(process.env.ORDO_APP_DIST ?? path.join(__dirname, "../../../platform/ordo-app/dist"));

export type DeviceKey = { id: string; public_key: string; private_key: string };

export class OrdoServer {
  private proc?: ChildProcess;
  /** The address search's geocoder (ADR-0066); off unless a test starts a stand-in, so
   *  the tests never ask the public Nominatim. */
  geocoder = "off";

  constructor(readonly work: string, readonly port: number) {}

  get url(): string {
    return `http://127.0.0.1:${this.port}/`;
  }

  /** Creates the master key, the client secret for Dex, and (for users paired by hand)
   *  one device key per user plus the seed file. */
  setup(users: { name: string; roles: string[] }[] = []): Record<string, DeviceKey> {
    fs.rmSync(this.work, { recursive: true, force: true });
    fs.mkdirSync(this.work, { recursive: true });
    const run = (...args: string[]) => execFileSync(ORDO, args, { encoding: "utf8" });
    fs.writeFileSync(path.join(this.work, "master.key"), run("generate-key"));
    fs.writeFileSync(path.join(this.work, "oidc-secret"), "ordo-test-secret");
    const keys: Record<string, DeviceKey> = {};
    let seed = "";
    for (const user of users) {
      const values = Object.fromEntries([...run("generate-device-key").matchAll(/^(\w+) = "([^"]+)"/gm)].map((m) => [m[1], m[2]]));
      const key = values as DeviceKey;
      keys[user.name] = key;
      const id = `${user.name}@localhost`;
      seed += `[[user]]\nid = "${id}"\nroles = [${user.roles.map((r) => `"${r}"`).join(", ")}]\n\n`;
      seed += `[[device]]\nid = "${key.id}"\nuser = "${id}"\npublic_key = "${key.public_key}"\n\n`;
    }
    fs.writeFileSync(path.join(this.work, "seed.toml"), seed);
    return keys;
  }

  async start(): Promise<void> {
    // Never test against another ORDO on the port, e.g. the local demo.
    const taken = await fetch(`${this.url}readyz`).then(() => true, () => false);
    if (taken) throw new Error(`port ${this.port} is in use (the local demo?); set ORDO_E2E_PORT`);
    const log = fs.openSync(path.join(this.work, "server.log"), "a");
    this.proc = spawn(ORDO, ["serve"], {
      env: {
        ...process.env,
        ORDO_SERVER_NAME: "localhost",
        ORDO_ADMINS: "dora", // plans incidents and grants roles (ADR-0046)
        ORDO_GRAVATAR: "off", // no requests to the internet from tests
        ORDO_LISTEN: `127.0.0.1:${this.port}`,
        ORDO_DATA_DIR: path.join(this.work, "data"),
        ORDO_MASTER_KEY_FILE: path.join(this.work, "master.key"),
        ORDO_SEED_FILE: path.join(this.work, "seed.toml"),
        // No region at first; the incident command cuts one per incident from this "world
        // map" in the app (MAP-8, ADR-0062; made by make_region.py).
        ORDO_MAP_DIR: path.join(this.work, "maps"),
        ORDO_MAP_SOURCE: path.join(__dirname, "world.pmtiles"),
        ORDO_GEOCODER: this.geocoder,
        ORDO_APP_DIR: DIST,
        // Login with Dex (support/dex.yaml).
        ORDO_OIDC_ISSUER: DEX_URL,
        ORDO_OIDC_CLIENT_ID: "ordo",
        ORDO_OIDC_CLIENT_SECRET_FILE: path.join(this.work, "oidc-secret"),
        ORDO_PUBLIC_URL: `http://127.0.0.1:${this.port}`,
        // Local accounts through Dex's API (ADR-0060).
        ORDO_DEX_API: DEX_API.url,
        ORDO_DEX_API_CA_FILE: path.join(DEX_API.certs, "ca.pem"),
        ORDO_DEX_API_CERT_FILE: path.join(DEX_API.certs, "ordo.pem"),
        ORDO_DEX_API_KEY_FILE: path.join(DEX_API.certs, "ordo-key.pem"),
      },
      stdio: ["ignore", log, log],
    });
    const end = Date.now() + 20_000;
    while (Date.now() < end) {
      if (this.proc.exitCode !== null) throw new Error(`server exited (${this.proc.exitCode}); see ${this.work}/server.log`);
      try {
        if ((await fetch(`${this.url}readyz`)).ok) return;
      } catch {
        // not up yet
      }
      await new Promise((r) => setTimeout(r, 200));
    }
    throw new Error("server did not become ready");
  }

  async stop(): Promise<void> {
    const proc = this.proc;
    if (!proc || proc.exitCode !== null) return;
    const exited = new Promise((r) => proc.once("exit", r));
    proc.kill("SIGTERM");
    await exited;
  }
}
