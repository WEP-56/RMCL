import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, ArrowLeft } from 'lucide-react';
import { useNavigate, useLocation } from 'react-router-dom';

const TitleBar = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const appWindow = getCurrentWindow();

  useEffect(() => {
    // 监听窗口最大化状态
    const unlisten = appWindow.onResized(async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const handleMinimize = () => appWindow.minimize();
  const handleToggleMaximize = async () => {
    const maximized = await appWindow.isMaximized();
    if (maximized) {
      await appWindow.unmaximize();
    } else {
      await appWindow.maximize();
    }
  };
  const handleClose = () => appWindow.close();

  // 除了首页之外，显示返回按钮
  const showBackButton = location.pathname !== '/';

  return (
    <div
      data-tauri-drag-region
      style={{
        height: '32px',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        zIndex: 9999,
        // 背景完全透明，让底层 Mica 材质透出
        backgroundColor: 'transparent',
      }}
    >
      {/* 左侧：标题与返回按钮 */}
      <div
        data-tauri-drag-region
        style={{
          display: 'flex',
          alignItems: 'center',
          paddingLeft: '12px',
          height: '100%',
          flex: 1,
        }}
      >
        {showBackButton && (
          <div
            onClick={() => navigate(-1)}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: '32px',
              height: '32px',
              cursor: 'pointer',
              color: 'rgba(255, 255, 255, 0.8)',
              borderRadius: '4px',
              transition: 'background-color 0.2s',
              marginRight: '8px',
              zIndex: 10000,
            }}
            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.1)')}
            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
          >
            <ArrowLeft size={16} />
          </div>
        )}
        <span
          data-tauri-drag-region
          style={{
            fontSize: '12px',
            color: 'rgba(255, 255, 255, 0.7)',
            pointerEvents: 'none',
            fontFamily: 'Segoe UI, sans-serif',
          }}
        >
          RustMC Launcher
        </span>
      </div>

      {/* 右侧：窗口控制按钮 */}
      <div style={{ display: 'flex', height: '100%' }}>
        <div
          onClick={handleMinimize}
          style={{
            width: '46px',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            color: 'rgba(255, 255, 255, 0.8)',
            transition: 'background-color 0.2s',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.1)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
        >
          <Minus size={16} strokeWidth={1.5} />
        </div>
        <div
          onClick={handleToggleMaximize}
          style={{
            width: '46px',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            color: 'rgba(255, 255, 255, 0.8)',
            transition: 'background-color 0.2s',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'rgba(255, 255, 255, 0.1)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'transparent')}
        >
          {isMaximized ? (
            <div style={{ position: 'relative', width: '12px', height: '12px' }}>
              <Square size={10} strokeWidth={1.5} style={{ position: 'absolute', top: 0, right: 0 }} />
              <Square size={10} strokeWidth={1.5} style={{ position: 'absolute', bottom: 0, left: 0, backgroundColor: '#202020' }} />
            </div>
          ) : (
            <Square size={14} strokeWidth={1.5} />
          )}
        </div>
        <div
          onClick={handleClose}
          style={{
            width: '46px',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            color: 'rgba(255, 255, 255, 0.8)',
            transition: 'background-color 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = '#e81123';
            e.currentTarget.style.color = '#fff';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'rgba(255, 255, 255, 0.8)';
          }}
        >
          <X size={16} strokeWidth={1.5} />
        </div>
      </div>
    </div>
  );
};

export default TitleBar;
