-- Keep manual transfer choices authoritative when rescans add new evidence.
ALTER TABLE transactions ADD COLUMN transfer_override boolean NOT NULL DEFAULT false;
ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
UPDATE transactions SET transfer_override = true WHERE rev > 1;
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
CREATE INDEX idx_transactions_card_evidence ON transactions (user_id,amount,value_date,direction) WHERE NOT deleted;

CREATE FUNCTION is_cred_funding(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND t.direction = 'debit' AND is_named_bank(t.bank)
      AND trim(regexp_replace(upper(t.description), '[^A-Z0-9]+', ' ', 'g')) IN ('CRED','CRED CLUB');
$$;
CREATE FUNCTION is_card_receipt(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND t.direction = 'credit' AND t.is_transfer
      AND (is_card_repayment(t.description, t.direction, t.bank = 'GENERIC_CC')
           OR t.sms_source->>'payment_role' IN ('card_receipt','card_confirmation'));
$$;
CREATE FUNCTION is_cred_card_pair(f transactions, r transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT f.user_id = r.user_id AND f.id <> r.id AND f.amount = r.amount AND f.amount > 0
      AND is_cred_funding(f) AND is_card_receipt(r) AND abs(f.value_date-r.value_date) <= 2
      AND f.sms_source->>'received_at' IS NOT NULL AND r.sms_source->>'received_at' IS NOT NULL
      AND abs((f.sms_source->>'received_at')::bigint - (r.sms_source->>'received_at')::bigint) <= 600000;
$$;

-- A review suggestion, never proof based only on equal amounts. It needs a
-- known card receipt linked in time to a CRED debit and a later same-bank credit.
CREATE FUNCTION is_card_confirmation_candidate(a transactions, b transactions) RETURNS boolean LANGUAGE plpgsql STABLE AS $$
DECLARE receipt transactions; confirmation transactions;
BEGIN
    IF a.user_id <> b.user_id OR a.id = b.id OR a.deleted OR b.deleted
       OR a.direction <> 'credit' OR b.direction <> 'credit' OR a.amount <> b.amount
       OR upper(a.bank) <> upper(b.bank) OR abs(a.value_date-b.value_date) > 2 THEN RETURN false; END IF;
    IF is_card_receipt(b) AND (lower(trim(a.description)) = 'transaction'
        OR a.sms_source->>'payment_role'='card_confirmation') THEN receipt := b; confirmation := a;
    ELSE RETURN false; END IF;
    IF receipt.sms_source->>'received_at' IS NULL OR confirmation.sms_source->>'received_at' IS NULL THEN RETURN false; END IF;
    RETURN (confirmation.sms_source->>'received_at')::bigint BETWEEN
        (receipt.sms_source->>'received_at')::bigint - 120000 AND (receipt.sms_source->>'received_at')::bigint + 172800000
        AND EXISTS (SELECT 1 FROM transactions f WHERE f.user_id = receipt.user_id
            AND f.amount = receipt.amount AND is_cred_card_pair(f,receipt));
END;
$$;

CREATE FUNCTION reconcile_card_repayments(owner_id uuid, paid numeric, dated date) RETURNS void LANGUAGE plpgsql AS $$
DECLARE pair record; confirmation record; root transactions; funding_count integer; debit_count integer;
BEGIN
    PERFORM id FROM users WHERE id = owner_id FOR UPDATE;
    -- Exactly one immediate receipt and one CRED debit establish the two transfer legs.
    FOR pair IN
        SELECT f.id FROM transactions f JOIN transactions r ON is_cred_card_pair(f,r)
        WHERE f.user_id=owner_id AND f.amount=paid AND abs(f.value_date-dated)<=2
          AND NOT f.is_transfer AND NOT f.transfer_override
          AND NOT EXISTS (SELECT 1 FROM transactions other WHERE other.user_id=owner_id
              AND other.id<>f.id AND other.amount=paid AND is_cred_card_pair(other,r))
        GROUP BY f.id HAVING count(*)=1
    LOOP
        UPDATE transactions SET is_transfer=true,rev=rev+1,version=version+1,updated_at=now() WHERE id=pair.id;
    END LOOP;

    -- Automatic exclusion needs distinct receipt/confirmation roles, the same
    -- card fingerprint and exactly one recorded funding debit. Two receipts,
    -- different cards, missing identifiers and ambiguous funding stay separate.
    FOR root IN SELECT t.* FROM transactions t WHERE t.user_id=owner_id AND t.amount=paid
        AND abs(t.value_date-dated)<=2 AND is_card_receipt(t)
        AND t.sms_source->>'payment_role'='card_receipt'
        AND length(t.sms_source->>'card_account_key')=64
    LOOP
        SELECT count(*) INTO funding_count FROM transactions f WHERE f.user_id=owner_id AND f.amount=paid AND is_cred_card_pair(f,root);
        SELECT count(*) INTO debit_count FROM transactions f WHERE f.user_id=owner_id AND NOT f.deleted
            AND f.direction='debit' AND f.amount=paid AND abs(f.value_date-root.value_date)<=2;
        IF funding_count<>1 OR debit_count<>1 THEN CONTINUE; END IF;
        IF EXISTS (SELECT 1 FROM transactions other WHERE other.user_id=owner_id AND other.id<>root.id
            AND is_card_receipt(other) AND other.amount=paid AND abs(other.value_date-root.value_date)<=2
            AND other.sms_source->>'payment_role'='card_receipt'
            AND other.sms_source->>'card_account_key'=root.sms_source->>'card_account_key') THEN CONTINUE; END IF;
        FOR confirmation IN SELECT c.id FROM transactions c WHERE c.user_id=owner_id AND c.id<>root.id
            AND is_card_receipt(c) AND c.amount=paid AND upper(c.bank)=upper(root.bank)
            AND abs(c.value_date-root.value_date)<=2
            AND c.sms_source->>'payment_role'='card_confirmation'
            AND c.sms_source->>'card_account_key'=root.sms_source->>'card_account_key'
            AND (c.sms_source->>'received_at')::bigint BETWEEN (root.sms_source->>'received_at')::bigint
                AND (root.sms_source->>'received_at')::bigint + 172800000
        LOOP
            UPDATE transactions SET duplicate_of=root.id,rev=rev+1,version=version+1,updated_at=now() WHERE duplicate_of=confirmation.id;
            UPDATE transactions SET duplicate_of=root.id,deleted=true,rev=rev+1,version=version+1,updated_at=now() WHERE id=confirmation.id;
        END LOOP;
    END LOOP;
END;
$$;

CREATE FUNCTION preserve_manual_transfer() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.transfer_override THEN NEW.is_transfer := OLD.is_transfer; END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER transaction_00_manual_transfer BEFORE UPDATE OF sms_source ON transactions
FOR EACH ROW EXECUTE FUNCTION preserve_manual_transfer();
CREATE FUNCTION reconcile_new_card_evidence() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP='UPDATE' AND NEW.sms_source IS NOT DISTINCT FROM OLD.sms_source
       AND NEW.description=OLD.description AND NEW.bank_ref IS NOT DISTINCT FROM OLD.bank_ref THEN RETURN NEW; END IF;
    IF NOT NEW.deleted AND (is_cred_funding(NEW) OR is_card_receipt(NEW)) THEN
        PERFORM reconcile_card_repayments(NEW.user_id,NEW.amount,NEW.value_date);
    END IF;
    RETURN NEW;
END;
$$;
-- Evaluate the whole sync batch before inferring uniqueness, irrespective of alert order.
CREATE CONSTRAINT TRIGGER transaction_card_evidence AFTER INSERT OR UPDATE ON transactions
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION reconcile_new_card_evidence();

ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
WITH candidates AS MATERIALIZED (
    SELECT DISTINCT user_id,amount,value_date FROM transactions t WHERE is_cred_funding(t)
)
SELECT reconcile_card_repayments(user_id,amount,value_date) FROM candidates;
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
