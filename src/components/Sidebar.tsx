import { Link, useLocation } from 'react-router-dom';
import { Home, List, ShoppingBag, User, Settings, Menu } from 'lucide-react';
import { useState } from 'react';

const Sidebar = () => {
  const location = useLocation();
  const [isCollapsed, setIsCollapsed] = useState(false);

  const navItems = [
    { path: '/', label: '首页', icon: Home },
    { path: '/instances', label: '实例', icon: List },
    { path: '/market', label: '市场', icon: ShoppingBag },
    { path: '/accounts', label: '账号', icon: User },
    { path: '/settings', label: '设置', icon: Settings },
  ];

  return (
    <div style={{
      width: isCollapsed ? '64px' : '240px',
      backgroundColor: 'rgba(32, 32, 32, 0.4)', // 更透明一点，让 Mica 更有感觉
      backdropFilter: 'blur(20px)',
      borderRight: '1px solid rgba(255, 255, 255, 0.05)',
      display: 'flex',
      flexDirection: 'column',
      padding: '48px 12px 16px 12px', // 顶部留出给自定义标题栏的空间
      boxSizing: 'border-box',
      transition: 'width 0.3s cubic-bezier(0.2, 0.8, 0.2, 1)', // 丝滑 Win11 动效
      position: 'relative',
      zIndex: 9000,
    }}>
      {/* 汉堡包菜单按钮 */}
      <div 
        onClick={() => setIsCollapsed(!isCollapsed)}
        style={{ 
          marginBottom: '24px', 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: isCollapsed ? 'center' : 'flex-start',
          padding: isCollapsed ? '0' : '0 12px',
          cursor: 'pointer',
          color: 'rgba(255, 255, 255, 0.8)',
        }}
      >
        <div style={{
          padding: '8px',
          borderRadius: '6px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          transition: 'background-color 0.2s',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.08)')}
        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
        >
          <Menu size={20} strokeWidth={1.5} />
        </div>
      </div>

      <nav style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.path;

          return (
            <Link
              key={item.path}
              to={item.path}
              title={isCollapsed ? item.label : undefined} // 折叠时鼠标悬停显示 tooltip
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: isCollapsed ? 'center' : 'flex-start',
                gap: isCollapsed ? '0' : '14px',
                padding: isCollapsed ? '12px 0' : '10px 14px',
                borderRadius: '6px',
                textDecoration: 'none',
                color: isActive ? '#ffffff' : 'rgba(255, 255, 255, 0.7)',
                backgroundColor: isActive ? 'rgba(255, 255, 255, 0.06)' : 'transparent',
                transition: 'all 0.2s ease',
                position: 'relative',
                overflow: 'hidden',
              }}
              onMouseEnter={(e) => {
                if (!isActive) e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.03)';
              }}
              onMouseLeave={(e) => {
                if (!isActive) e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              {/* 活动指示条 (左侧蓝色小竖线) */}
              {isActive && (
                <div style={{
                  position: 'absolute',
                  left: '4px',
                  top: '50%',
                  transform: 'translateY(-50%)',
                  height: '16px',
                  width: '3px',
                  backgroundColor: '#60CDFF', // Win11 主题蓝
                  borderRadius: '4px',
                }} />
              )}
              <Icon size={18} strokeWidth={isActive ? 2 : 1.5} />
              
              {!isCollapsed && (
                <span style={{ 
                  fontSize: '14px', 
                  fontWeight: isActive ? 500 : 400,
                  whiteSpace: 'nowrap',
                  opacity: 1,
                  transition: 'opacity 0.2s',
                }}>
                  {item.label}
                </span>
              )}
            </Link>
          );
        })}
      </nav>
    </div>
  );
};

export default Sidebar;
