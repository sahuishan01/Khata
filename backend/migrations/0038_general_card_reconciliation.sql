-- Reconcile complete repayment groups for every tenant, not particular SMS IDs.
ALTER TABLE transactions ADD COLUMN duplicate_override boolean NOT NULL DEFAULT false;
ALTER TABLE transactions ADD COLUMN duplicate_reason text;

CREATE OR REPLACE FUNCTION protect_sms_evidence() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    latest date := (now() AT TIME ZONE 'Asia/Kolkata')::date;
    received bigint;
    matches uuid[];
    source_changed boolean := TG_OP = 'INSERT';
    ref_changed boolean := TG_OP = 'INSERT';
BEGIN
    PERFORM id FROM users WHERE id = NEW.user_id FOR UPDATE;
    IF TG_OP = 'UPDATE' THEN
        source_changed := NEW.sms_source IS DISTINCT FROM OLD.sms_source;
        ref_changed := NEW.bank_ref IS DISTINCT FROM OLD.bank_ref;
    END IF;
    IF NEW.deleted OR NEW.duplicate_of IS NOT NULL THEN RETURN NEW; END IF;

    -- Old clients have no provenance. Their queue/import day is the last fallback.
    -- Explicit later date edits are unaffected; only new imports/new evidence are parsed.
    IF NEW.statement_id IS NULL AND NEW.notes LIKE 'Auto-captured via SMS%'
       AND (source_changed OR NEW.sms_source->>'parser_version' = '0') THEN
        received := (NEW.sms_source->>'received_at')::bigint;
        IF received BETWEEN 946684800000 AND (extract(epoch FROM now()) * 1000)::bigint THEN
            latest := least(latest, (to_timestamp(received / 1000.0) AT TIME ZONE 'Asia/Kolkata')::date);
        END IF;
        IF NEW.txn_date > latest OR NEW.value_date > latest THEN
            NEW.sms_source := coalesce(NEW.sms_source, '{}'::jsonb) || jsonb_build_object(
                'parser_version', coalesce((NEW.sms_source->>'parser_version')::int, 0),
                'previous_txn_date', NEW.txn_date, 'previous_value_date', NEW.value_date,
                'date_source', CASE WHEN received IS NULL THEN 'queue' ELSE 'message' END,
                'parsed_date', latest);
            IF NEW.txn_date > latest THEN NEW.txn_date := latest; END IF;
            IF NEW.value_date > latest THEN NEW.value_date := latest; END IF;
        END IF;
    END IF;

    -- An explicit restore is authoritative for all later automatic matchers.
    IF NEW.duplicate_override THEN RETURN NEW; END IF;

    -- Rescans can now supply a previously missing reference. Ignore the row itself.
    -- Never apply this to ordinary category/date edits or explicit duplicate restores.
    IF TG_OP = 'UPDATE' AND ref_changed AND is_named_bank(NEW.bank)
       AND btrim(NEW.bank_ref) ~ '^[0-9]{10,20}$' AND NEW.amount > 0 THEN
        SELECT array_agg(t.id) INTO matches FROM transactions t
        WHERE t.user_id = NEW.user_id AND t.id <> NEW.id AND NOT t.deleted
          AND upper(t.bank) = upper(NEW.bank) AND btrim(t.bank_ref) = btrim(NEW.bank_ref)
          AND t.amount = NEW.amount AND t.direction = NEW.direction
          AND abs(t.value_date - NEW.value_date) <= 2;
        IF cardinality(matches) = 1 THEN
            UPDATE transactions SET duplicate_of = matches[1], rev = rev + 1,
                version = version + 1, updated_at = now() WHERE duplicate_of = NEW.id;
            NEW.duplicate_of := matches[1]; NEW.deleted := true; RETURN NEW;
        END IF;
    END IF;

    -- No-reference repeat alerts need identical anchored text AND receipt within 2 minutes.
    -- Generic text, equal payee/amount/day and distant receipts remain separate payments.
    IF source_changed AND is_named_bank(NEW.bank) AND NEW.amount > 0
       AND length(NEW.sms_source->>'event_key') = 64
       AND NEW.sms_source->>'event_key' = NEW.sms_source->>'body_hash'
       AND NEW.sms_source->>'received_at' IS NOT NULL THEN
        SELECT array_agg(t.id) INTO matches FROM transactions t
        WHERE t.user_id = NEW.user_id AND t.id <> NEW.id AND NOT t.deleted
          AND upper(t.bank) = upper(NEW.bank) AND t.amount = NEW.amount
          AND t.direction = NEW.direction AND t.value_date = NEW.value_date
          AND t.sms_source->>'event_key' = NEW.sms_source->>'event_key'
          AND abs((t.sms_source->>'received_at')::bigint - (NEW.sms_source->>'received_at')::bigint) <= 120000
          -- Different references prove distinct payments, even if their text matches.
          AND (nullif(btrim(t.bank_ref),'') IS NULL OR nullif(btrim(NEW.bank_ref),'') IS NULL
               OR btrim(t.bank_ref) = btrim(NEW.bank_ref));
        IF cardinality(matches) = 1 THEN
            UPDATE transactions SET duplicate_of = matches[1], rev = rev + 1,
                version = version + 1, updated_at = now() WHERE duplicate_of = NEW.id;
            NEW.duplicate_of := matches[1]; NEW.deleted := true;
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
CREATE FUNCTION is_card_funding(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND t.direction='debit' AND is_named_bank(t.bank)
        AND (is_cred_funding(t) OR (t.is_transfer AND
            is_card_repayment(t.description,t.direction,t.bank='GENERIC_CC')));
$$;
CREATE FUNCTION is_card_funding_pair(f transactions,r transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT f.user_id=r.user_id AND f.id<>r.id AND f.amount=r.amount AND f.amount>0
        AND is_card_funding(f) AND is_card_receipt(r) AND abs(f.value_date-r.value_date)<=2
        AND r.sms_source->>'payment_role' IS DISTINCT FROM 'card_confirmation'
        AND f.sms_source->>'received_at' IS NOT NULL AND r.sms_source->>'received_at' IS NOT NULL
        AND abs((f.sms_source->>'received_at')::bigint-(r.sms_source->>'received_at')::bigint)<=600000;
$$;

-- Review may include a missing funding leg, but automatic exclusion below must
-- establish either a complete unique chain or explicit card identity and role.
CREATE OR REPLACE FUNCTION is_card_confirmation_candidate(a transactions,b transactions) RETURNS boolean LANGUAGE plpgsql STABLE AS $$
DECLARE delta bigint;
BEGIN
    IF a.user_id<>b.user_id OR a.id=b.id OR a.deleted OR b.deleted OR a.duplicate_override
        OR a.direction<>'credit' OR b.direction<>'credit' OR a.amount<>b.amount OR a.amount<=0
        OR upper(a.bank)<>upper(b.bank) OR abs(a.value_date-b.value_date)>2
        OR is_card_receipt(b) IS NOT TRUE OR b.sms_source->>'payment_role'='card_confirmation'
        OR (lower(trim(a.description))='transaction' OR is_card_receipt(a)) IS NOT TRUE
        OR a.sms_source->>'received_at' IS NULL OR b.sms_source->>'received_at' IS NULL THEN RETURN false; END IF;
    IF a.sms_source->>'card_account_key' IS NOT NULL AND b.sms_source->>'card_account_key' IS NOT NULL
        AND lower(a.sms_source->>'card_account_key')<>lower(b.sms_source->>'card_account_key') THEN RETURN false; END IF;
    delta := (a.sms_source->>'received_at')::bigint-(b.sms_source->>'received_at')::bigint;
    RETURN delta<=172800000 AND (delta>=600000 OR
        (a.sms_source->>'payment_role'='card_confirmation' AND delta>=-120000));
END;
$$;

CREATE OR REPLACE FUNCTION reconcile_card_repayments(owner_id uuid,paid numeric,dated date) RETURNS void LANGUAGE plpgsql AS $$
DECLARE pair record; root transactions; confirmation transactions;
    funding transactions; funding_count integer; debit_count integer;
    receipt_count integer; candidate_count integer; near_count integer; keyed_count integer;
    complete_chain boolean; keyed_confirmation boolean;
BEGIN
    PERFORM id FROM users WHERE id=owner_id FOR UPDATE;
    FOR pair IN
        SELECT f.id FROM transactions f JOIN transactions r ON is_card_funding_pair(f,r)
        WHERE f.user_id=owner_id AND f.amount=paid AND abs(f.value_date-dated)<=2
            AND NOT f.is_transfer AND NOT f.transfer_override
            AND NOT EXISTS (SELECT 1 FROM transactions other WHERE other.user_id=owner_id
                AND other.id<>f.id AND other.amount=paid AND is_card_funding_pair(other,r))
        GROUP BY f.id HAVING count(*)=1
    LOOP
        UPDATE transactions SET is_transfer=true,rev=rev+1,version=version+1,updated_at=now() WHERE id=pair.id;
    END LOOP;

    FOR root IN SELECT t.* FROM transactions t WHERE t.user_id=owner_id AND t.amount=paid
        AND abs(t.value_date-dated)<=2 AND is_card_receipt(t) AND NOT t.transfer_override
        AND t.sms_source->>'payment_role' IS DISTINCT FROM 'card_confirmation'
        ORDER BY (t.sms_source->>'received_at')::bigint,t.id
    LOOP
        IF NOT EXISTS (SELECT 1 FROM transactions t WHERE t.id=root.id AND NOT t.deleted) THEN CONTINUE; END IF;
        SELECT count(*) INTO funding_count FROM transactions f WHERE f.user_id=owner_id AND f.amount=paid AND is_card_funding_pair(f,root);
        SELECT count(*) INTO debit_count FROM transactions f WHERE f.user_id=owner_id AND NOT f.deleted
            AND f.direction='debit' AND f.amount=paid AND abs(f.value_date-root.value_date)<=2;
        SELECT count(*) INTO receipt_count FROM transactions r WHERE r.user_id=owner_id
            AND upper(r.bank)=upper(root.bank) AND r.amount=paid AND abs(r.value_date-root.value_date)<=2
            AND is_card_receipt(r) AND r.sms_source->>'payment_role' IS DISTINCT FROM 'card_confirmation';
        SELECT count(*) INTO candidate_count FROM transactions c WHERE c.user_id=owner_id
            AND c.amount=paid AND is_card_confirmation_candidate(c,root);
        near_count:=0;
        IF funding_count=1 THEN
            SELECT f.* INTO funding FROM transactions f WHERE f.user_id=owner_id AND f.amount=paid AND is_card_funding_pair(f,root);
            SELECT count(*) INTO near_count FROM transactions r WHERE r.user_id=owner_id AND r.amount=paid AND is_card_funding_pair(funding,r);
        END IF;
        SELECT count(*) INTO keyed_count FROM transactions r WHERE r.user_id=owner_id
            AND upper(r.bank)=upper(root.bank) AND r.amount=paid AND abs(r.value_date-root.value_date)<=2
            AND is_card_receipt(r) AND r.sms_source->>'payment_role'='card_receipt'
            AND lower(r.sms_source->>'card_account_key')=lower(root.sms_source->>'card_account_key');

        FOR confirmation IN SELECT c.* FROM transactions c WHERE c.user_id=owner_id AND c.amount=paid
            AND NOT c.transfer_override AND NOT c.duplicate_override AND is_card_confirmation_candidate(c,root)
        LOOP
            -- Distinct known references can describe separate payments.
            IF nullif(btrim(root.bank_ref),'') IS NOT NULL AND nullif(btrim(confirmation.bank_ref),'') IS NOT NULL
                AND upper(btrim(root.bank_ref))<>upper(btrim(confirmation.bank_ref)) THEN CONTINUE; END IF;
            complete_chain := funding_count=1 AND debit_count=1 AND near_count=1 AND NOT funding.transfer_override AND (
                -- Older imports have no card fingerprint. Require one receipt,
                -- one later generic alert and valid body dates on both messages.
                (receipt_count=1 AND candidate_count=1 AND lower(trim(confirmation.description))='transaction'
                    AND root.sms_source->>'date_source'='body' AND confirmation.sms_source->>'date_source'='body')
                OR (length(root.sms_source->>'card_account_key')=64
                    AND lower(root.sms_source->>'card_account_key')=lower(confirmation.sms_source->>'card_account_key'))
            );
            -- An explicit receipt/confirmation pair can be identified even when
            -- the account used to pay the bill is not present in this history.
            keyed_confirmation := root.sms_source->>'payment_role'='card_receipt'
                AND confirmation.sms_source->>'payment_role'='card_confirmation'
                AND length(root.sms_source->>'card_account_key')=64
                AND lower(root.sms_source->>'card_account_key')=lower(confirmation.sms_source->>'card_account_key')
                AND keyed_count=1 AND debit_count<=1 AND (debit_count=0 OR (funding_count=1 AND near_count=1 AND NOT funding.transfer_override));
            IF NOT coalesce(complete_chain,false) AND NOT coalesce(keyed_confirmation,false) THEN CONTINUE; END IF;
            UPDATE transactions SET duplicate_of=root.id,rev=rev+1,version=version+1,updated_at=now() WHERE duplicate_of=confirmation.id;
            UPDATE transactions SET duplicate_of=root.id,deleted=true,duplicate_reason='card_confirmation',
                rev=rev+1,version=version+1,updated_at=now() WHERE id=confirmation.id;
        END LOOP;
    END LOOP;
END;
$$;

CREATE FUNCTION is_card_evidence(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND (is_card_funding(t) OR is_card_receipt(t) OR
        (t.direction='credit' AND lower(trim(t.description))='transaction' AND t.sms_source->>'received_at' IS NOT NULL));
$$;
CREATE OR REPLACE FUNCTION reconcile_new_card_evidence() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM reconcile_card_repayments(NEW.user_id,NEW.amount,NEW.value_date);
    RETURN NEW;
END;
$$;
DROP TRIGGER transaction_card_evidence ON transactions;
-- Include a generic confirmation arriving in a later upload. Changes made by
-- reconciliation itself do not enqueue more work or affect earlier migrations.
CREATE CONSTRAINT TRIGGER transaction_card_evidence AFTER INSERT ON transactions
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (is_card_evidence(NEW)) EXECUTE FUNCTION reconcile_new_card_evidence();
CREATE CONSTRAINT TRIGGER transaction_card_evidence_updated AFTER UPDATE ON transactions
DEFERRABLE INITIALLY DEFERRED FOR EACH ROW WHEN (is_card_evidence(NEW) AND (
    NEW.sms_source IS DISTINCT FROM OLD.sms_source OR NEW.description IS DISTINCT FROM OLD.description
    OR NEW.bank_ref IS DISTINCT FROM OLD.bank_ref OR NEW.bank IS DISTINCT FROM OLD.bank
    OR NEW.amount IS DISTINCT FROM OLD.amount OR NEW.direction IS DISTINCT FROM OLD.direction
    OR NEW.value_date IS DISTINCT FROM OLD.value_date)) EXECUTE FUNCTION reconcile_new_card_evidence();

ALTER TABLE transactions NO FORCE ROW LEVEL SECURITY;
WITH candidates AS MATERIALIZED (
    SELECT DISTINCT user_id,amount,value_date FROM transactions t WHERE is_card_receipt(t)
)
SELECT reconcile_card_repayments(user_id,amount,value_date) FROM candidates;
SET CONSTRAINTS transaction_card_evidence,transaction_card_evidence_updated IMMEDIATE;
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;
SET CONSTRAINTS transaction_card_evidence,transaction_card_evidence_updated DEFERRED;
