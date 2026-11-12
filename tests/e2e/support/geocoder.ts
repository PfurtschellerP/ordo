// A stand-in for Nominatim (ADR-0066): answers /search like the public service, for one
// known address, so the address search runs without the internet.
import http from "node:http";
import type { AddressInfo } from "node:net";

export const PLACE = { label: "Rathausplatz, 86150 Augsburg, Bayern, Deutschland", lat: "48.3700", lon: "10.9000" };

/** Starts the stand-in; returns its base URL and a function that stops it. */
export async function startGeocoder(): Promise<{ url: string; stop: () => void }> {
  const server = http.createServer((request, response) => {
    const url = new URL(request.url ?? "/", "http://localhost");
    const q = url.searchParams.get("q") ?? "";
    const places = url.pathname === "/search" && q.includes("rathausplatz")
      ? [{ display_name: PLACE.label, lat: PLACE.lat, lon: PLACE.lon }]
      : [];
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify(places));
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address() as AddressInfo;
  return { url: `http://127.0.0.1:${port}`, stop: () => server.close() };
}
