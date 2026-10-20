/** The form text is compared in when a search box matches it: no case, no accents. */
export function searchKey(text: string): string {
  return text.toLocaleLowerCase('it-IT').normalize('NFD').replace(/\p{Diacritic}/gu, '')
}
