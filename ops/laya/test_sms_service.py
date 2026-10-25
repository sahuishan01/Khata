import unittest
from sms_policy import contains_otp, cpu_idle_enough, decide

class FakeAgent:
    def __init__(self, transaction=.95, debit=.9):
        self.transaction = transaction
        self.debit = debit
        self.calls = []
    def predict(self, body, questions):
        name = next(iter(questions)); self.calls.append(name)
        probability = self.transaction if name == 'transaction' else self.debit
        key = 'yes' if name == 'transaction' or 'yes' in questions[name]['criteria'] else 'debit'
        return {'answers': {name: {'probabilities': {key: probability}}}}

class DecisionsTest(unittest.TestCase):
    def test_inference_requires_eighty_percent_cpu_idle(self):
        self.assertTrue(cpu_idle_enough([0]*8,[20,0,0,80,0,0,0,0]))
        self.assertFalse(cpu_idle_enough([0]*8,[21,0,0,79,0,0,0,0]))
        self.assertFalse(cpu_idle_enough([0]*8,[0]*8))
    def test_otp_never_reaches_model(self):
        for body in ['OTP 123456 for INR 500', 'Your otp is 123456', 'One-time password 123456', 'Login code 123456']:
            agent = FakeAgent()
            self.assertTrue(contains_otp(body))
            self.assertEqual(decide(agent, body)['status'], 'blocked')
            self.assertEqual(agent.calls, [])
    def test_nontransaction_does_not_ask_debit(self):
        agent=FakeAgent(transaction=.05)
        result=decide(agent,'Your statement is available')
        self.assertEqual(result['status'],'ignored'); self.assertFalse(result['is_transaction'])
        self.assertEqual(agent.calls,['transaction']); self.assertIsNone(result['is_debit'])
    def test_credit_is_an_explicit_confident_no_to_debit(self):
        agent=FakeAgent(debit=.05); result=decide(agent,'INR 500 credited')
        self.assertEqual(agent.calls,['transaction','debit','debit','debit']); self.assertEqual(result['status'],'decision')
        self.assertFalse(result['is_debit'])
    def test_uncertainty_is_not_silently_a_credit(self):
        agent=FakeAgent(debit=.5); result=decide(agent,'Payment notice')
        self.assertEqual(result['status'],'review'); self.assertIsNone(result['is_debit'])
    def test_uncertain_transaction_stops_before_direction(self):
        agent=FakeAgent(transaction=.5); result=decide(agent,'Payment request')
        self.assertIsNone(result['is_transaction']); self.assertEqual(agent.calls,['transaction'])
    def test_result_is_bound_to_source(self):
        result=decide(FakeAgent(),'INR 500 debited')
        self.assertEqual(len(result['source_hash']),64); self.assertTrue(result['model_version'].startswith('laya:'))
    def test_long_messages_are_reviewed_without_truncation_or_inference(self):
        agent=FakeAgent(); result=decide(agent,'A'*2001)
        self.assertEqual(result['status'],'review'); self.assertEqual(agent.calls,[])

if __name__=='__main__':unittest.main()
