import { Outlet } from 'react-router-dom';
import Sidebar from '../components/Sidebar';

const Layout = () => {
  return (
    <div style={{
      display: 'flex',
      height: '100vh',
      width: '100vw',
      backgroundColor: 'transparent',
    }}>
      {/* 侧边栏 */}
      <Sidebar />

      {/* 主内容区域 */}
      <div style={{
        flex: 1,
        backgroundColor: 'rgba(28, 28, 28, 0.8)',
        backdropFilter: 'blur(30px)',
        overflowY: 'auto',
        position: 'relative',
        display: 'flex',
        flexDirection: 'column',
      }}>
        {/* 自定义拖拽标题栏 (Windows Mica 体验) */}
        <div 
          data-tauri-drag-region 
          style={{
            height: '32px',
            width: '100%',
            position: 'absolute',
            top: 0,
            left: 0,
            zIndex: 100,
            cursor: 'default'
          }}
        />
        
        {/* 内容容器 */}
        <div style={{ padding: '40px 32px 32px 32px', flex: 1, zIndex: 1 }}>
          <Outlet />
        </div>
      </div>
    </div>
  );
};

export default Layout;
