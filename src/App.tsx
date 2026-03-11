import { useState } from 'react'
import './App.css'
import { AddressEntryCreatePage } from './features/address/AddressEntryCreatePage.tsx'
import { AddressEntryListPage } from './features/address/AddressEntryListPage.tsx'

type Page = 'list' | 'create'

function App() {
  const [page, setPage] = useState<Page>('list')

  const handleNavigateToList = () => {
    setPage('list')
  }

  const handleNavigateToCreate = () => {
    setPage('create')
  }

  const title = page === 'list' ? '住所録一覧' : '住所録新規作成'

  return (
    <div className="app-root">
      <header className="app-header">
        <h1 className="app-title">{title}</h1>
      </header>
      <main className="app-main">
        {page === 'list' ? (
          <AddressEntryListPage
            onClickCreate={handleNavigateToCreate}
            onSelectDetail={(id) => {
              // eslint-disable-next-line no-alert
              alert(`詳細画面（ADDR004）は今後実装予定です。\nID: ${id}`)
            }}
          />
        ) : (
          <AddressEntryCreatePage
            onCreated={handleNavigateToList}
            onCancel={handleNavigateToList}
          />
        )}
      </main>
    </div>
  )
}

export default App
