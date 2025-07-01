import { NavLink } from 'react-router-dom'

const navigation = [
  { name: '首页', href: '/', icon: '🏠' },
  { name: '笔记', href: '/notes', icon: '📝' },
  { name: '设置', href: '/settings', icon: '⚙️' },
]

export default function Sidebar() {
  return (
    <div style={{ 
      width: '256px', 
      backgroundColor: '#f8f9fa', 
      borderRight: '1px solid #e9ecef',
      padding: '1rem'
    }}>
      <div style={{ marginBottom: '2rem' }}>
        <h1 style={{ fontSize: '1.25rem', fontWeight: 'bold', margin: 0 }}>Zeno</h1>
        <p style={{ fontSize: '0.875rem', color: '#6c757d', margin: '0.25rem 0 0 0' }}>知识管理平台</p>
      </div>
      
      <nav style={{ display: 'flex', flexDirection: 'column', gap: '0.25rem' }}>
        {navigation.map((item) => (
          <NavLink
            key={item.name}
            to={item.href}
            style={({ isActive }) => ({
              display: 'flex',
              alignItems: 'center',
              padding: '0.5rem 0.75rem',
              fontSize: '0.875rem',
              fontWeight: '500',
              borderRadius: '0.375rem',
              textDecoration: 'none',
              backgroundColor: isActive ? '#007bff' : 'transparent',
              color: isActive ? 'white' : '#6c757d',
              transition: 'all 0.2s'
            })}
          >
            <span style={{ marginRight: '0.75rem', fontSize: '1rem' }}>{item.icon}</span>
            {item.name}
          </NavLink>
        ))}
      </nav>
    </div>
  )
}