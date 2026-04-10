import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Text, Card, CardHeader, Spinner } from '@fluentui/react-components';
import { Play, Package, User, Coffee } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

interface Stats {
  instances_count: number;
  accounts_count: number;
  java_path: string;
}

const Home = () => {
  const [stats, setStats] = useState<Stats | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const instances = await invoke<any[]>('get_instances');
        const accounts = await invoke<any[]>('get_accounts');
        const settings = await invoke<{ javaPath: string }>('get_settings');
        
        setStats({
          instances_count: instances.length,
          accounts_count: accounts.length,
          java_path: settings.javaPath
        });
      } catch (e) {
        console.error(e);
      }
    };
    fetchStats();
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
      <div>
        <h1 style={{ margin: 0, fontSize: '42px', fontWeight: 700, backgroundImage: 'linear-gradient(90deg, #60A5FA, #34D399)', WebkitBackgroundClip: 'text', color: 'transparent' }}>
          RustMC Launcher
        </h1>
        <Text size={500} style={{ color: 'rgba(255,255,255,0.7)', marginTop: '8px', display: 'block' }}>
          极速、轻量、原生的 Minecraft 启动器。
        </Text>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '20px' }}>
        <Card 
          style={{ backgroundColor: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)', cursor: 'pointer', transition: '0.2s' }}
          onClick={() => navigate('/instances')}
          onMouseEnter={(e) => e.currentTarget.style.transform = 'translateY(-2px)'}
          onMouseLeave={(e) => e.currentTarget.style.transform = 'none'}
        >
          <CardHeader
            header={<Text weight="semibold" size={500}>快速开始</Text>}
            description={<Text size={300} style={{ color: 'gray' }}>点击进入实例页面，创建或启动你的游戏环境！</Text>}
            action={<Play size={32} color="#60A5FA" />}
          />
        </Card>

        {stats ? (
          <>
            <Card style={{ backgroundColor: 'rgba(255,255,255,0.02)', border: '1px solid rgba(255,255,255,0.05)' }}>
              <CardHeader
                header={<Text weight="semibold" size={400}>系统状态</Text>}
                description={
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginTop: '12px' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Package size={16} color="#34D399" />
                      <Text style={{ color: '#ccc' }}>已创建 {stats.instances_count} 个实例</Text>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <User size={16} color="#60A5FA" />
                      <Text style={{ color: '#ccc' }}>已登录 {stats.accounts_count} 个账号</Text>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <Coffee size={16} color="#FBBF24" />
                      <Text style={{ color: '#ccc' }}>Java 路径: {stats.java_path === 'java' ? '系统默认' : stats.java_path.split('\\').pop()?.split('/').pop()}</Text>
                    </div>
                  </div>
                }
              />
            </Card>
          </>
        ) : (
          <Spinner size="small" />
        )}
      </div>
    </div>
  );
};

export default Home;
