/**
 * Categorical palette for the dark chart surface (#1B1916). Validated with the
 * dataviz palette checker (lightness band, chroma floor, adjacent CVD ΔE ≥ 8.4,
 * normal-vision ΔE ≥ 19, ≥ 3:1 contrast). Green and red are deliberately absent:
 * they mean money in / money out everywhere else (DESIGN.md §1).
 *
 * Fixed order, never cycled — a 7th+ category folds into "Other".
 */
export const CATEGORICAL = ['#3987E5', '#D95926', '#199E70', '#C98500', '#D55181', '#9085E9'] as const
export const OTHER = '#6E675E'

export function foldToOther<T extends { category: string; amount: number; pct: number }>(data: T[]): T[] {
  if (data.length <= CATEGORICAL.length) return data
  const head = data.slice(0, CATEGORICAL.length - 1)
  const tail = data.slice(CATEGORICAL.length - 1)
  const other = {
    ...tail[0],
    category: 'Other',
    amount: tail.reduce((s, d) => s + d.amount, 0),
    pct: tail.reduce((s, d) => s + d.pct, 0),
  }
  return [...head, other]
}

export function colorFor(category: string, index: number): string {
  return category === 'Other' ? OTHER : CATEGORICAL[index % CATEGORICAL.length]
}
