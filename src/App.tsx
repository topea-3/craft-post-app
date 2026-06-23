import './App.css'
import { BrowserRouter, NavLink, Route, Routes } from 'react-router-dom'
import { AddressEntryCreatePage } from './features/address/AddressEntryCreatePage'
import { AddressEntryListPage } from './features/address/AddressEntryListPage'
import { AddressEntryDetailPage } from './features/address/AddressEntryDetailPage'
import { AddressEntryEditPage } from './features/address/AddressEntryEditPage'
import { SenderEntryCreatePage } from './features/sender/SenderEntryCreatePage'
import { SenderEntryListPage } from './features/sender/SenderEntryListPage'
import { SenderEntryDetailPage } from './features/sender/SenderEntryDetailPage'
import { SenderEntryEditPage } from './features/sender/SenderEntryEditPage'
import { PostcardReceiptListPage } from './features/postcard-receipt/PostcardReceiptListPage'
import { PostcardReceiptCreatePage } from './features/postcard-receipt/PostcardReceiptCreatePage'
import { PostcardReceiptDetailPage } from './features/postcard-receipt/PostcardReceiptDetailPage'
import { PostcardReceiptEditPage } from './features/postcard-receipt/PostcardReceiptEditPage'

function App() {
  const navigationItems = [
    { to: '/addresses', label: '住所録一覧' },
    { to: '/addresses/new', label: '住所録新規作成' },
    { to: '/senders', label: '差出人一覧' },
    { to: '/senders/new', label: '差出人新規作成' },
    { to: '/receipts', label: '受取履歴一覧' },
    { to: '/receipts/new', label: '受取履歴新規作成' },
  ]

  return (
    <BrowserRouter>
      <div className="app-layout">
        <aside className="app-sidebar" aria-label="サイドメニュー">
          <h1 className="app-sidebar-title">メニュー</h1>
          <nav className="app-nav">
            {navigationItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  `app-nav-link${isActive ? ' app-nav-link-active' : ''}`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </aside>
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
            <Route path="/senders" element={<SenderEntryListPage />} />
            <Route path="/senders/new" element={<SenderEntryCreatePage />} />
            <Route path="/senders/:id" element={<SenderEntryDetailPage />} />
            <Route path="/senders/:id/edit" element={<SenderEntryEditPage />} />
            <Route path="/receipts" element={<PostcardReceiptListPage />} />
            <Route path="/receipts/new" element={<PostcardReceiptCreatePage />} />
            <Route path="/receipts/:id" element={<PostcardReceiptDetailPage />} />
            <Route path="/receipts/:id/edit" element={<PostcardReceiptEditPage />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  )
}

export default App
