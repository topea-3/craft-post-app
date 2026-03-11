import './App.css'
import { AddressEntryCreatePage } from './features/address/AddressEntryCreatePage'

function App() {
  return (
    <div className="app-root">
      <header className="app-header">
        <h1 className="app-title">住所録新規作成</h1>
      </header>
      <main className="app-main">
        <AddressEntryCreatePage />
      </main>
    </div>
  )
}

export default App
