// Starts Dex (https://dexidp.io) in Docker as the identity provider of the browser tests
// (ADR-0038). Set ORDO_E2E_DEX=external to use a Dex that is already running. While the
// local demo runs (Dex on 5556, ORDO on 18090), set ORDO_E2E_DEX_PORT and ORDO_E2E_PORT.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const PORT = process.env.ORDO_E2E_DEX_PORT ?? "5556";
/** Dex's gRPC API for local accounts (ADR-0060): the next port. */
const API_PORT = String(Number(PORT) + 1);
/** The first ORDO test server's port; Dex sends logins back there. */
export const ORDO_PORT = Number(process.env.ORDO_E2E_PORT ?? 18090);
export const DEX_URL = `http://127.0.0.1:${PORT}/dex`;
const IMAGE = "ghcr.io/dexidp/dex:v2.46.1";
const NAME = "ordo-e2e-dex";

/** Dex's API for ORDO: address and the directory with the certificates of
 *  `ordo generate-dex-certs` (made by `startDex`; with ORDO_E2E_DEX=external, from
 *  ORDO_E2E_DEX_CERTS). */
export const DEX_API = {
  url: `https://127.0.0.1:${API_PORT}`,
  certs: process.env.ORDO_E2E_DEX_CERTS ?? fs.mkdtempSync(path.join(os.tmpdir(), "ordo-dex-certs-")),
};

/** support/dex.yaml, moved to other ports if they are asked for. */
function config(): string {
  const file = path.join(__dirname, "dex.yaml");
  if (PORT === "5556" && ORDO_PORT === 18090) return file;
  const moved = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "ordo-dex-")), "dex.yaml");
  const text = fs
    .readFileSync(file, "utf8")
    .replaceAll(":5557", `:${API_PORT}`)
    .replaceAll(":5556", `:${PORT}`)
    .replaceAll(":18090/", `:${ORDO_PORT}/`);
  fs.writeFileSync(moved, text);
  return moved;
}

/** Starts Dex and waits until it answers; returns a function that stops it. */
export async function startDex(): Promise<() => void> {
  if (process.env.ORDO_E2E_DEX !== "external") {
    try {
      execFileSync("docker", ["rm", "-f", NAME], { stdio: "ignore" });
    } catch {
      // not running
    }
    // The certificates of Dex's API, made as an operator would (ADR-0060).
    const ordo = path.resolve(process.env.ORDO_BIN ?? path.join(__dirname, "../../../target/debug/ordo"));
    execFileSync(ordo, ["generate-dex-certs", DEX_API.certs, "127.0.0.1", "localhost"]);
    // Dex runs as its own user in the container; a temporary directory is private.
    fs.chmodSync(DEX_API.certs, 0o755);
    execFileSync("docker", ["run", "-d", "--name", NAME, "-p", `${PORT}:${PORT}`, "-p", `${API_PORT}:${API_PORT}`,
      "-v", `${config()}:/etc/dex/config.yaml:ro,Z`, "-v", `${DEX_API.certs}:/etc/dex/certs:ro,Z`,
      IMAGE, "dex", "serve", "/etc/dex/config.yaml"]);
  }
  const end = Date.now() + 60_000;
  while (Date.now() < end) {
    try {
      if ((await fetch(`${DEX_URL}/.well-known/openid-configuration`)).ok) break;
    } catch {
      // not up yet
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  return () => {
    if (process.env.ORDO_E2E_DEX !== "external") execFileSync("docker", ["rm", "-f", NAME], { stdio: "ignore" });
  };
}
