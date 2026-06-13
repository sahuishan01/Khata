# DESIGN.md — Khata Design Contract

**Status:** Authoritative. **Applies to:** Web (`khata.algosculptor.com`) and Android.
**Rule of precedence:** When the two apps disagree, this file wins. When this file is silent, **match the web app** — it is the reference implementation.

Keywords **MUST / MUST NOT / SHOULD / SHOULD NOT / MAY** are used in the RFC 2119 sense.

---

## 0. Why this file exists

The two apps have forked into different products: different currency formatting, different transaction rows, different navigation, different input controls. This document defines one set of tokens and one set of component contracts so they converge and stay converged. Every PR that touches UI **MUST** be checkable against this file.

---

## 1. The cardinal color rule

Most of the "looks unpolished / inconsistent" feedback traces to two accent colors (purple **and** green) competing for "primary." Fix it with a rule, not a repaint:

- **Purple is for _actions only_** — buttons, links, active nav tab, focus rings, selected toggles.
- **Green and red are reserved _exclusively for money and state_** — green = money in / positive, red = money out / negative / destructive.

Consequences (these are current bugs that this rule resolves):
- The Android "Expense" type toggle **MUST NOT** highlight green (it currently does, while expense rows are red — a contradiction). Selected toggles use purple; the expense/income meaning is carried by the amount color and the +/− sign.
- Bottom-nav active state **MUST** be purple on both platforms (Android currently uses green).
- Filter pills, "Apply", and primary CTAs **MUST** all use the same purple.

---

## 2. Design tokens

These are the single source of truth. Port the **same hex values and scale** to both platforms. Do not hand-pick colors in components.

### 2.1 Color

| Token | Hex | Role |
|---|---|---|
| `bg` | `#0E0E13` | App background |
| `surface` | `#17171F` | Cards, sheets |
| `surface-2` | `#1F1F2A` | Inputs, chips, nested surfaces |
| `hairline` | `#2A2A38` | Borders, dividers |
| `text` | `#F2F2F5` | Primary text |
| `text-2` | `#9A9AA8` | Secondary text |
| `text-muted` | `#6B6B78` | Captions, placeholders, disabled |
| `brand` | `#8479F2` | **Actions only** |
| `brand-press` | `#6F62E6` | Pressed/active brand |
| `brand-soft` | `rgba(132,121,242,.16)` | Brand chip/badge backgrounds |
| `income` | `#2EC27E` | **Money in only** |
| `income-soft` | `rgba(46,194,126,.14)` | Income badge bg |
| `expense` | `#EE6B4D` | **Money out + destructive only** |
| `expense-soft` | `rgba(238,107,77,.14)` | Expense badge bg |
| `warn` | `#E0A33A` | "Needs attention" (e.g. low savings rate) |

> Light mode is out of scope until both platforms reach parity in dark mode. The Settings "Dark Mode" toggle **MUST** persist server-side per user, not only in local storage.

### 2.2 Typography

System stack (no web-font dependency, identical metrics both platforms):
`-apple-system, "Segoe UI", Roboto, Inter, system-ui, sans-serif`.

| Role | Size / Weight / Tracking | Use |
|---|---|---|
| Display | 26 / 700 / −0.02em | Screen titles ("Dashboard") |
| Title | 20 / 700 | Card titles, section heads |
| Body | 15 / 500 | Default text |
| Body-strong | 15 / 600 | Payee names, list item titles |
| Label | 13 / 600 / 0.08em UPPER | Eyebrows, field labels |
| Caption | 12.5 / 500 | Dates, helper text |

**All numeric/monetary text MUST use tabular figures** (`font-variant-numeric: tabular-nums` / `font-feature-settings:"tnum"`; on Android, a font with tabular numerals or `TextView` with monospaced digits). This is what makes amount columns line up.

### 2.3 Spacing, radius, motion

- Spacing scale (4-pt): `4, 8, 12, 16, 20, 24, 32`. No off-scale values.
- Radius: `sm 8`, `md 12`, `lg 16`, `pill 999`.
- Touch target: **min 44×44 px / 48×48 dp.** Trash/delete/edit icons currently fail this.
- Motion: 150–200 ms ease-out for state changes; respect `prefers-reduced-motion` / system "remove animations."

### 2.4 Platform mapping

```css
/* Web: tokens.css */
:root { --bg:#0E0E13; --surface:#17171F; --brand:#8479F2;
        --income:#2EC27E; --expense:#EE6B4D; /* …full set… */ }
```
```kotlin
// Android: Theme.kt  (mirror the SAME values — do not improvise)
object KhataColors {
  val bg = Color(0xFF0E0E13); val surface = Color(0xFF17171F)
  val brand = Color(0xFF8479F2)
  val income = Color(0xFF2EC27E); val expense = Color(0xFFEE6B4D)
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
- Amount: signed, colored (`income`/`expense`), tabular.
- Edit/swap actions **MUST NOT** sit in the resting layout. Use swipe (mobile) or row-hover (web). This removes Android's extra button row and roughly doubles rows-per-screen across 517 items.

### 4.2 Category-rule row
- Keyword chip **MUST** be width-capped with ellipsis (`max-width:100%`), `title`/tooltip carries the full string. A long UPI keyword **MUST NOT** be able to expand the card (current "UPI-CRED CLUB…" bug, present on both platforms).
- The `→ Category` mapping **MUST** remain visible on every row.
- Adding a rule **MUST** reject a duplicate keyword (you currently have two `INTEREST PAID` rules with different targets — undefined behavior). Define and document match precedence (recommend: most-specific/longest keyword wins; ties → newest).

### 4.3 Empty states
Every empty list **MUST** render: an icon, one sentence of direction, and the primary action — never bare "No X" text floating in a void. Example: `🎯 · No budgets yet · Set a monthly limit per category to get alerts before you overspend. · [Set your first budget]`.

### 4.4 Buttons
- Primary = filled `brand`. Secondary = `hairline` outline. Destructive = `expense` outline + trailing `…` + confirmation dialog (see §4.6).
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
- [ ] Purple used for actions only; green/red for money/state only.
- [ ] All money via shared `formatINR`; dates via `dd MMM yyyy`.
- [ ] Amounts/totals use tabular figures.
- [ ] Touch targets ≥ 44/48.
- [ ] Component matches its §4 contract on **both** platforms.
- [ ] Empty/error/loading states defined.
- [ ] Screenshot of web **and** Android attached, side by side.

