// A camera photo of a card shown on a screen, made in the browser from a picture of the card
// (ADR-0073): the card among lines of text on a screen whose pixels have dark gaps, turned
// and shrunk to camera size (which turns the gaps into moiré), with sensor noise, as a JPEG
// of 3900 × 2925 pixels. Made at run time, so no large photo is kept in the repository.

import type { Page } from "@playwright/test";

export async function screenPhoto(page: Page, card: Buffer): Promise<Buffer> {
  const jpeg = await page.evaluate(async (png: string) => {
    const image = new Image();
    image.src = `data:image/png;base64,${png}`;
    await image.decode();
    const canvas = (w: number, h: number) => {
      const c = document.createElement("canvas");
      c.width = w;
      c.height = h;
      return [c, c.getContext("2d")!] as const;
    };
    // The screen: 1000 × 750 pixels, the card at a third (2 screen pixels per module).
    const [screen, s] = canvas(1000, 750);
    s.fillStyle = "#e6e6e6";
    s.fillRect(0, 0, 1000, 750);
    s.fillStyle = "#2a2a2a";
    s.font = "14px sans-serif";
    for (let y = 30; y < 740; y += 24) s.fillText("Meldekarte · Name · Anschrift · Telefon · Einheit", 20, y);
    s.imageSmoothingEnabled = false;
    s.drawImage(image, 600, 250, image.width / 3, image.height / 3);
    // Each screen pixel 4 × 4, with a dark gap on its right and bottom.
    const k = 4;
    const [fine, f] = canvas(1000 * k, 750 * k);
    f.imageSmoothingEnabled = false;
    f.drawImage(screen, 0, 0, 1000 * k, 750 * k);
    f.fillStyle = "rgba(0, 0, 0, 0.55)";
    for (let x = 0; x < 1000; x++) f.fillRect(x * k + k - 1, 0, 1, 750 * k);
    for (let y = 0; y < 750; y++) f.fillRect(0, y * k + k - 1, 1000 * k, 1);
    // The camera: turned by 5°, 3 camera pixels per screen pixel, the screen's frame around.
    const [photo, p] = canvas(3900, 2925);
    p.fillStyle = "#101010";
    p.fillRect(0, 0, 3900, 2925);
    p.translate(1950, 1462);
    p.rotate((5 * Math.PI) / 180);
    p.scale(3 / k, 3 / k);
    p.drawImage(fine, -500 * k, -375 * k);
    p.setTransform(1, 0, 0, 1, 0, 0);
    // Uneven light and sensor noise (a fixed sequence, so every run is the same).
    const data = p.getImageData(0, 0, 3900, 2925);
    let state = 7;
    for (let i = 0; i < data.data.length; i += 4) {
      state = (Math.imul(state, 1103515245) + 12345) >>> 0;
      const light = 0.75 + 0.25 * (((i / 4) % 3900) / 3900);
      const grain = ((state >>> 16) % 25) - 12;
      for (let c = 0; c < 3; c++) data.data[i + c] = data.data[i + c] * light + grain;
    }
    p.putImageData(data, 0, 0);
    const blob: Blob = await new Promise((resolve) => photo.toBlob((b) => resolve(b!), "image/jpeg", 0.85));
    const bytes = new Uint8Array(await blob.arrayBuffer());
    let binary = "";
    for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
    return btoa(binary);
  }, card.toString("base64"));
  return Buffer.from(jpeg, "base64");
}
