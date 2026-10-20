-- Shared policy for statement imports, email alerts, device sync and backfill.
-- A card purchase/refund is economic activity; repayment of its bill is not.
CREATE FUNCTION is_card_repayment(description text, direction text, card_account boolean DEFAULT false)
RETURNS boolean LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $$
    WITH normalized AS (
        SELECT trim(regexp_replace(upper(coalesce(description, '')), '[^A-Z0-9]+', ' ', 'g')) AS d
    )
    SELECT d !~ '\m(REFUND|REVERSAL|REVERSED|CASHBACK|CHARGEBACK|FEE|FEES|CHARGE|CHARGES|INTEREST|INSURANCE|FAILED|DECLINED|UNSUCCESSFUL)\M|PAYMENT DUE|DUE DATE|WILL BE|SCHEDULED'
      AND d !~ '\m(CREDIT CARD|CREDITCARD|CC) PAYMENT (AT|TO)\M'
      AND (
        d ~ '\m(CREDIT CARD|CREDITCARD|CC) (BILL )?(PAYMENT|REPAYMENT|PMT)\M|\mCCPMT\M'
        OR ((card_account OR d ~ '\m(CREDIT CARD|CREDITCARD|CC)\M') AND (
            (direction = 'debit' AND d ~ '\m(PAYMENT|PAID|DEBITED|TOWARDS)\M.*\m(TOWARDS|FOR|TO)\M.*\m(CREDIT CARD|CREDITCARD|CC)\M|\mTOWARDS\M.*\m(CREDIT CARD|CREDITCARD|CC)\M')
            OR (direction = 'credit' AND d ~ '\mPAYMENT\M.*\m(RECEIVED|CREDITED|ACKNOWLEDGED|SETTLED|SETTLEMENT)\M|\m(RECEIVED|CREDITED)\M.*\mPAYMENT\M|\mTHANK YOU FOR (PAYING|YOUR PAYMENT)\M')
        ))
      )
    FROM normalized;
$$;

CREATE FUNCTION classify_new_card_repayment() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.is_transfer := NEW.is_transfer OR is_card_repayment(
        NEW.description || ' ' || NEW.raw_description, NEW.direction, NEW.bank = 'GENERIC_CC'
    );
    RETURN NEW;
END;
$$;

-- Only classify automatically on insertion. A user's later transfer override wins.
CREATE TRIGGER transaction_card_repayment
BEFORE INSERT ON transactions
FOR EACH ROW EXECUTE FUNCTION classify_new_card_repayment();

-- Correct existing identifiable repayments once, preserving the rows and amounts.
-- The migration owner must see every tenant; FORCE RLS is restored in this same transaction.
ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
UPDATE transactions SET is_transfer = true, version = version + 1, rev = rev + 1, updated_at = now()
WHERE NOT is_transfer AND NOT deleted
  AND is_card_repayment(description || ' ' || raw_description, direction, bank = 'GENERIC_CC');
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
