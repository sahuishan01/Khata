# DESIGN.md — Khata Design Contract

**Status:** Authoritative. **Applies to:** Web (`khata.algosculptor.com`) and Android.
**Rule of precedence:** When the two apps disagree, this file wins. When this file is silent, **match the web app** — it is the reference implementation.

Keywords **MUST / MUST NOT / SHOULD / SHOULD NOT / MAY** are used in the RFC 2119 sense.

---

## 0. Why this file exists

The two apps have forked into different products: different currency formatting, different transaction rows, different navigation, different input controls. This document defines one set of tokens and one set of component contracts so they converge and stay converged. Every PR that touches UI **MUST** be checkable against this file.

---

## 1. The cardinal color rule

The look is **"Bahi by lamplight"**: a personal bahi-khata (the cloth-bound Indian
ledger) read at night. Warm lamp-black paper, paper-tinted text, fountain-pen ink
blue for the pen (actions), and the ledger's red and green ink for money. Color is
a rule, not decoration:

- **Brand (ink blue) is for _actions only_**: buttons, links, active nav tab, focus rings, selected toggles and filter pills.
- **Green and red are reserved _exclusively for money and state_**: green = money in / positive, red = money out / negative / destructive.
- **Amber is "needs attention"** only (low savings rate, budget ≥ 80%).
- Decorative icons, category chips and data values use the neutral text tokens, never brand.

Consequences:
- A selected "Expense" toggle is brand, not red; expense/income meaning lives in the amount color and the +/− sign.
- Bottom-nav active state is brand on both platforms.
- Unsigned totals (Total spent, liabilities, budget limits) are **neutral text**, not green. Only signed amounts are colored by sign.

---

## 2. Design tokens

These are the single source of truth. Port the **same hex values and scale** to both platforms (`frontend/src/index.css` ↔ `android/.../ui/theme/Theme.kt`). Do not hand-pick colors in components.

### 2.1 Color

| Token | Hex | Role |
|---|---|---|
| `bg` | `#12110F` | App background (lamp-black, warm) |
| `surface` | `#1B1916` | Cards, sheets |
| `surface-2` | `#24211D` | Inputs, chips, nested surfaces, expanded rows |
| `surface-3` | `#2E2A25` | Tracks, skeletons |
| `hairline` | `#34302A` | Borders, row dividers |
| `rule` | `#4D463D` | Ledger rules (subtotal/total lines), secondary-button outline |
| `text` | `#F1ECE3` | Primary text (paper) |
| `text-2` | `#B7AE9F` | Secondary text (≥ 7:1 on surface) |
| `text-muted` | `#968C7D` | Placeholders, disabled (≥ 4.5:1 on surface) |
| `brand` | `#6F8CFF` | **Actions only** (ink blue) |
| `brand-press` | `#5A78F0` | Pressed/hover brand |
| `brand-soft` | `rgba(111,140,255,.16)` | Selected pill / nav indicator background |
| `on-brand` | `#0E1226` | Text on a brand fill (white fails contrast) |
| `income` | `#52C78D` | **Money in only** |
| `income-soft` | `rgba(82,199,141,.14)` | Income icon bg |
| `expense` | `#F26B4F` | **Money out + destructive only** |
| `expense-soft` | `rgba(242,107,79,.14)` | Expense icon bg |
| `warn` | `#E8A33D` | Needs attention |

**Chart categorical palette** (validated with the dataviz palette checker on `surface`: adjacent CVD ΔE ≥ 8.4, normal-vision ΔE ≥ 19, ≥ 3:1 contrast). Fixed order, never cycled; the 7th+ category folds into **Other** `#6E675E`. No green/red, which mean money:
`#3987E5, #D95926, #199E70, #C98500, #D55181, #9085E9`.
Spend-vs-earn charts use `income`/`expense` because they *are* money in/out.

> Light mode is out of scope until both platforms reach parity in dark mode.

### 2.2 Typography

Two families, both bundled (web via `@fontsource`, Android in `res/font`). There are no
CDN requests, which the CSP's `font-src 'self'` requires.

| Family | Use |
|---|---|
| **Schibsted Grotesk** (variable) | All UI text, headings, payees |
| **Space Mono** 400/700 | Every money figure and date, plus short data labels. It is monospaced, so amounts line up in columns |

Both include the ₹ glyph. Labels are **sentence case**. No tracked-out ALL-CAPS eyebrows.

| Role | Size / Weight | Use |
|---|---|---|
| Hero figure | 40–44 / Mono 700 / −0.02em | The dashboard net figure |
| Display | 28 / 700 / −0.02em | Screen titles |
| Title | 18 / 650 | Card titles |
| Body | 15 / 400–500 | Default text, payees |
| Small | 13.5–14 / 400–500 | Secondary text, buttons |
| Caption | 12 / 500 | Dates, helper text. **Floor: nothing smaller than 12** |

Tabular figures apply to figures and mono text only, not running prose (tabular periods read as gaps: "63 .3%").

### 2.3 Spacing, radius, motion

- Spacing scale (4-pt): `4, 8, 12, 16, 20, 24, 32`.
- Radius follows hierarchy: `sm 6` (small buttons, swatches), `md 10` (inputs, buttons, icon boxes), `lg 14` (panels, cards), `pill 999` (chips, filter pills). Rows inside a panel are flush (radius 0) and separated by hairlines.
- No shadows on resting surfaces, no gradients, no glass, no hover-lift. Hierarchy comes from type and rules.
- Touch target: **min 44×44 px / 48×48 dp.**
- Motion: 150–200 ms ease-out for state changes only. One page-level fade on navigation. Charts do not animate on mount. Respect `prefers-reduced-motion` / system "remove animations".

### 2.4 Ledger devices (the signature)

Structure carries meaning, as in an accountant's book:
- A **single rule** (`rule` color) closes a set of lines into a subtotal.
- A **double rule** marks a final total (the dashboard "Saved" line).
- A faint red **margin rule** at the left of the statement panel nods to the bahi page, and appears in the app mark.
- Amount columns are right-aligned in Space Mono.

The dashboard opens with this **statement** (Earned, Spent, a rule, Saved/Overspent, a double rule, then a one-sentence note) instead of a row of look-alike stat cards. It is the one bold element. Everything around it stays quiet.

### 2.5 Platform mapping

```css
/* Web: frontend/src/index.css */
:root { --bg:#12110F; --surface:#1B1916; --brand:#6F8CFF; --on-brand:#0E1226;
        --income:#52C78D; --expense:#F26B4F; /* …full set… */ }
```
```kotlin
// Android: ui/theme/Theme.kt (mirror the SAME values; do not improvise)
object KhataColors {
  val bg = Color(0xFF12110F); val surface = Color(0xFF1B1916)
  val brand = Color(0xFF6F8CFF); val onBrand = Color(0xFF0E1226)
  val income = Color(0xFF52C78D); val expense = Color(0xFFF26B4F)
  // …full set…
}
```

---

## 3. Money & numbers (single shared helper)

The web already formats correctly (`₹14,58,879`). Android uses international grouping (`₹1,458,879`) and stray `.00`. The donut center even printed an unformatted, off-by-one `1458878`. All of this is one root cause: formatting is duplicated instead of shared.

Rules:
1. **Store money as integer paise (`Long`), never floating point.** Float is the source of the off-by-one and rounding drift.
2. **One formatter, called from every screen** including chart centers, tooltips, and the LLM answer text.
3. Grouping **MUST** be Indian (lakh/crore) via `en-IN`.
4. Decimals: show 2 only when non-zero fractional paise exist; otherwise show none. Be consistent within a list.
5. Signed contexts (transaction amounts, deltas): prefix `+` / `−` and color by sign. Unsigned contexts (totals, balances): no prefix; negative uses `−`.
6. **Clamp `−0` to `₹0`** (Portfolio currently shows `₹-0`).

```ts
// shared/format.ts
export function formatINR(paise: number, opts: { sign?: boolean } = {}) {
  const rupees = Math.round(paise) / 100, abs = Math.abs(rupees);
  const body = abs.toLocaleString('en-IN', {
    minimumFractionDigits: Number.isInteger(abs) ? 0 : 2, maximumFractionDigits: 2 });
  const s = opts.sign ? (rupees < 0 ? '−' : '+') : (rupees < 0 ? '−' : '');
  return `${s}₹${body}`;
}
```
```kotlin
// Android equivalent — SAME rules
fun formatINR(paise: Long, sign: Boolean = false): String {
  val rupees = paise / 100.0; val abs = kotlin.math.abs(rupees)
  val nf = NumberFormat.getInstance(Locale.forLanguageTag("en-IN")).apply {
    maximumFractionDigits = 2
    minimumFractionDigits = if (abs % 1.0 == 0.0) 0 else 2
  }
  val pre = if (sign) (if (rupees < 0) "−" else "+") else (if (rupees < 0) "−" else "")
  return "$pre₹${nf.format(abs)}"
}
```

**Dates:** display format **MUST** be `dd MMM yyyy` (`31 Mar 2026`) on both platforms; never raw ISO in the UI. Transactions carry both `txn_date` and `value_date` — label them explicitly.

---

## 4. Component contracts

### 4.1 Transaction row (canonical = web)
- Layout: `[direction icon 36px] [payee + meta, flex] [amount, right-aligned, tabular]`.
- **Payee MUST show in full**, wrapping to max 2 lines then ellipsis. No mid-string truncation (Android's `…@AXL-BA…` hides the meaningful part).
- Category is an **inline dropdown chip** (edit in place), not a separate screen.
- Amount: signed, colored (`income`/`expense`), Space Mono. **Transfers** (`is_transfer`) show a neutral "Transfer" tag and a muted amount, because they are excluded from totals.
- Edit/swap actions **MUST NOT** sit in the resting layout. Use swipe (mobile) or row-hover (web). This removes Android's extra button row and roughly doubles rows-per-screen across 517 items.

### 4.2 Category-rule row
- Keyword chip **MUST** be width-capped with ellipsis (`max-width:100%`), `title`/tooltip carries the full string. A long UPI keyword **MUST NOT** be able to expand the card (current "UPI-CRED CLUB…" bug, present on both platforms).
- The `→ Category` mapping **MUST** remain visible on every row.
- Adding a rule **MUST** reject a duplicate keyword (you currently have two `INTEREST PAID` rules with different targets — undefined behavior). Define and document match precedence (recommend: most-specific/longest keyword wins; ties → newest).

### 4.3 Empty states
Every empty list **MUST** render: an icon, one sentence of direction, and the primary action — never bare "No X" text floating in a void. Example: `🎯 · No budgets yet · Set a monthly limit per category to get alerts before you overspend. · [Set your first budget]`.

### 4.4 Buttons
- Primary = filled `brand` with `on-brand` text. Secondary = `rule` outline. Destructive = `expense` outline + trailing `…` + confirmation dialog (see §4.6).
- One primary action per screen.

### 4.5 Navigation / IA (convergence decision)
The web's 7 flat tabs overflow and clip "Budgets"; Android's 5-tab + "More" hub scales. **Decision: adopt Android's structure, web's visuals.**
- Primary tabs (both platforms): **Dashboard · Transactions · Add · More · Settings**.
- Everything else (Analytics, Ask Claude, Accounts, Rules, Budgets, Portfolio, Categories, Users) lives under **More**.
- Labels **MUST** match across platforms: use `Transactions` (not "Txns"), `Add` , `Ask Claude` (not "Chat"). No per-platform renaming.
- Active tab uses `brand`.

### 4.6 Destructive actions
"Clear all data," "Delete user," "Delete account" **MUST**: be visually separated from benign actions (today "Clear All Data" sits flush against "Logout"), use `expense` color, end with `…`, and require a **typed confirmation** ("type DELETE") — not a single tap. See SECURITY.md §2.4.

### 4.7 PII on screen (links to SECURITY.md §2.3)
Account numbers, UPI IDs, full names, and IMPS reference strings are PII. The UI **MUST** offer a privacy/blur mode and **MUST** mask account numbers by default (`••••5914`). Do not render full IMPS reference strings in headline cards (Dashboard "Largest Expense" currently does).

---

## 5. Accessibility floor (non-negotiable)
- Color contrast ≥ 4.5:1 for text. Verify `text-2`/`text-muted` on `surface`.
- Never encode meaning by color alone — the +/− sign and icon carry income/expense too (colorblind users).
- Visible keyboard/focus state (web) and TalkBack labels (Android) on every interactive element.
- Respect reduced-motion and system font scaling.

---

## 6. Parity checklist (paste into every UI PR)
- [ ] Uses tokens from §2 only (no ad-hoc hex / spacing).
- [ ] Brand used for actions only; green/red for money/state only; unsigned totals neutral.
- [ ] All money via shared `formatINR`; dates via `dd MMM yyyy`.
- [ ] Amounts and dates use Space Mono; nothing below 12px/sp; sentence-case labels.
- [ ] Touch targets ≥ 44/48.
- [ ] Component matches its §4 contract on **both** platforms.
- [ ] Empty/error/loading states defined.
- [ ] Screenshot of web **and** Android attached, side by side.

