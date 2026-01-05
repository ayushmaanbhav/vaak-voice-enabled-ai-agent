import { useState, useEffect } from 'react'
import { MicButton } from './components/MicButton'
import { Transcript } from './components/Transcript'
import { useVoiceAgent } from './hooks/useVoiceAgent'
import SimplePTT from './pages/SimplePTT'

// Kotak brand colors
const KOTAK_RED = '#ED1C24'

// Simple hash-based router
function useHashRoute() {
  const [route, setRoute] = useState(window.location.hash.slice(1) || '/')

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(window.location.hash.slice(1) || '/')
    }
    window.addEventListener('hashchange', handleHashChange)
    return () => window.removeEventListener('hashchange', handleHashChange)
  }, [])

  return route
}

interface Customer {
  id: string
  name: string
  language: string
  segment: string
  current_provider: string
  estimated_outstanding: number
  estimated_rate: number
  city: string
}

const LANGUAGES = {
  hi: { name: 'Hindi', native: 'हिंदी' },
  ta: { name: 'Tamil', native: 'தமிழ்' },
  te: { name: 'Telugu', native: 'తెలుగు' },
  kn: { name: 'Kannada', native: 'ಕನ್ನಡ' },
  ml: { name: 'Malayalam', native: 'മലയാളം' },
  en: { name: 'English', native: 'English' },
}

function App() {
  const route = useHashRoute()

  // Render SimplePTT for /simple route
  if (route === '/simple') {
    return <SimplePTT />
  }

  // Render main app for all other routes
  return <MainApp />
}

function MainApp() {
  const [customers, setCustomers] = useState<Customer[]>([])
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null)
  const [selectedLanguage, setSelectedLanguage] = useState('hi')
  const [isStarted, setIsStarted] = useState(false)

  const {
    isConnected,
    isRecording,
    transcript,
    startConversation,
    startRecording,
    stopRecording,
    endConversation,
  } = useVoiceAgent()

  // Fetch customers on mount
  useEffect(() => {
    fetch('/api/customers')
      .then(res => res.json())
      .then(data => setCustomers(data.customers))
      .catch(err => {
        console.error('Failed to fetch customers:', err)
        // Use mock data if API not available
        setCustomers([
          { id: 'C001', name: 'Rajesh Kumar', language: 'hi', segment: 'high_value', current_provider: 'muthoot', estimated_outstanding: 800000, estimated_rate: 18, city: 'Mumbai' },
          { id: 'C005', name: 'Lakshmi Devi', language: 'ta', segment: 'shakti', current_provider: 'manappuram', estimated_outstanding: 200000, estimated_rate: 22, city: 'Chennai' },
          { id: 'C003', name: 'Venkat Rao', language: 'te', segment: 'trust_seeker', current_provider: 'iifl', estimated_outstanding: 500000, estimated_rate: 20, city: 'Hyderabad' },
        ])
      })
  }, [])

  const handleStart = async () => {
    if (!selectedCustomer) return
    setIsStarted(true)
    await startConversation(selectedCustomer.id, selectedLanguage)
  }

  const handleEnd = () => {
    endConversation()
    setIsStarted(false)
  }

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency: 'INR',
      maximumFractionDigits: 0,
    }).format(amount)
  }

  const calculateSavings = (customer: Customer) => {
    const currentInterest = customer.estimated_outstanding * (customer.estimated_rate / 100)
    const kotakInterest = customer.estimated_outstanding * 0.10
    return currentInterest - kotakInterest
  }

  return (
    <div style={styles.container}>
      {/* Header */}
      <header style={styles.header}>
        <div style={styles.logo}>
          <span style={styles.logoText}>KOTAK</span>
          <span style={styles.logoSubtext}>Gold Loan Voice Agent</span>
        </div>
        <a href="#/simple" style={styles.simpleModeLink}>Simple Mode →</a>
      </header>

      <main style={styles.main}>
        {!isStarted ? (
          /* Setup Screen */
          <div style={styles.setupContainer}>
            <h1 style={styles.title}>Voice Agent Demo</h1>
            <p style={styles.subtitle}>
              Select a customer profile to start a personalized gold loan pitch conversation
            </p>

            {/* Customer Selection */}
            <div style={styles.card}>
              <h2 style={styles.cardTitle}>Select Customer</h2>
              <div style={styles.customerGrid}>
                {customers.map(customer => (
                  <div
                    key={customer.id}
                    style={{
                      ...styles.customerCard,
                      ...(selectedCustomer?.id === customer.id ? styles.customerCardSelected : {}),
                    }}
                    onClick={() => {
                      setSelectedCustomer(customer)
                      setSelectedLanguage(customer.language)
                    }}
                  >
                    <div style={styles.customerName}>{customer.name}</div>
                    <div style={styles.customerDetails}>
                      <span style={styles.badge}>{customer.segment.replace('_', ' ')}</span>
                      <span style={styles.badgeProvider}>{customer.current_provider}</span>
                    </div>
                    <div style={styles.customerMeta}>
                      <div>Outstanding: {formatCurrency(customer.estimated_outstanding)}</div>
                      <div>Rate: {customer.estimated_rate}%</div>
                      <div style={styles.savings}>
                        Potential Savings: {formatCurrency(calculateSavings(customer))}/yr
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Language Selection */}
            <div style={styles.card}>
              <h2 style={styles.cardTitle}>Select Language</h2>
              <div style={styles.languageGrid}>
                {Object.entries(LANGUAGES).map(([code, lang]) => (
                  <button
                    key={code}
                    style={{
                      ...styles.languageButton,
                      ...(selectedLanguage === code ? styles.languageButtonSelected : {}),
                    }}
                    onClick={() => setSelectedLanguage(code)}
                  >
                    <span style={styles.languageNative}>{lang.native}</span>
                    <span style={styles.languageName}>{lang.name}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Start Button */}
            <button
              style={{
                ...styles.startButton,
                ...(selectedCustomer ? {} : styles.startButtonDisabled),
              }}
              onClick={handleStart}
              disabled={!selectedCustomer}
            >
              Start Conversation
            </button>
          </div>
        ) : (
          /* Conversation Screen */
          <div style={styles.conversationContainer}>
            {/* Customer Info Bar */}
            <div style={styles.customerInfoBar}>
              <div>
                <strong>{selectedCustomer?.name}</strong> ({selectedCustomer?.segment.replace('_', ' ')})
              </div>
              <div>
                {LANGUAGES[selectedLanguage as keyof typeof LANGUAGES]?.native} | {selectedCustomer?.city}
              </div>
            </div>

            {/* Transcript */}
            <Transcript messages={transcript} />

            {/* Controls */}
            <div style={styles.controls}>
              <MicButton
                isRecording={isRecording}
                onStart={startRecording}
                onStop={stopRecording}
                disabled={!isConnected}
              />
              <button style={styles.endButton} onClick={handleEnd}>
                End Conversation
              </button>
            </div>

            {/* Connection Status */}
            <div style={styles.status}>
              {isConnected ? '🟢 Connected' : '🔴 Connecting...'}
              {isRecording && ' | 🎙️ Recording...'}
            </div>
          </div>
        )}
      </main>
    </div>
  )
}

const styles: { [key: string]: React.CSSProperties } = {
  container: {
    minHeight: '100vh',
    color: '#fff',
  },
  header: {
    background: KOTAK_RED,
    padding: '1rem 2rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  simpleModeLink: {
    color: '#fff',
    textDecoration: 'none',
    fontSize: '0.9rem',
    opacity: 0.9,
    padding: '0.5rem 1rem',
    background: 'rgba(255,255,255,0.15)',
    borderRadius: '6px',
    transition: 'background 0.2s',
  },
  logo: {
    display: 'flex',
    flexDirection: 'column',
  },
  logoText: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    letterSpacing: '2px',
  },
  logoSubtext: {
    fontSize: '0.75rem',
    opacity: 0.9,
  },
  main: {
    maxWidth: '1200px',
    margin: '0 auto',
    padding: '2rem',
  },
  setupContainer: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1.5rem',
  },
  title: {
    fontSize: '2rem',
    textAlign: 'center',
    marginBottom: '0.5rem',
  },
  subtitle: {
    textAlign: 'center',
    opacity: 0.8,
    marginBottom: '1rem',
  },
  card: {
    background: 'rgba(255,255,255,0.05)',
    borderRadius: '12px',
    padding: '1.5rem',
  },
  cardTitle: {
    fontSize: '1.1rem',
    marginBottom: '1rem',
    opacity: 0.9,
  },
  customerGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
    gap: '1rem',
  },
  customerCard: {
    background: 'rgba(255,255,255,0.05)',
    borderRadius: '8px',
    padding: '1rem',
    cursor: 'pointer',
    border: '2px solid transparent',
    transition: 'all 0.2s',
  },
  customerCardSelected: {
    borderColor: KOTAK_RED,
    background: 'rgba(237,28,36,0.1)',
  },
  customerName: {
    fontSize: '1.1rem',
    fontWeight: 'bold',
    marginBottom: '0.5rem',
  },
  customerDetails: {
    display: 'flex',
    gap: '0.5rem',
    marginBottom: '0.5rem',
  },
  badge: {
    background: 'rgba(255,255,255,0.1)',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
    textTransform: 'capitalize',
  },
  badgeProvider: {
    background: 'rgba(237,28,36,0.3)',
    padding: '0.25rem 0.5rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
    textTransform: 'capitalize',
  },
  customerMeta: {
    fontSize: '0.85rem',
    opacity: 0.8,
    lineHeight: 1.6,
  },
  savings: {
    color: '#4ade80',
    fontWeight: 'bold',
  },
  languageGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(6, 1fr)',
    gap: '0.75rem',
  },
  languageButton: {
    background: 'rgba(255,255,255,0.05)',
    border: '2px solid transparent',
    borderRadius: '8px',
    padding: '0.75rem',
    cursor: 'pointer',
    color: '#fff',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    gap: '0.25rem',
    transition: 'all 0.2s',
  },
  languageButtonSelected: {
    borderColor: KOTAK_RED,
    background: 'rgba(237,28,36,0.2)',
  },
  languageNative: {
    fontSize: '1.1rem',
    fontWeight: 'bold',
  },
  languageName: {
    fontSize: '0.7rem',
    opacity: 0.7,
  },
  startButton: {
    background: KOTAK_RED,
    color: '#fff',
    border: 'none',
    borderRadius: '8px',
    padding: '1rem 2rem',
    fontSize: '1.1rem',
    fontWeight: 'bold',
    cursor: 'pointer',
    transition: 'all 0.2s',
    alignSelf: 'center',
  },
  startButtonDisabled: {
    opacity: 0.5,
    cursor: 'not-allowed',
  },
  conversationContainer: {
    display: 'flex',
    flexDirection: 'column',
    height: 'calc(100vh - 150px)',
    gap: '1rem',
  },
  customerInfoBar: {
    background: 'rgba(255,255,255,0.05)',
    padding: '1rem',
    borderRadius: '8px',
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '0.9rem',
  },
  controls: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    gap: '1rem',
    padding: '1rem',
  },
  endButton: {
    background: 'rgba(255,255,255,0.1)',
    color: '#fff',
    border: '1px solid rgba(255,255,255,0.2)',
    borderRadius: '8px',
    padding: '0.75rem 1.5rem',
    cursor: 'pointer',
    fontSize: '0.9rem',
  },
  status: {
    textAlign: 'center',
    fontSize: '0.85rem',
    opacity: 0.7,
  },
}

export default App
