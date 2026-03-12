import './App.css'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { AddressEntryCreatePage } from './features/address/AddressEntryCreatePage.tsx'
import { AddressEntryListPage } from './features/address/AddressEntryListPage.tsx'
import { AddressEntryDetailPage } from './features/address/AddressEntryDetailPage.tsx'

function App() {
  return (
    <BrowserRouter>
      <div className="app-root">
        <main className="app-main">
          <Routes>
            <Route
              path="/"
              element={<AddressEntryListPage />}
            />
            <Route
              path="/addresses"
              element={<AddressEntryListPage />}
            />
            <Route
              path="/addresses/new"
              element={<AddressEntryCreatePage />}
            />
            <Route
              path="/addresses/:id"
              element={<AddressEntryDetailPage />}
            />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}

export default App
