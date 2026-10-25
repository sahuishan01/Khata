# Generic SMS posting grammar

SMS transaction recognition does not select templates by bank or match an amount,
account, card or reference from a particular report. The parser uses shared
posting grammar across issuers:

- Completed debit/credit wording before or after a rupee amount.
- Account-qualified `Dr`/`Cr` markers, with optional periods and posted-state words.
  The amount can precede or follow the marker. Account labels include `A/c`, `AC`,
  `Acct`, `Acc` and `Account`, with masked or numeric account identifiers.
- Currency tokens `INR`, `Rs`, `Rs.` and `₹`, with grouped or ungrouped amounts.
- Payee field boundaries such as transaction dates, references, balances and
  punctuation. A semicolon ends the payee before the next ledger field.
- UPI references with colon, slash or hyphen separators and optional whitespace.
  A UPI reference must have exactly 12 digits; existing reference identities remain
  discoverable through the importer's legacy lookup.

An issuer catalog supplies canonical bank names and sender aliases. An optional
`Bank` suffix can be attached or separated by whitespace. An unknown issuer still
produces a transaction when the posting grammar matches. Learning an issuer alias
must not duplicate a verified old record that used the unknown-issuer identity.

Abbreviations alone are insufficient: doctor names, available balances and credit
limits are not postings. Existing clause guards continue to reject requests,
pending/future debits, application acknowledgements, fund blocks and offers.
Neither a larger account balance nor a matching amount makes a card repayment.

A card-payment receipt keeps its `credit` direction and transfer flag. A CRED
funding debit keeps its `debit` direction. Existing server reconciliation may mark
both as transfers using a unique matching repayment, compatible dates and receipt
times; amount alone is not sufficient. Transfer entries remain visible but leave
income/spending totals. Explicit user decisions remain authoritative.

Install the signed update, then use **Add Data → Scan SMS inbox** while the
original messages remain present. Missed postings are added once with stable
identities. Exact SMS text stays in Notes, and existing custom titles/notes and
full edits remain protected. A web refresh shows the synced results.

Public regressions use neutralized examples across known and unknown issuers,
both directions, account labels, currencies, separators, exclusions, import
idempotence and prior identities. Exact reported messages and their Android-to-
server replay checks are kept in the protected local evidence directory rather
than published with account/card details. No production transactions are inserted
or changed by development tests.


Verified signed release: [v0.45.26 APK](https://github.com/sahuishan01/Khata/releases/download/v0.45.26/khata_0.45.26-release.apk) · [Release page](https://github.com/sahuishan01/Khata/releases/tag/v0.45.26).
CI `40194289828` passed 88 release tests; release lint, exact-message regressions,
isolated server repayment/retry replay, Security Scan `40194289598`, browser
checks and API 35 SMS receive/upload/display checks passed. Production API/web
source remains unchanged from verified run `40176342020`.
