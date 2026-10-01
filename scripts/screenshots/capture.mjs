/**
 * Captures the brochure screenshots of psifatture.it from the running Vite dev
 * server, with the Tauri IPC replaced by deterministic demo data (mock-tauri.js).
 * No native window is driven: macOS permission prompts are never triggered.
 *
 *   npm run dev -- --port 1421                              # in another shell
 *   npm install --no-save playwright sharp
 *   node scripts/screenshots/capture.mjs ../psi-fatture-brochure/public/screenshots
 *
 * ONLY=shots or ONLY=excerpts limits the run to the full screens or to the cropped excerpts.
 * NAMES=soglia,stima limits the excerpts to the named ones.
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

/** CSS pixels of app background kept around a cropped element. */
const MARGIN = 20

/** A paid demo invoice already registered on the Sistema TS and already emailed to the patient. */
const demoInvoiceRoute = async (page) => {
  const id = await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke
    const accepted = (await invoke('list_ts_submissions', { filters: {} })).filter((s) => s.status === 'accettata')
    const emailed = new Set((await invoke('list_invoice_emails', { filters: {} })).map((e) => e.invoice_id))
    return accepted.map((s) => s.invoice_id).filter((invoiceId) => emailed.has(invoiceId)).at(-1)
  })
  return `/invoices/${id}`
}

const padded = (rect) => ({ x: rect.x - MARGIN, y: rect.y - MARGIN, width: rect.width + 2 * MARGIN, height: rect.height + 2 * MARGIN })

/** Excerpts: one region of a screen each, for the feature sections of the site. */
const EXCERPTS = [
  {
    // The Sistema TS queue with three invoices selected, down to the floating send bar.
    name: 'sts-coda',
    route: async () => '/sts',
    viewport: { width: 1200, height: 760 },
    prepare: async (page) => {
      for (const box of (await page.locator('tbody input[type=checkbox]:not([disabled])').all()).slice(0, 3)) await box.check()
      await page.waitForTimeout(500)
    },
    region: (page) => page.evaluate(() => {
      const main = document.querySelector('main').getBoundingClientRect()
      const top = document.querySelector('[aria-label="Anno delle spese"]').getBoundingClientRect().top - 12
      return { x: main.left, y: top, width: main.width, height: window.innerHeight - top }
    }),
  },
  {
    name: 'sts-fattura',
    route: demoInvoiceRoute,
    viewport: { width: 1440, height: 1300 },
    region: async (page) => padded(await page.evaluate(() => {
      const label = [...document.querySelectorAll('.label-quiet')].find((p) => p.textContent.trim() === 'Sistema Tessera Sanitaria')
      return label.closest('.sheet').getBoundingClientRect().toJSON()
    })),
  },
  {
    name: 'email-invio',
    route: demoInvoiceRoute,
    viewport: { width: 1440, height: 900 },
    prepare: async (page) => {
      await page.getByRole('button', { name: 'Invia di nuovo' }).click()
      await page.locator('.dialog-panel textarea').waitFor()
      // The crop keeps a margin around the dialog: plain app canvas there, not the blurred screen behind.
      await page.locator('.dialog-panel').evaluate((panel) => {
        Object.assign(panel.previousElementSibling.style, { background: getComputedStyle(document.body).backgroundColor, backdropFilter: 'none' })
      })
      await page.waitForTimeout(600)
    },
    region: async (page) => padded(await page.locator('.dialog-panel').boundingBox()),
  },
  {
    // The provider tiles and the psypec.it note, without the rest of the settings card.
    name: 'email-casella',
    route: async () => '/settings/email',
    viewport: { width: 1440, height: 900 },
    region: async (page) => padded(await page.evaluate(() => {
      const card = document.querySelector('[aria-label="Provider della casella"]').closest('.sheet').getBoundingClientRect()
      const note = document.querySelector('[aria-label="Provider della casella"]').closest('.sheet').querySelector('[class*="bg-surface-sunken"]').getBoundingClientRect()
      return { x: card.x, y: card.y, width: card.width, height: note.bottom - card.y }
    })),
  },
  {
    // The invoice form's live summary, with the regime note under it.
    name: 'fattura-riepilogo',
    route: async () => '/invoices/180/edit',
    viewport: { width: 1440, height: 900 },
    region: async (page) => padded(await page.evaluate(() =>
      [...document.querySelectorAll('aside')].find((aside) => aside.textContent.includes('Riepilogo')).getBoundingClientRect().toJSON())),
  },
  {
    name: 'agenda-giorno',
    route: async () => '/agenda',
    viewport: { width: 1440, height: 900 },
    region: async (page) => padded(await page.evaluate(() => document.querySelector('main aside .sheet').getBoundingClientRect().toJSON())),
  },
  {
    // The monthly run's total and its button, without the payment options above them.
    name: 'mensile-totale',
    route: async () => '/invoices/monthly',
    viewport: { width: 1440, height: 900 },
    // The crop starts under the divider, so the card runs off the top edge of the excerpt.
    region: async (page) => padded(await page.evaluate(() => {
      const card = document.querySelector('main aside .sheet').getBoundingClientRect()
      const total = document.querySelector('main aside .sheet .border-t').getBoundingClientRect()
      return { x: card.x, y: total.top + 1, width: card.width, height: card.bottom - total.top - 1 }
    })),
  },
  {
    name: 'soglia',
    route: async () => '/dashboard',
    viewport: { width: 1440, height: 1000 },
    region: async (page) => padded(await page.evaluate(() =>
      [...document.querySelectorAll('h2, h3, p')].find((node) => node.textContent.trim() === 'Soglia forfettario' && node.closest('main')).closest('.sheet').getBoundingClientRect().toJSON())),
  },
  {
    name: 'stima',
    route: async () => '/dashboard',
    viewport: { width: 1440, height: 1000 },
    region: async (page) => padded(await page.evaluate(() =>
      [...document.querySelectorAll('h2, h3, p')].find((node) => node.textContent.trim() === 'Stima fiscale' && node.closest('main')).closest('.sheet').getBoundingClientRect().toJSON())),
  },
  {
    name: 'paziente-scheda',
    route: async () => '/clients/5/edit',
    viewport: { width: 1440, height: 900 },
    region: async (page) => padded(await page.evaluate(() => document.querySelector('.record-card').getBoundingClientRect().toJSON())),
  },
  {
    // First run, step two: the first step done, the regime chosen.
    name: 'onboarding-fisco',
    route: async () => '/onboarding',
    viewport: { width: 1440, height: 900 },
    prepare: async (page) => {
      await page.getByRole('textbox', { name: 'Nome', exact: true }).fill('Maria')
      await page.getByRole('textbox', { name: 'Cognome', exact: true }).fill('Ferretti')
      await page.getByRole('textbox', { name: 'Partita IVA', exact: true }).fill('12345678903')
      await page.getByRole('textbox', { name: 'Codice fiscale', exact: true }).fill('FRRMRA80A41F205B')
      await page.getByRole('button', { name: 'Avanti' }).click()
      await page.waitForTimeout(800)
    },
    region: async (page) => padded(await page.evaluate(() => {
      const steps = document.querySelector('[aria-label="Passaggi"]').getBoundingClientRect()
      const form = document.querySelector('main form').getBoundingClientRect()
      const section = document.querySelector('main form section > div:last-child').getBoundingClientRect()
      return { x: steps.left, y: steps.top, width: form.width, height: section.bottom - steps.top }
    })),
  },
]

const only = process.env.ONLY
const browser = await chromium.launch()
for (const shot of only === 'excerpts' ? [] : SHOTS) {
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

const names = process.env.NAMES?.split(',')
for (const excerpt of only === 'shots' ? [] : EXCERPTS.filter((item) => !names || names.includes(item.name))) {
  for (const theme of ['light', 'dark']) {
    const context = await browser.newContext({ viewport: excerpt.viewport, deviceScaleFactor: 2, colorScheme: theme, reducedMotion: 'reduce' })
    await context.addInitScript(readFileSync(join(here, 'mock-tauri.js'), 'utf8'))
    await context.addInitScript(`try { localStorage.setItem('psi-fatture.theme', '${theme}') } catch {}`)
    const page = await context.newPage()
    await page.goto(`${base}/#/dashboard`)
    await page.waitForTimeout(600)
    await page.goto(`${base}/#${await excerpt.route(page)}`)
    await page.waitForTimeout(1000)
    await excerpt.prepare?.(page)
    const png = await page.screenshot({ clip: await excerpt.region(page) })
    const name = theme === 'dark' ? `${excerpt.name}-dark` : excerpt.name
    await sharp(png).resize({ width: 1600, withoutEnlargement: true }).webp({ quality: 86, effort: 6 }).toFile(join(outDir, `${name}.webp`))
    console.log(`${name}.webp`)
    await context.close()
  }
}
await browser.close()
