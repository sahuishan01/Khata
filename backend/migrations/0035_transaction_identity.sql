-- Retain excluded SMS identities as sync tombstones so rescans cannot recreate them.
ALTER TABLE transactions ADD COLUMN duplicate_of uuid REFERENCES transactions(id);
ALTER TABLE transactions ADD CONSTRAINT transaction_duplicate_not_self
    CHECK (duplicate_of IS NULL OR duplicate_of <> id);
CREATE INDEX idx_transactions_reference_identity
    ON transactions (user_id, (btrim(bank_ref)), amount, value_date)
    WHERE NOT deleted AND btrim(bank_ref) ~ '^[0-9]{10,20}$';
CREATE INDEX idx_transactions_duplicate_review
    ON transactions (user_id, value_date, amount, direction)
    WHERE NOT deleted;

CREATE FUNCTION is_named_bank(bank text) RETURNS boolean
LANGUAGE sql IMMUTABLE AS $$
    SELECT upper(bank) IN ('HDFC','KOTAK','AXIS','CANARA','ICICI','SBI','PNB',
        'BOB','PAYTM','IDFC','YES','INDUSIND');
$$;

CREATE FUNCTION protect_transaction_identity() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    matches uuid[];
    counterpart uuid;
BEGIN
    -- Serialize reference checks with other device/statement imports for this tenant.
    PERFORM id FROM users WHERE id = NEW.user_id FOR UPDATE;
    IF NEW.duplicate_of IS NOT NULL THEN
        IF NOT EXISTS (SELECT 1 FROM transactions t
                       WHERE t.id = NEW.duplicate_of AND t.user_id = NEW.user_id
                         AND t.duplicate_of IS NULL AND t.id <> NEW.id) THEN
            RAISE EXCEPTION 'Invalid duplicate target';
        END IF;
        NEW.deleted := true;
        RETURN NEW;
    END IF;

    -- Later manual edits win. Automatic classification is insertion-only.
    IF TG_OP <> 'INSERT' OR NEW.deleted OR NOT is_named_bank(NEW.bank)
       OR NEW.bank_ref IS NULL OR btrim(NEW.bank_ref) !~ '^[0-9]{10,20}$'
       OR NEW.amount <= 0 THEN
        RETURN NEW;
    END IF;
    -- An ON CONFLICT retry must not reclassify an existing client's manual override.
    IF NEW.client_id IS NOT NULL AND EXISTS (SELECT 1 FROM transactions t
        WHERE t.user_id = NEW.user_id AND t.client_id = NEW.client_id) THEN
        RETURN NEW;
    END IF;

    SELECT array_agg(t.id) INTO matches FROM transactions t
    WHERE t.user_id = NEW.user_id AND NOT t.deleted
      AND btrim(t.bank_ref) ~ '^[0-9]{10,20}$'
      AND upper(t.bank) = upper(NEW.bank) AND btrim(t.bank_ref) = btrim(NEW.bank_ref)
      AND t.direction = NEW.direction AND t.amount = NEW.amount
      AND abs(t.value_date - NEW.value_date) <= 2;
    IF cardinality(matches) = 1 THEN
        NEW.duplicate_of := matches[1];
        NEW.deleted := true;
        RETURN NEW;
    ELSIF cardinality(matches) > 1 THEN
        RETURN NEW;
    END IF;

    -- Only a unique debit/credit pair in different banks proves an own-account move.
    SELECT array_agg(t.id) INTO matches FROM transactions t
    WHERE t.user_id = NEW.user_id AND NOT t.deleted AND is_named_bank(t.bank)
      AND btrim(t.bank_ref) = btrim(NEW.bank_ref) AND t.amount = NEW.amount
      AND abs(t.value_date - NEW.value_date) <= 2;
    IF cardinality(matches) = 1 THEN
        SELECT t.id INTO counterpart FROM transactions t
        WHERE t.id = matches[1] AND upper(t.bank) <> upper(NEW.bank)
          AND t.direction <> NEW.direction;
        IF counterpart IS NOT NULL THEN
            NEW.is_transfer := true;
            UPDATE transactions SET is_transfer = true, version = version + 1,
                rev = rev + 1, updated_at = now()
            WHERE id = counterpart AND NOT is_transfer;
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER transaction_identity
BEFORE INSERT OR UPDATE ON transactions
FOR EACH ROW EXECUTE FUNCTION protect_transaction_identity();

-- One-time repair for existing unique reference-backed own-account pairs.
-- Never infer a duplicate or transfer from amount, merchant and date alone.
ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
WITH pairs AS (
    SELECT d.id AS debit_id, c.id AS credit_id
    FROM transactions d JOIN transactions c ON c.user_id = d.user_id
      AND btrim(c.bank_ref) = btrim(d.bank_ref) AND c.amount = d.amount
      AND c.direction = 'credit' AND abs(c.value_date - d.value_date) <= 2
    WHERE d.direction = 'debit' AND NOT d.deleted AND NOT c.deleted
      AND d.amount > 0 AND btrim(d.bank_ref) ~ '^[0-9]{10,20}$'
      AND is_named_bank(d.bank) AND is_named_bank(c.bank)
      AND upper(d.bank) <> upper(c.bank)
      AND NOT EXISTS (
          SELECT 1 FROM transactions t WHERE t.user_id = d.user_id AND NOT t.deleted
            AND btrim(t.bank_ref) = btrim(d.bank_ref) AND t.amount = d.amount
            AND (abs(t.value_date - d.value_date) <= 2 OR abs(t.value_date - c.value_date) <= 2)
            AND t.id NOT IN (d.id, c.id)
      )
)
UPDATE transactions SET is_transfer = true, version = version + 1,
    rev = rev + 1, updated_at = now()
WHERE NOT is_transfer AND id IN (SELECT debit_id FROM pairs UNION SELECT credit_id FROM pairs);
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
