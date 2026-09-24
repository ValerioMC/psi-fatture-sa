/**
 * Captures the brochure screenshots of psifatture.it from the running Vite dev
 * server, with the Tauri IPC replaced by deterministic demo data (mock-tauri.js).
 * No native window is driven: macOS permission prompts are never triggered.
 *
 *   npm run dev -- --port 1421                              # in another shell
 *   npm install --no-save playwright sharp
 *   node scripts/screenshots/capture.mjs ../psi-fatture-brochure/public/screenshots
 */
import { chromium } from 'playwright'
import sharp from 'sharp'
import { mkdirSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const outDir = process.argv[2] ?? join(here, 'out')
const base = process.env.BASE ?? 'http://localhost:1421'
mkdirSync(outDir, { recursive: true })

/** The macOS overlay title bar draws the traffic lights over the sidebar; a headless browser has none. */
const TRAFFIC_LIGHTS = ".sidebar-top{position:relative}.sidebar-top::before{content:'';position:absolute;left:20px;top:20px;width:12px;height:12px;border-radius:50%;background:#ff5f57;box-shadow:20px 0 0 #febc2e,40px 0 0 #28c840}"

const SHOTS = [
  { name: 'dashboard', route: '/dashboard', theme: 'light' },
  { name: 'dashboard-dark', route: '/dashboard', theme: 'dark' },
  { name: 'fatture', route: '/invoices', theme: 'light' },
  { name: 'agenda', route: '/agenda', theme: 'light' },
  { name: 'pazienti', route: '/clients', theme: 'light' },
  { name: 'mensile', route: '/invoices/monthly', theme: 'light' },
  { name: 'pdf', route: '/invoices/180/print', theme: 'light' },
]

const browser = await chromium.launch()
for (const shot of SHOTS) {
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 2,
    colorScheme: shot.theme,
    reducedMotion: 'reduce',
  })
  await context.addInitScript(readFileSync(join(here, 'mock-tauri.js'), 'utf8'))
  await context.addInitScript(`try { localStorage.setItem('psi-fatture.theme', '${shot.theme}') } catch {}`)
  await context.addInitScript(`document.addEventListener('DOMContentLoaded', () => { const s = document.createElement('style'); s.textContent = ${JSON.stringify(TRAFFIC_LIGHTS)}; document.head.appendChild(s) })`)
  const page = await context.newPage()
  await page.goto(`${base}/#${shot.route}`)
  await page.waitForTimeout(1000)
  const png = await page.screenshot()
  await sharp(png).resize({ width: 1920 }).webp({ quality: 84, effort: 6 }).toFile(join(outDir, `${shot.name}.webp`))
  console.log(`${shot.name}.webp`)
  await context.close()
}
await browser.close()
