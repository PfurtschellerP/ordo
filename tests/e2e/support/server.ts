// Starts and stops a real `ordo serve` for the browser tests.
import { ChildProcess, execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export const ORDO = path.resolve(process.env.ORDO_BIN ?? path.join(__dirname, "../../../target/debug/ordo"));
export const DIST = path.resolve(process.env.ORDO_APP_DIST ?? path.join(__dirname, "../../../platform/ordo-app/dist"));

export type DeviceKey = { id: string; public_key: string; private_key: string };

export class OrdoServer {
  private proc?: ChildProcess;

  constructor(readonly work: string, readonly port: number) {}

  get url(): string {
    return `http://127.0.0.1:${this.port}/`;
  }

  /** Creates the master key, one device key per user, and the seed file. */
  setup(users: { name: string; compartments: string[] }[]): Record<string, DeviceKey> {
    fs.rmSync(this.work, { recursive: true, force: true });
    fs.mkdirSync(this.work, { recursive: true });
    const run = (...args: string[]) => execFileSync(ORDO, args, { encoding: "utf8" });
    fs.writeFileSync(path.join(this.work, "master.key"), run("generate-key"));
    const keys: Record<string, DeviceKey> = {};
    let seed = "";
    for (const user of users) {
      const values = Object.fromEntries([...run("generate-device-key").matchAll(/^(\w+) = "([^"]+)"/gm)].map((m) => [m[1], m[2]]));
      const key = values as DeviceKey;
      keys[user.name] = key;
      const id = `${user.name}@localhost`;
      seed += `[[user]]\nid = "${id}"\ncompartments = [${user.compartments.map((c) => `"${c}"`).join(", ")}]\n\n`;
      seed += `[[device]]\nid = "${key.id}"\nuser = "${id}"\npublic_key = "${key.public_key}"\n\n`;
    }
    fs.writeFileSync(path.join(this.work, "seed.toml"), seed);
    return keys;
  }

  async start(): Promise<void> {
    const log = fs.openSync(path.join(this.work, "server.log"), "a");
    this.proc = spawn(ORDO, ["serve"], {
      env: {
        ...process.env,
        ORDO_SERVER_NAME: "localhost",
        ORDO_LISTEN: `127.0.0.1:${this.port}`,
        ORDO_DATA_DIR: path.join(this.work, "data"),
        ORDO_MASTER_KEY_FILE: path.join(this.work, "master.key"),
        ORDO_SEED_FILE: path.join(this.work, "seed.toml"),
        ORDO_APP_DIR: DIST,
      },
      stdio: ["ignore", log, log],
    });
    const end = Date.now() + 20_000;
    while (Date.now() < end) {
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
