import './App.css'
import { createBrowserRouter, NavLink, Outlet, RouterProvider } from 'react-router-dom'
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
import { PrintSelectPage } from './features/print/pages/PrintSelectPage'
import { PrintConfirmPage } from './features/print/pages/PrintConfirmPage'
import { PrintPreviewPage } from './features/print/pages/PrintPreviewPage'

function AppLayout() {
  const navigationItems = [
    { to: '/addresses', label: '住所録一覧' },
    { to: '/addresses/new', label: '住所録新規作成' },
    { to: '/senders', label: '差出人一覧' },
    { to: '/senders/new', label: '差出人新規作成' },
    { to: '/receipts', label: '受取履歴一覧' },
    { to: '/receipts/new', label: '受取履歴新規作成' },
    { to: '/print/select', label: '宛名印刷' },
  ]

  return (
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
        <Outlet />
      </main>
    </div>
  )
}

const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      { index: true, element: <AddressEntryListPage /> },
      { path: 'addresses', element: <AddressEntryListPage /> },
      { path: 'addresses/new', element: <AddressEntryCreatePage /> },
      { path: 'addresses/:id', element: <AddressEntryDetailPage /> },
      { path: 'addresses/:id/edit', element: <AddressEntryEditPage /> },
      { path: 'senders', element: <SenderEntryListPage /> },
      { path: 'senders/new', element: <SenderEntryCreatePage /> },
      { path: 'senders/:id', element: <SenderEntryDetailPage /> },
      { path: 'senders/:id/edit', element: <SenderEntryEditPage /> },
      { path: 'receipts', element: <PostcardReceiptListPage /> },
      { path: 'receipts/new', element: <PostcardReceiptCreatePage /> },
      { path: 'receipts/:id', element: <PostcardReceiptDetailPage /> },
      { path: 'receipts/:id/edit', element: <PostcardReceiptEditPage /> },
      { path: 'print/select', element: <PrintSelectPage /> },
      { path: 'print/confirm', element: <PrintConfirmPage /> },
      { path: 'print/preview', element: <PrintPreviewPage /> },
    ],
  },
])

function App() {
  return <RouterProvider router={router} />
}

export default App
