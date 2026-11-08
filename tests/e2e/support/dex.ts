// Starts Dex (https://dexidp.io) in Docker as the identity provider of the browser tests
// (ADR-0038). Set ORDO_E2E_DEX=external to use a Dex that is already running.
import { execFileSync } from "node:child_process";
import path from "node:path";

export const DEX_URL = "http://127.0.0.1:5556/dex";
const IMAGE = "ghcr.io/dexidp/dex:v2.46.1";
const NAME = "ordo-e2e-dex";
const CONFIG = path.join(__dirname, "dex.yaml");

/** Starts Dex and waits until it answers; returns a function that stops it. */
export async function startDex(): Promise<() => void> {
  if (process.env.ORDO_E2E_DEX !== "external") {
    try {
      execFileSync("docker", ["rm", "-f", NAME], { stdio: "ignore" });
    } catch {
      // not running
    }
    execFileSync("docker", ["run", "-d", "--name", NAME, "-p", "5556:5556",
      "-v", `${CONFIG}:/etc/dex/config.yaml:ro,Z`, IMAGE, "dex", "serve", "/etc/dex/config.yaml"]);
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
