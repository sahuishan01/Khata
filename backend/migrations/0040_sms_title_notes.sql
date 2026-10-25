-- SMS imports now put their capture label in the title and exact message in notes.
-- No stored rows are rewritten; inbox rescans swap only verified automatic fields.
CREATE FUNCTION original_sms_text(t transactions) RETURNS text LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN length(t.sms_source->>'body_hash')=64 THEN
        CASE WHEN has_original_sms_description(t) THEN t.description
        WHEN t.description = 'Auto-captured via SMS (' || t.bank || ')'
            AND encode(sha256(convert_to(lower(btrim(regexp_replace(t.notes, '\s+', ' ', 'g'))), 'UTF8')), 'hex')
                = lower(t.sms_source->>'body_hash') THEN t.notes END END;
$$;

CREATE OR REPLACE FUNCTION is_cred_funding(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND coalesce(t.sms_source->>'user_edited','false') <> 'true' AND t.direction = 'debit' AND is_named_bank(t.bank)
      AND trim(regexp_replace(upper(CASE WHEN t.sms_source->>'parsed_payee' IS NOT NULL AND original_sms_text(t) IS NOT NULL
          THEN t.sms_source->>'parsed_payee' ELSE t.description END), '[^A-Z0-9]+', ' ', 'g')) IN ('CRED','CRED CLUB');
$$;
CREATE OR REPLACE FUNCTION is_card_receipt(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND coalesce(t.sms_source->>'user_edited','false') <> 'true' AND t.direction = 'credit' AND t.is_transfer
      AND (is_card_repayment(coalesce(original_sms_text(t),t.description), t.direction, t.bank = 'GENERIC_CC')
           OR t.sms_source->>'payment_role' IN ('card_receipt','card_confirmation'));
$$;
CREATE OR REPLACE FUNCTION is_card_funding(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT NOT t.deleted AND coalesce(t.sms_source->>'user_edited','false') <> 'true' AND t.direction='debit' AND is_named_bank(t.bank)
        AND (is_cred_funding(t) OR (t.is_transfer AND
            is_card_repayment(coalesce(original_sms_text(t),t.description),t.direction,t.bank='GENERIC_CC')));
$$;
CREATE OR REPLACE FUNCTION is_generic_sms_confirmation(t transactions) RETURNS boolean LANGUAGE sql IMMUTABLE AS $$
    SELECT coalesce(t.sms_source->>'user_edited','false') <> 'true' AND (
        lower(trim(t.description))='transaction' OR (t.sms_source->>'payment_role' IS NULL
            AND lower(trim(t.sms_source->>'parsed_payee'))='transaction' AND original_sms_text(t) IS NOT NULL
            AND original_sms_text(t) !~* '\m(REFUND[A-Z]*|REVERS[A-Z]*|CASHBACK[A-Z]*|CHARGEBACK[A-Z]*|SALAR[A-Z]*|DIVIDEND[A-Z]*|BONUS[A-Z]*|INSURANCE|INTEREST[A-Z]*|LOAN[A-Z]*|DISBURS[A-Z]*|IPO[A-Z]*)\M'));
$$;

CREATE OR REPLACE FUNCTION protect_sms_evidence() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    latest date := (now() AT TIME ZONE 'Asia/Kolkata')::date;
    received bigint;
    matches uuid[];
    source_changed boolean := TG_OP = 'INSERT';
    ref_changed boolean := TG_OP = 'INSERT';
BEGIN
    IF NEW.sms_source->>'user_edited' = 'true' THEN RETURN NEW; END IF;
    PERFORM id FROM users WHERE id = NEW.user_id FOR UPDATE;
    IF TG_OP = 'UPDATE' THEN
        source_changed := NEW.sms_source IS DISTINCT FROM OLD.sms_source;
        ref_changed := NEW.bank_ref IS DISTINCT FROM OLD.bank_ref;
    END IF;
    IF NEW.deleted OR NEW.duplicate_of IS NOT NULL THEN RETURN NEW; END IF;

    -- Old clients have no provenance. Their queue/import day is the last fallback.
    -- Explicit later date edits are unaffected; only new imports/new evidence are parsed.
    IF NEW.statement_id IS NULL AND (NEW.notes LIKE 'Auto-captured via SMS%' OR
        (NEW.description = 'Auto-captured via SMS (' || NEW.bank || ')' AND original_sms_text(NEW) IS NOT NULL))
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
