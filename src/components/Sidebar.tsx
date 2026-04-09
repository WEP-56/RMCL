import { Link, useLocation } from 'react-router-dom';
import { Home, List, ShoppingBag, User, Settings } from 'lucide-react';

const Sidebar = () => {
  const location = useLocation();

  const navItems = [
    { path: '/', label: '首页', icon: Home },
    { path: '/instances', label: '实例', icon: List },
    { path: '/market', label: '市场', icon: ShoppingBag },
    { path: '/accounts', label: '账号', icon: User },
    { path: '/settings', label: '设置', icon: Settings },
  ];

  return (
    <div style={{
      width: '240px',
      backgroundColor: 'rgba(32, 32, 32, 0.6)',
      backdropFilter: 'blur(20px)',
      borderRight: '1px solid rgba(255, 255, 255, 0.08)',
      display: 'flex',
      flexDirection: 'column',
      padding: '16px 12px',
      boxSizing: 'border-box'
    }}>
      <div style={{ marginBottom: '24px', paddingLeft: '12px' }}>
        <h2 style={{ fontSize: '18px', fontWeight: 600, margin: 0 }}>RustMC</h2>
      </div>

      <nav style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.path;

          return (
            <Link
              key={item.path}
              to={item.path}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                padding: '10px 12px',
                borderRadius: '6px',
                textDecoration: 'none',
                color: isActive ? '#ffffff' : 'rgba(255, 255, 255, 0.7)',
                backgroundColor: isActive ? 'rgba(255, 255, 255, 0.06)' : 'transparent',
                transition: 'all 0.2s ease',
              }}
            >
              <Icon size={18} />
              <span style={{ fontSize: '14px', fontWeight: isActive ? 500 : 400 }}>{item.label}</span>
            </Link>
          );
        })}
      </nav>
    </div>
  );
};

export default Sidebar;
