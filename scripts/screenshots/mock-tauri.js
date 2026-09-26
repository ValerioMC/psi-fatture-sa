// Injected before the app loads: a fake Tauri IPC backed by deterministic demo data.
(() => {
  let seed = 42
  const rand = () => { seed = (seed * 1664525 + 1013904223) % 4294967296; return seed / 4294967296 }
  const pick = (arr) => arr[Math.floor(rand() * arr.length)]
  const pad = (n) => String(n).padStart(2, '0')
  const iso = (y, m, d) => `${y}-${pad(m)}-${pad(d)}`
  const TODAY = new Date()
  const TY = TODAY.getFullYear(), TM = TODAY.getMonth() + 1, TD = TODAY.getDate()
  const todayIso = iso(TY, TM, TD)
  const addDays = (s, n) => { const [y, m, d] = s.split('-').map(Number); const dt = new Date(y, m - 1, d + n); return iso(dt.getFullYear(), dt.getMonth() + 1, dt.getDate()) }

  const firstF = ['Giulia','Chiara','Francesca','Sara','Martina','Elena','Valentina','Alessia','Federica','Laura','Silvia','Anna','Beatrice','Irene','Camilla','Marta','Paola','Elisa','Serena','Noemi']
  const firstM = ['Marco','Luca','Andrea','Matteo','Alessandro','Davide','Simone','Federico','Lorenzo','Riccardo','Stefano','Giorgio','Paolo','Tommaso','Filippo']
  const lasts = ['Rossi','Bianchi','Esposito','Romano','Colombo','Ricci','Marino','Greco','Bruno','Gallo','Conti','De Luca','Mancini','Costa','Giordano','Rizzo','Lombardi','Moretti','Barbieri','Fontana','Santoro','Mariani','Rinaldi','Caruso','Ferrara','Galli','Martini','Leone','Longo','Gentile','Martinelli','Vitale','Serra','Coppola','De Santis','D\'Angelo','Marchetti','Parisi','Villa','Conte','Ferraro','Ferri','Fabbri','Bianco','Marini','Grasso','Valentini','Messina','Sala','De Angelis']
  const cities = [['Milano','MI','20121'],['Milano','MI','20135'],['Monza','MB','20900'],['Sesto San Giovanni','MI','20099'],['Bergamo','BG','24121'],['Como','CO','22100'],['Milano','MI','20144']]
  const streets = ['Via Solferino','Corso Buenos Aires','Via Tortona','Viale Monza','Via Paolo Sarpi','Corso di Porta Romana','Via Vigevano','Via Padova','Via Savona','Via Washington']

  const clients = []
  for (let i = 0; i < 50; i++) {
    const female = rand() < 0.62
    const first = female ? pick(firstF) : pick(firstM)
    const last = lasts[i]
    const [city, prov, zip] = pick(cities)
    const by = 1958 + Math.floor(rand() * 45)
    clients.push({
      id: i + 1, client_type: i === 17 ? 'azienda' : 'persona_fisica',
      first_name: i === 17 ? '' : first, last_name: i === 17 ? 'Studio Associato Hera' : last,
      birth_date: i === 17 ? undefined : iso(by, 1 + Math.floor(rand() * 12), 1 + Math.floor(rand() * 27)),
      gender: female ? 'F' : 'M',
      fiscal_code: (last.replace(/[^A-Z]/gi, '').toUpperCase() + 'XXX').slice(0, 3) + (first.toUpperCase() + 'XXX').slice(0, 3) + String(by).slice(2) + 'ABCDEHLMPRST'[Math.floor(rand() * 12)] + pad(1 + Math.floor(rand() * 28)) + 'F205' + 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'[Math.floor(rand() * 26)],
      vat_number: i === 17 ? '09876543210' : undefined,
      address: `${pick(streets)} ${1 + Math.floor(rand() * 90)}`, city, province: prov, zip_code: zip,
      email: rand() < 0.8 ? `${first.toLowerCase()}.${last.toLowerCase().replace(/[^a-z]/g, '')}@email.it` : undefined,
      phone: `+39 3${Math.floor(rand() * 5) + 3}${Math.floor(1000000 + rand() * 8999999)}`,
      notes: '', sts_authorization: rand() < 0.8, created_at: '2025-01-10T09:00:00', updated_at: '2025-01-10T09:00:00',
    })
  }
  const clientName = (c) => c.client_type === 'azienda' ? c.last_name : `${c.last_name} ${c.first_name}`

  const services = [
    ['Colloquio psicologico individuale', 'Seduta di 50 minuti', 80],
    ['Psicoterapia individuale', 'Seduta di 50 minuti', 90],
    ['Psicoterapia di coppia', 'Seduta di 75 minuti', 120],
    ['Primo colloquio conoscitivo', 'Valutazione iniziale, 60 minuti', 70],
    ['Seduta psicoanalitica', 'Setting analitico, 45 minuti', 95],
    ['Sostegno genitoriale', 'Incontro con i genitori', 85],
    ['Valutazione psicodiagnostica', 'Somministrazione e restituzione test', 110],
    ['Consulenza online', 'Videochiamata, 50 minuti', 75],
    ['Supervisione clinica', 'Per colleghi, 60 minuti', 100],
    ['Relazione clinica', 'Stesura su richiesta', 120],
  ].map(([name, description, price], i) => ({ id: i + 1, name, description, default_price: price, vat_rate: 0, is_active: i !== 9, created_at: '2025-01-01', updated_at: '2025-01-01' }))

  const config = {
    id: 1, title: 'Dott.ssa', first_name: 'Maria', last_name: 'Ferretti', vat_number: '12345678903', fiscal_code: 'FRRMRA80A41F205Z',
    tax_regime: 'forfettario', albo_number: '12345', albo_region: 'Lombardia', address: 'Via Solferino 24', city: 'Milano', province: 'MI', zip_code: '20121', country: 'IT',
    phone: '+39 02 1234567', pec_email: 'maria.ferretti@pec.it', iban: 'IT60X0542811101000000123456', coefficient: 78, profession: 'psicoterapeuta', is_psicoanalista: false,
    specialization: '', hide_quantity_in_invoice: false,
    initial_invoice_number: 1, created_at: '2025-01-01', updated_at: '2025-01-01',
  }

  const round2 = (v) => Math.round(v * 100) / 100
  const totals = (lines, enpap) => {
    const net = round2(lines.reduce((s, l) => s + l.quantity * l.unit_price, 0))
    const c = enpap ? round2(net * 0.02) : 0
    const bollo = net > 77.47
    return { total_net: net, total_tax: 0, contributo_enpap: c, ritenuta_acconto: 0, marca_da_bollo: bollo, total_gross: round2(net + c), total_due: round2(net + c + (bollo ? 2 : 0)) }
  }

  const invoices = []
  let nextId = 1
  const counters = {}
  const weekly = clients.slice(0, 44)
  for (let y = TY - 1; y <= TY; y++) {
    const lastMonth = y === TY ? TM - 1 : 12
    for (let m = 1; m <= lastMonth; m++) {
      if (m === 8 && rand() < 0.9) { /* August: lighter */ }
      for (const c of weekly) {
        if (rand() < (m === 8 ? 0.6 : 0.38)) continue
        const svc = services[c.id % 4 === 0 ? 2 : c.id % 5 === 0 ? 4 : 1]
        const qty = m === 8 ? 1 + Math.floor(rand() * 2) : 2 + Math.floor(rand() * 3)
        const lines = [{ id: nextId * 10, invoice_id: nextId, service_id: svc.id, description: `${svc.name} — ${['gennaio','febbraio','marzo','aprile','maggio','giugno','luglio','agosto','settembre','ottobre','novembre','dicembre'][m - 1]} ${y}`, quantity: qty, unit_price: svc.default_price, vat_rate: 0, line_total: qty * svc.default_price }]
        counters[y] = (counters[y] ?? 0) + 1
        const lastDay = new Date(y, m, 0).getDate()
        const issue = iso(y, m, lastDay)
        const due = addDays(issue, 30)
        let status = 'paid'
        const recent = y === TY && m >= TM - 1
        if (recent) status = rand() < 0.35 ? 'paid' : 'issued'
        if (y === TY && m === TM - 2 && rand() < 0.08) status = 'issued'
        const t = totals(lines, true)
        invoices.push({
          id: nextId, client_id: c.id, client_name: clientName(c), invoice_number: String(counters[y]), year: y, issue_date: issue, due_date: due,
          status, payment_method: rand() < 0.8 ? 'bonifico' : 'pos', notes: '', apply_enpap: true, ...t,
          paid_date: status === 'paid' ? addDays(issue, Math.floor(rand() * 20)) : undefined, lines, created_at: issue, updated_at: issue,
        })
        nextId++
      }
    }
  }
  // A few drafts this month and one cancelled.
  for (const c of clients.slice(44, 48)) {
    const svc = services[0]
    const lines = [{ id: nextId * 10, service_id: 1, description: svc.name, quantity: 1, unit_price: 80, vat_rate: 0, line_total: 80 }]
    counters[TY] = (counters[TY] ?? 0) + 1
    invoices.push({ id: nextId, client_id: c.id, client_name: clientName(c), invoice_number: String(counters[TY]), year: TY, issue_date: iso(TY, TM, Math.max(1, TD - 3)), due_date: undefined,
      status: c.id === 47 ? 'cancelled' : 'draft', payment_method: 'bonifico', notes: '', apply_enpap: true, ...totals(lines, true), lines, created_at: todayIso, updated_at: todayIso })
    nextId++
  }

  // Appointments for last month, this month and next.
  const appointments = []
  let apptId = 1
  const slots = ['08:30','09:30','10:30','11:30','14:00','15:00','16:00','17:00','18:00','19:00']
  for (let offset = -1; offset <= 1; offset++) {
    const dt = new Date(TY, TM - 1 + offset, 1)
    const y = dt.getFullYear(), m = dt.getMonth() + 1
    const days = new Date(y, m, 0).getDate()
    for (let d = 1; d <= days; d++) {
      const dow = new Date(y, m - 1, d).getDay()
      if (dow === 0 || dow === 6) continue
      if (m === 8 && d > 7 && d < 25) continue
      const n = 3 + Math.floor(rand() * 5)
      const used = new Set()
      for (let k = 0; k < n; k++) {
        let s = pick(slots); if (used.has(s)) continue; used.add(s)
        const c = pick(weekly), svc = services[c.id % 4 === 0 ? 2 : 1]
        const date = iso(y, m, d)
        const [hh, mm] = s.split(':').map(Number)
        const endMin = hh * 60 + mm + (svc.id === 3 ? 75 : 50)
        const status = date < todayIso ? (rand() < 0.9 ? 'completed' : 'cancelled') : 'scheduled'
        appointments.push({ id: apptId++, client_id: c.id, client_name: clientName(c), service_id: svc.id, service_name: svc.name, date, start_time: s, end_time: `${pad(Math.floor(endMin / 60))}:${pad(endMin % 60)}`,
          status, notes: rand() < 0.1 ? 'Portare referto' : '', invoice_id: offset < 0 && status === 'completed' && rand() < 0.2 ? 1 : undefined, created_at: date, updated_at: date })
      }
    }
  }

  const MONTHS = ['Gennaio','Febbraio','Marzo','Aprile','Maggio','Giugno','Luglio','Agosto','Settembre','Ottobre','Novembre','Dicembre']
  const clone = (x) => JSON.parse(JSON.stringify(x))
  // ── Sistema TS: a simulated queue and remote store over the demo invoices ──
  const stamp = (date, time = '10:15:00') => `${date} ${time}`
  const tsSettings = { environment: window.__MOCK_TS_TEST ? 'test' : 'produzione', username: 'FRRMRA80A41F205Z', vat_number: '12345678903' }
  const tsCreds = { password_configured: !window.__MOCK_NO_TS_CREDENTIALS, pincode_configured: !window.__MOCK_NO_TS_CREDENTIALS }
  const tsSubs = []
  let tsId = 1
  // Protocols are 17 digits: past 2^53, so they are built as strings.
  let tsProtocolSeq = 100000
  const nextProtocol = () => `99260101000${tsProtocolSeq++}`
  const tsInvoiceView = (inv) => ({ invoice_number: inv.invoice_number, invoice_year: inv.year, client_name: inv.client_name })
  const tsNew = (inv, operation, status, extra = {}) => {
    const sub = { id: tsId++, invoice_id: inv.id, ...tsInvoiceView(inv), operation, status, target_submission_id: null, environment: tsSettings.environment,
      document: { vat_number: tsSettings.vat_number, issue_date: inv.issue_date, number: inv.invoice_number }, protocol: null, outcome_code: null, outcome_message: null,
      attempt_count: 0, last_error: null, next_attempt_at: stamp(todayIso), last_attempt_at: null, sent_at: null, resolved_at: null, created_at: stamp(todayIso), updated_at: stamp(todayIso), ...extra }
    tsSubs.push(sub)
    return sub
  }
  const tsAccept = (sub, date) => Object.assign(sub, { status: 'accettata', protocol: nextProtocol(), attempt_count: sub.attempt_count + 1, sent_at: stamp(date), resolved_at: stamp(date), last_attempt_at: stamp(date) })
  const seedTs = () => {
    const paid = invoices.filter((i) => i.status === 'paid' && i.paid_date).sort((a, b) => a.paid_date.localeCompare(b.paid_date))
    const cutoff = addDays(todayIso, -40)
    paid.forEach((inv, index) => {
      const client = clients.find((c) => c.id === inv.client_id)
      if (inv.paid_date > cutoff) return
      if (client?.client_type !== 'persona_fisica') return
      const sub = tsNew(inv, 'invio', 'non_inviata', { created_at: stamp(inv.paid_date) })
      if (index % 90 === 5) Object.assign(sub, { status: 'scartata', attempt_count: 1, outcome_code: 'WS19', outcome_message: 'WS19 CF DEL CITTADINO NON VALIDO', resolved_at: stamp(addDays(inv.paid_date, 2)) })
      else tsAccept(sub, addDays(inv.paid_date, 2))
    })
    const recent = paid.filter((i) => i.paid_date > cutoff)
    if (recent[0]) tsNew(recent[0], 'invio', 'non_inviata', { attempt_count: 2, last_error: 'Sistema TS non raggiungibile: operation timed out', next_attempt_at: stamp(todayIso, '23:59:00') })
  }
  seedTs()
  const tsLive = (invoiceId) => [...tsSubs].reverse().find((s) => s.invoice_id === invoiceId && s.status === 'accettata' && s.operation !== 'annullamento' && s.environment === tsSettings.environment)
  const tsTotal = (inv) => inv.total_gross
  const handlers = {
    get_config: () => (window.__MOCK_NO_CONFIG ? null : config),
    get_ts_settings: () => tsSettings,
    update_ts_settings: ({ input }) => Object.assign(tsSettings, { ...input, username: input.username.trim().toUpperCase() }),
    get_ts_credentials_status: () => tsCreds,
    save_ts_pincode: () => Object.assign(tsCreds, { pincode_configured: true }),
    delete_ts_pincode: () => Object.assign(tsCreds, { pincode_configured: false }),
    save_ts_password: () => Object.assign(tsCreds, { password_configured: true }),
    delete_ts_password: () => Object.assign(tsCreds, { password_configured: false }),
    check_ts_connection: () => tsCreds.password_configured && tsCreds.pincode_configured
      ? { ok: true, message: `Credenziali accettate dal Sistema TS (${tsSettings.environment})` }
      : { ok: false, message: 'Manca la password del Sistema TS: inseriscila nelle Impostazioni' },
    list_ts_submissions: ({ filters }) => tsSubs.filter((s) => !filters?.invoice_id || s.invoice_id === filters.invoice_id),
    enqueue_ts_submission: ({ invoiceId }) => {
      const inv = invoices.find((i) => i.id === invoiceId)
      if (inv.status !== 'paid') throw new Error('Solo le fatture pagate possono essere trasmesse al Sistema TS')
      return tsNew(inv, 'invio', 'non_inviata')
    },
    enqueue_ts_replacement: ({ submissionId }) => { const t = tsSubs.find((s) => s.id === submissionId); return tsNew(invoices.find((i) => i.id === t.invoice_id), 'sostituzione', 'non_inviata', { target_submission_id: t.id, document: t.document }) },
    enqueue_ts_cancellation: ({ submissionId }) => { const t = tsSubs.find((s) => s.id === submissionId); return tsNew(invoices.find((i) => i.id === t.invoice_id), 'annullamento', 'non_inviata', { target_submission_id: t.id, document: t.document }) },
    withdraw_ts_submission: ({ submissionId }) => { tsSubs.splice(tsSubs.findIndex((s) => s.id === submissionId), 1) },
    dispatch_ts_queue: () => {
      const summary = { accepted: 0, rejected: 0, retrying: 0, waiting_other_environment: 0, blocked: null }
      if (!tsCreds.password_configured || !tsCreds.pincode_configured) return { ...summary, blocked: 'Manca la password del Sistema TS: inseriscila nelle Impostazioni' }
      for (const sub of tsSubs.filter((s) => s.status === 'non_inviata')) {
        if (sub.environment !== tsSettings.environment) { summary.waiting_other_environment++; continue }
        tsAccept(sub, todayIso)
        sub.last_error = null
        const target = tsSubs.find((s) => s.id === sub.target_submission_id)
        if (target) target.status = sub.operation === 'annullamento' ? 'annullata' : 'sostituita'
        summary.accepted++
      }
      return summary
    },
    query_ts_invoice: ({ invoiceId }) => {
      const inv = invoices.find((i) => i.id === invoiceId)
      const all = tsSubs.filter((s) => s.invoice_id === invoiceId && s.status !== 'scartata' && s.protocol)
      const last = all[all.length - 1]
      if (!last) return { kind: 'not_found' }
      const cancelled = last.operation === 'annullamento'
      return { kind: 'found', document: { id: last.document, payment_date: inv.paid_date, totals: [{ expense_type: 'SP', amount: tsTotal(inv) }], refunded_totals: [], protocol: last.protocol,
        sent_date: last.sent_at.slice(0, 10), send_kind: last.operation === 'sostituzione' ? 'V' : 'I', cancelled, messages: cancelled ? [{ code: 'W010', description: "IL DOCUMENTO E' STATO ANNULLATO IN PRECEDENZA", kind: 'W' }] : [] } }
    },
    get_ts_monthly_report: ({ year, month, basis }) => {
      if (!tsCreds.password_configured) throw new Error('Manca la password del Sistema TS: inseriscila nelle Impostazioni')
      const prefix = `${year}-${pad(month)}`
      const rows = []
      for (const inv of invoices) {
        const live = tsLive(inv.id)
        if (!live) continue
        const day = basis === 'invio' ? live.sent_at.slice(0, 10) : inv.paid_date
        if (!day.startsWith(prefix)) continue
        rows.push({ vat_number: live.document.vat_number, issue_date: inv.issue_date, document_number: inv.invoice_number, payment_date: inv.paid_date, protocol: live.protocol,
          sent_date: live.sent_at.slice(0, 10), send_kind: live.operation === 'sostituzione' ? 'V' : 'I', amount: tsTotal(inv), refunded_amount: 0, invoice_id: inv.id })
      }
      if (rows.length > 3) rows.splice(2, 0, { ...rows[0], document_number: 'FT-0042', invoice_id: null, protocol: nextProtocol() })
      return rows
    },
    upsert_config: ({ input }) => Object.assign(config, input),
    list_clients: ({ search }) => clients.filter((c) => !search || (clientName(c) + c.fiscal_code).toLowerCase().includes(search.toLowerCase())).sort((a, b) => clientName(a).localeCompare(clientName(b))),
    get_client: ({ id }) => clients.find((c) => c.id === id),
    create_client: ({ input }) => { const c = { ...input, id: clients.length + 1 }; clients.push(c); return c },
    update_client: ({ input }) => { const i = clients.findIndex((c) => c.id === input.id); clients[i] = { ...clients[i], ...input }; return clients[i] },
    delete_client: ({ id }) => { clients.splice(clients.findIndex((c) => c.id === id), 1) },
    list_services: ({ activeOnly }) => services.filter((s) => !activeOnly || s.is_active),
    get_service: ({ id }) => services.find((s) => s.id === id),
    create_service: ({ input }) => { const s = { ...input, id: services.length + 1 }; services.push(s); return s },
    update_service: ({ input }) => { const i = services.findIndex((s) => s.id === input.id); services[i] = { ...services[i], ...input }; return services[i] },
    delete_service: ({ id }) => { services.splice(services.findIndex((s) => s.id === id), 1) },
    list_invoices: ({ filters }) => invoices.filter((i) => (!filters.year || i.year === filters.year) && (!filters.status || i.status === filters.status) && (!filters.client_id || i.client_id === filters.client_id) && (!filters.search || (i.client_name + i.invoice_number).toLowerCase().includes(filters.search.toLowerCase())))
      .sort((a, b) => b.issue_date.localeCompare(a.issue_date) || Number(b.invoice_number) - Number(a.invoice_number)),
    get_invoice: ({ id }) => invoices.find((i) => i.id === id),
    update_invoice: ({ input }) => { const i = invoices.findIndex((x) => x.id === input.id); invoices[i] = { ...invoices[i], ...input, lines: input.lines.map((l) => ({ ...l, line_total: l.quantity * l.unit_price })), ...totals(input.lines, input.apply_enpap) }; return invoices[i] },
    create_invoice: ({ input }) => { const inv = { ...input, id: nextId++, invoice_number: String(++counters[TY]), year: TY, client_name: clientName(clients.find((c) => c.id === input.client_id)), lines: input.lines.map((l) => ({ ...l, line_total: l.quantity * l.unit_price })), ...totals(input.lines, input.apply_enpap) }; invoices.push(inv); return inv },
    delete_invoice: ({ id }) => { invoices.splice(invoices.findIndex((i) => i.id === id), 1) },
    get_next_invoice_number: () => String((counters[TY] ?? 0) + 1),
    bulk_update_invoice_status: ({ input }) => { for (const i of invoices) if (input.ids.includes(i.id)) { i.status = input.status; i.paid_date = input.paid_date } return input.ids.length },
    preview_monthly_invoices: ({ year, month }) => {
      const prefix = `${year}-${pad(month)}`
      const by = {}
      for (const a of appointments) if (a.date.startsWith(prefix) && a.status === 'completed' && !a.invoice_id) (by[a.client_id] ??= []).push(a)
      return Object.entries(by).map(([cid, list]) => {
        const svc = services.find((s) => s.id === list[0].service_id)
        const lines = [{ service_id: svc.id, description: `${svc.name} — ${MONTHS[month - 1].toLowerCase()} ${year}`, quantity: list.length, unit_price: svc.default_price, vat_rate: 0 }]
        const t = totals(lines, true)
        return { client_id: Number(cid), client_name: list[0].client_name, appointment_count: list.length, lines, estimated_net: t.total_net, estimated_due: t.total_due }
      }).sort((a, b) => a.client_name.localeCompare(b.client_name))
    },
    generate_monthly_invoices: ({ input }) => input.client_ids.map((id) => ({ id })),
    list_appointments: ({ dateFrom, dateTo }) => appointments.filter((a) => (!dateFrom || a.date >= dateFrom) && (!dateTo || a.date <= dateTo)),
    get_appointment: ({ id }) => appointments.find((a) => a.id === id),
    create_appointment: ({ input }) => { const a = { ...input, id: apptId++, client_name: clientName(clients.find((c) => c.id === input.client_id)) }; appointments.push(a); return a },
    create_recurring_appointments: ({ input }) => input.dates.map((date) => handlers.create_appointment({ input: { ...input, date, status: 'scheduled' } })),
    update_appointment: ({ input }) => { const i = appointments.findIndex((a) => a.id === input.id); appointments[i] = { ...appointments[i], ...input }; return appointments[i] },
    delete_appointment: ({ id }) => { appointments.splice(appointments.findIndex((a) => a.id === id), 1) },
    get_dashboard: ({ year }) => {
      const list = invoices.filter((i) => i.year === year && i.status !== 'cancelled')
      const sum = (arr, k) => round2(arr.reduce((s, i) => s + i[k], 0))
      const paid = list.filter((i) => i.status === 'paid')
      return {
        year, total_revenue: sum(list, 'total_due'), total_net_revenue: sum(list, 'total_net'), paid_revenue: sum(paid, 'total_due'),
        unpaid_revenue: sum(list.filter((i) => i.status === 'issued' || i.status === 'overdue'), 'total_due'),
        total_invoices: list.length, paid_invoices: paid.length, draft_invoices: list.filter((i) => i.status === 'draft').length,
        monthly_revenue: MONTHS.map((name, idx) => { const ms = paid.filter((i) => Number(i.issue_date.slice(5, 7)) === idx + 1); return { month: idx + 1, month_name: name, revenue: sum(ms, 'total_due'), invoice_count: ms.length } }),
        recent_invoices: [...list].sort((a, b) => b.issue_date.localeCompare(a.issue_date)).slice(0, 5),
      }
    },
    print_current_page: () => null,
  }

  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args = {}) => {
      const handler = handlers[cmd]
      if (!handler) { console.warn('mock: unhandled', cmd); throw new Error(`mock: ${cmd}`) }
      await new Promise((r) => setTimeout(r, 30))
      return clone(handler(args) ?? null)
    },
    transformCallback: () => 0,
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
  }
})()
