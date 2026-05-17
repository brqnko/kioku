// One-shot script to generate raster assets (favicon.ico, og-image.png) from
// inline SVG sources. Run with `node scripts/generate-assets.mjs` from the
// frontend directory. Re-run when the brand changes.
import { promises as fs } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const publicDir = path.resolve(__dirname, "..", "public");

const FAVICON_SVG = `
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
  <rect width="64" height="64" rx="14" fill="#1a1a1a"/>
  <text x="32" y="44" font-family="system-ui,-apple-system,Segoe UI,sans-serif"
    font-size="40" font-weight="800" fill="#ffffff" text-anchor="middle">k</text>
</svg>`;

const OG_SVG = `
<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#1a1a1a"/>
      <stop offset="100%" stop-color="#262626"/>
    </linearGradient>
    <radialGradient id="glow" cx="0.18" cy="0.5" r="0.6">
      <stop offset="0%" stop-color="#f97316" stop-opacity="0.25"/>
      <stop offset="100%" stop-color="#f97316" stop-opacity="0"/>
    </radialGradient>
  </defs>
  <rect width="1200" height="630" fill="url(#bg)"/>
  <rect width="1200" height="630" fill="url(#glow)"/>
  <g transform="translate(96 200)">
    <rect width="120" height="120" rx="26" fill="#f97316"/>
    <text x="60" y="86" font-family="system-ui,-apple-system,Segoe UI,sans-serif"
      font-size="84" font-weight="800" fill="#1a1a1a" text-anchor="middle">k</text>
  </g>
  <text x="96" y="430" font-family="system-ui,-apple-system,Segoe UI,sans-serif"
    font-size="72" font-weight="800" fill="#ffffff" letter-spacing="-2">kioku</text>
  <text x="96" y="500" font-family="system-ui,-apple-system,Segoe UI,sans-serif"
    font-size="36" font-weight="500" fill="#a3a3a3">Folder structure for NotebookLM.</text>
</svg>`;

async function writeOg() {
  const out = path.join(publicDir, "og-image.png");
  await sharp(Buffer.from(OG_SVG)).png({ compressionLevel: 9 }).toFile(out);
  console.log("wrote", out);
}

async function writeFavicon() {
  // sharp cannot emit ICO directly; emit a 32x32 PNG and save it under the
  // .ico filename — every browser that still asks for /favicon.ico accepts
  // PNG-encoded content there.
  const out = path.join(publicDir, "favicon.ico");
  await sharp(Buffer.from(FAVICON_SVG))
    .resize(32, 32)
    .png({ compressionLevel: 9 })
    .toFile(out);
  console.log("wrote", out);
}

await writeOg();
await writeFavicon();
await fs.access(publicDir);
