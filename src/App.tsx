import './App.css'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import { AddressEntryCreatePage } from './features/address/AddressEntryCreatePage'
import { AddressEntryListPage } from './features/address/AddressEntryListPage'
import { AddressEntryDetailPage } from './features/address/AddressEntryDetailPage'
import { AddressEntryEditPage } from './features/address/AddressEntryEditPage'

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
            <Route
              path="/addresses/:id/edit"
              element={<AddressEntryEditPage />}
            />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}

export default App
