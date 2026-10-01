/**
 * The terms of use accepted before the app opens. Any change to the text needs a new
 * TERMS_VERSION: the app then asks every user to accept again, and the backend records
 * each version accepted.
 */

export const TERMS_VERSION = '2026-10-01'

export interface TermsClause {
  readonly number: number
  readonly title: string
  readonly paragraphs: readonly string[]
}

export const TERMS_CLAUSES: readonly TermsClause[] = [
  {
    number: 1,
    title: 'Oggetto',
    paragraphs: [
      "PSI Fatture è un'app desktop gratuita per la fatturazione di psicologi e psicoterapeuti. Il codice è pubblicato con licenza MIT. Queste condizioni regolano l'uso dell'app da parte tua.",
    ],
  },
  {
    number: 2,
    title: 'Fornitura senza garanzia',
    paragraphs: [
      "L'app è fornita gratuitamente e «così com'è». L'autore non garantisce che sia priva di errori, che funzioni senza interruzioni o che sia adatta alla tua attività.",
    ],
  },
  {
    number: 3,
    title: 'Importi e contenuto delle fatture',
    paragraphs: [
      "L'app calcola importi, contributo integrativo ENPAP, marca da bollo, ritenuta d'acconto e diciture con le regole note all'autore al momento del rilascio. Le regole fiscali cambiano e dipendono dalla tua situazione.",
      "La responsabilità del contenuto delle fatture, verso i pazienti e verso l'Amministrazione finanziaria, resta tua. Verifica le prime fatture con il tuo commercialista e verificale di nuovo quando cambiano le norme o il tuo regime.",
    ],
  },
  {
    number: 4,
    title: 'Sistema TS ed email',
    paragraphs: [
      "Le trasmissioni al Sistema Tessera Sanitaria e gli invii email partono con le tue credenziali e sotto la tua responsabilità. Controlla l'esito di ogni trasmissione nell'app o sul portale del Sistema TS.",
      "L'autore non risponde di trasmissioni scartate, ritardate o non eseguite, né del mancato rispetto delle relative scadenze.",
    ],
  },
  {
    number: 5,
    title: 'Dati e backup',
    paragraphs: [
      "I dati restano sul tuo computer. L'autore non riceve e non conserva i tuoi dati né quelli dei tuoi pazienti, dei quali sei titolare del trattamento.",
      'Il backup del file database.db è a tuo carico.',
    ],
  },
  {
    number: 6,
    title: 'Limitazione di responsabilità',
    paragraphs: [
      "Nei limiti consentiti dalla legge, l'autore non risponde di danni diretti o indiretti derivanti dall'uso o dal mancato uso dell'app. Rientrano tra questi gli errori negli importi, le sanzioni, gli interessi, la perdita di dati e i mancati guadagni.",
      "La limitazione non si applica ai casi di dolo o colpa grave, come prevede l'art. 1229 del Codice civile.",
    ],
  },
  {
    number: 7,
    title: 'Modifiche',
    paragraphs: [
      "Le condizioni possono cambiare con una nuova versione dell'app. In quel caso l'app ti chiede di accettarle di nuovo prima di continuare.",
    ],
  },
  {
    number: 8,
    title: 'Legge applicabile',
    paragraphs: ['Le condizioni sono regolate dalla legge italiana.'],
  },
]

/** The clauses that limit liability, approved with their own checkbox as artt. 1341-1342 c.c. require. */
export const SPECIFICALLY_APPROVED: readonly number[] = [2, 3, 4, 5, 6]

/** "a", "a e b", "a, b e c". */
function joinItalian(items: readonly string[]): string {
  if (items.length <= 1) return items.join('')
  return `${items.slice(0, -1).join(', ')} e ${items[items.length - 1]}`
}

export function specificApprovalText(): string {
  const listed = SPECIFICALLY_APPROVED.map((number) => {
    const clause = TERMS_CLAUSES.find((candidate) => candidate.number === number)
    return clause ? `${number} (${clause.title})` : String(number)
  })
  return `Ai sensi degli artt. 1341 e 1342 del Codice civile approvo specificamente i punti ${joinItalian(listed)}.`
}
