import { Outlet } from 'react-router-dom';
import Sidebar from '../components/Sidebar';
import TitleBar from '../components/TitleBar';

const Layout = () => {
  return (
    <div style={{
      display: 'flex',
      height: '100vh',
      width: '100vw',
      backgroundColor: 'transparent',
    }}>
      {/* 自定义拖拽标题栏 (Windows Mica 体验) */}
      <TitleBar />

      {/* 侧边栏 */}
      <Sidebar />

      {/* 主内容区域 */}
      <div style={{
        flex: 1,
        backgroundColor: 'rgba(32, 32, 32, 0.4)',
        backdropFilter: 'blur(20px)',
        overflowY: 'auto',
        position: 'relative',
        display: 'flex',
        flexDirection: 'column',
        // 为了美观，给左上角一个微妙的圆角效果和发光边框，更符合 Win11
        borderTopLeftRadius: '8px',
        borderLeft: '1px solid rgba(255, 255, 255, 0.05)',
        borderTop: '1px solid rgba(255, 255, 255, 0.05)',
      }}>
        {/* 内容容器 */}
        <div style={{ padding: '48px 32px 32px 32px', flex: 1, zIndex: 1 }}>
          <Outlet />
        </div>
      </div>
    </div>
  );
};

export default Layout;
