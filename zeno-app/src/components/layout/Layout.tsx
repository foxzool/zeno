import { ReactNode } from 'react'
import Sidebar from './Sidebar'

interface LayoutProps {
  children: ReactNode
}

function Layout({ children }: LayoutProps) {
  return (
    <div className="app-layout">
      <Sidebar />
      <main className="main-content">
        <div className="content-wrapper">
          {children}
        </div>
      </main>
    </div>
  )
}

export default Layout;