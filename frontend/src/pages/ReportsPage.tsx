import { useState, useEffect } from 'react'
import { api, getServerUrl } from '../api/client'
import { Download, Shield, FileSpreadsheet } from 'lucide-react'
import { Screen, Card, CardBody, ListRow, ListRowText, Button } from '../components/shared'

interface TaxItem {
  section: string
  description: string
  amount: number
  txn_date: string
}

interface TaxSummary {
  total_80c_eligible: number
  total_80d_medical: number
  total_charity_80g: number
  breakdown: TaxItem[]
}

export function ReportsPage() {
  const [taxData, setTaxData] = useState<TaxSummary | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    api.get<TaxSummary>('/reports/tax-summary')
      .then(res => setTaxData(res.data))
      .catch(err => console.error(err))
      .finally(() => setLoading(false))
  }, [])

  const handleDownloadCsv = () => {
    const baseUrl = getServerUrl() || window.location.origin
    window.open(`${baseUrl}/api/reports/export/csv`, '_blank')
  }

  return (
    <div style={{ maxWidth: 650, margin: '0 auto' }}>
      <Screen
        title="Reports & Export"
        subtitle="Export financial statements & tax readiness summary"
      >
        <div style={{ marginBottom: 16 }}>
          <Card>
            <CardBody>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  <div style={{ width: 36, height: 36, borderRadius: 8, background: 'var(--income-soft)', color: 'var(--income)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    <FileSpreadsheet size={18} />
                  </div>
                  <div>
                    <div style={{ fontWeight: 600, fontSize: 14 }}>Export All Transactions</div>
                    <div style={{ fontSize: 12, color: 'var(--text-2)' }}>Download your complete transaction ledger as CSV format</div>
                  </div>
                </div>
                <Button variant="primary" size="sm" onClick={handleDownloadCsv}>
                  <Download size={14} /> Download CSV
                </Button>
              </div>
            </CardBody>
          </Card>
        </div>

        <h3 style={{ fontSize: 15, fontWeight: 600, marginBottom: 10 }}>Tax Readiness Summary (India IT Act)</h3>

        {loading ? (
          <div>Loading tax summary...</div>
        ) : !taxData ? (
          <Card><CardBody>Failed to load tax summary</CardBody></Card>
        ) : (
          <div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 10, marginBottom: 16 }}>
              <Card>
                <CardBody>
                  <div style={{ fontSize: 11, color: 'var(--text-2)' }}>Section 80C (ELSS/PPF)</div>
                  <div style={{ fontSize: 16, fontWeight: 700, color: 'var(--brand)', marginTop: 2 }}>₹{taxData.total_80c_eligible.toLocaleString('en-IN')}</div>
                  <div style={{ fontSize: 10, color: 'var(--text-muted)', marginTop: 2 }}>Max ₹1,50,000</div>
                </CardBody>
              </Card>

              <Card>
                <CardBody>
                  <div style={{ fontSize: 11, color: 'var(--text-2)' }}>Section 80D (Health Ins)</div>
                  <div style={{ fontSize: 16, fontWeight: 700, color: 'var(--income)', marginTop: 2 }}>₹{taxData.total_80d_medical.toLocaleString('en-IN')}</div>
                  <div style={{ fontSize: 10, color: 'var(--text-muted)', marginTop: 2 }}>Medical & Insurance</div>
                </CardBody>
              </Card>

              <Card>
                <CardBody>
                  <div style={{ fontSize: 11, color: 'var(--text-2)' }}>Section 80G (Donations)</div>
                  <div style={{ fontSize: 16, fontWeight: 700, color: 'var(--warn)', marginTop: 2 }}>₹{taxData.total_charity_80g.toLocaleString('en-IN')}</div>
                  <div style={{ fontSize: 10, color: 'var(--text-muted)', marginTop: 2 }}>Charity</div>
                </CardBody>
              </Card>
            </div>

            <Card>
              <CardBody>
                <div style={{ fontWeight: 600, fontSize: 13, marginBottom: 10 }}>Categorized Tax Deductible Items ({taxData.breakdown.length})</div>
                {taxData.breakdown.length === 0 ? (
                  <div style={{ fontSize: 12, color: 'var(--text-2)', padding: '12px 0' }}>No tax deductible transactions found in your records.</div>
                ) : (
                  taxData.breakdown.map((item, idx) => (
                    <ListRow
                      key={idx}
                      leading={<Shield size={16} style={{ color: 'var(--brand)' }} />}
                      trailing={<span style={{ fontWeight: 600, fontSize: 13 }}>₹{item.amount.toLocaleString('en-IN')}</span>}
                    >
                      <ListRowText primary={item.description} secondary={`${item.section} • ${item.txn_date}`} />
                    </ListRow>
                  ))
                )}
              </CardBody>
            </Card>
          </div>
        )}
      </Screen>
    </div>
  )
}
