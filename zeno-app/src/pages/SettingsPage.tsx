export default function SettingsPage() {
  return (
    <div style={{ padding: '2rem' }}>
      <div style={{ maxWidth: '64rem', margin: '0 auto' }}>
        <h1 style={{ 
          fontSize: '1.875rem', 
          fontWeight: 'bold', 
          marginBottom: '1.5rem' 
        }}>
          设置
        </h1>
        
        <div style={{ 
          backgroundColor: '#f8f9fa', 
          border: '1px solid #e9ecef', 
          borderRadius: '0.5rem', 
          padding: '1.5rem' 
        }}>
          <div style={{ textAlign: 'center', padding: '3rem 0' }}>
            <div style={{ fontSize: '3.75rem', marginBottom: '1rem' }}>⚙️</div>
            <h2 style={{ 
              fontSize: '1.25rem', 
              fontWeight: '600', 
              marginBottom: '0.5rem' 
            }}>
              即将推出
            </h2>
            <p style={{ color: '#6c757d' }}>
              设置功能正在开发中，敬请期待！
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}