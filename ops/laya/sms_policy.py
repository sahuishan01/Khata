"""Pure Laya SMS decision policy; no network or heavyweight runtime imports."""
import hashlib
import re

MODEL_REVISION = "7b928d828b7b0e022f929d9bd2e44165aa270148"
POLICY_VERSION = "sms-choice-v1"
MODEL_VERSION = f"laya:{MODEL_REVISION}:{POLICY_VERSION}"
OTP = re.compile(r"\b(?:one[ -]?time[ -]?(?:password|passcode|code)|login[ -]?code|verification[ -]?code|authentication[ -]?code)\b", re.I)
TRANSACTION = {"transaction": {"type": "choice", "instructions": "Does the SMS report a completed movement of money?", "criteria": {
    "yes": "A payment, debit, credit, deposit, withdrawal or refund already happened.",
    "no": "A request, advertisement, future transaction, failed payment or balance notice.",
}}}
DEBIT = {"debit": {"type": "choice", "instructions": "Which direction did money move for the account described by this SMS?", "criteria": {
    "debit": "Money left this account: debited, paid, spent, sent or withdrawn.",
    "credit": "Money arrived in this account: credited, received, refunded or deposited.",
}}}
DEBIT_VIEWS = [DEBIT,
    {"debit": {"type": "choice", "instructions": "Is this SMS a debit transaction?", "criteria": {
        "yes": "Money left the customer account. It was sent, spent, paid, deducted, withdrawn or transferred out.",
        "no": "Money arrived in the customer account. It was received, credited, deposited or refunded.",
    }}},
    {"debit": {"type": "choice", "instructions": "Was money debited from the customer account?", "criteria": {
        "yes": "A debit: money was sent or paid out.", "no": "A credit: money was received.",
    }}},
]


def contains_otp(body):
    return "otp" in body.lower() or OTP.search(body) is not None


def cpu_idle_enough(before, after):
    delta = [end - start for start, end in zip(before[:8], after[:8])]
    return len(delta) == 8 and min(delta) >= 0 and sum(delta) > 0 and (delta[3] + delta[4]) / sum(delta) >= .8


def decide(agent, body):
    # This check is deliberately before the first call into the model.
    if contains_otp(body):
        return {"status": "blocked", "is_transaction": None, "is_debit": None,
                "transaction_probability": None, "debit_probability": None,
                "source_hash": hashlib.sha256(body.encode()).hexdigest(), "model_version": MODEL_VERSION}
    result = {"status": "review", "is_transaction": None, "is_debit": None,
              "transaction_probability": None, "debit_probability": None,
              "source_hash": hashlib.sha256(body.encode()).hexdigest(), "model_version": MODEL_VERSION}
    # Bound CPU inference without silently truncating a later negation/posting.
    if len(body) > 2000:
        return result
    answer = agent.predict(body, TRANSACTION)["answers"]["transaction"]
    probability = float(answer["probabilities"]["yes"])
    if not 0 <= probability <= 1:
        raise ValueError("Invalid probability")
    result["transaction_probability"] = probability
    if probability <= .2:
        result.update(status="ignored", is_transaction=False)
        return result  # No debit question for a negative transaction decision.
    if probability < .75:
        return result
    result["is_transaction"] = True
    # Three phrasings answer the same second question. Average their scores to
    # reduce sensitivity to one phrasing; never ask direction for a rejected SMS.
    probabilities = []
    for question in DEBIT_VIEWS:
        answer = agent.predict(body, question)["answers"]["debit"]["probabilities"]
        probability = float(answer.get("debit", answer.get("yes")))
        if not 0 <= probability <= 1:
            raise ValueError("Invalid probability")
        probabilities.append(probability)
    probability = sum(probabilities) / len(probabilities)
    if not 0 <= probability <= 1:
        raise ValueError("Invalid probability")
    result["debit_probability"] = probability
    if max(probability, 1 - probability) < .55:
        return result
    result.update(status="decision", is_debit=probability > .5)
    return result
