import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  Button,
  Input,
  Card,
  CardHeader,
  Text,
  Spinner,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  ProgressBar,
  Dropdown,
  Option,
  Switch,
  Label,
  Tag
} from '@fluentui/react-components';
import { Play, Plus, Trash2, Box, Settings as SettingsIcon, Package, Zap } from 'lucide-react';

interface Instance {
  id: string;
  name: string;
  mc_version: string;
  loader: string; // Vanilla, Fabric, etc
}

interface Account {
  uuid: string;
  username: string;
}

const Instances = () => {
  const [instances, setInstances] = useState<Instance[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [newName, setNewName] = useState('新实例');
  const [newVersion, setNewVersion] = useState('1.20.1');
  const [newLoader, setNewLoader] = useState('Vanilla');
  const [usePreset, setUsePreset] = useState(false);

  const [launching, setLaunching] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [creating, setCreating] = useState(false);

  const fetchData = async () => {
    try {
      setLoading(true);
      const resInstances = await invoke<Instance[]>('get_instances');
      const resAccounts = await invoke<Account[]>('get_accounts');
      setInstances(resInstances);
      setAccounts(resAccounts);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();

    // Listen for Minecraft Logs
    const unlisten = listen<string>('mc-log', (event) => {
      setLogs((prev) => [...prev.slice(-40), event.payload]);
    });

    const unlistenExit = listen<number>('mc-exit', (event) => {
      setLaunching(false);
      setLogs((prev) => [...prev, `[系统] 游戏进程已退出，退出码: ${event.payload}`]);
    });

    return () => {
      unlisten.then(f => f());
      unlistenExit.then(f => f());
    };
  }, []);

  const handleCreate = async () => {
    if (!newName || !newVersion) {
      alert("请输入名称和游戏版本");
      return;
    }
    
    try {
      setCreating(true);
      await invoke('create_instance', { 
        name: newName, 
        mcVersion: newVersion, 
        loader: newLoader,
        usePerformancePreset: usePreset
      });
      setIsDialogOpen(false);
      
      // Reset form
      setNewName('新实例');
      setNewVersion('1.20.1');
      setNewLoader('Vanilla');
      setUsePreset(false);
      
      await fetchData();
    } catch (e) {
      console.error(e);
      alert("创建实例失败: " + e);
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm("确定要删除这个实例吗？这会清除所有存档和模组！")) {
      try {
        await invoke('delete_instance', { id });
        fetchData();
      } catch (e) {
        console.error(e);
      }
    }
  };

  const handleLaunch = async (instanceId: string) => {
    if (accounts.length === 0) {
      alert("请先在账号页面添加一个离线账号！");
      return;
    }
    
    try {
      setLaunching(true);
      setLogs(['[系统] 正在准备启动环境...']);
      
      // TODO: Make this dynamic via user settings
      const javaPath = 'java'; 
      
      await invoke('launch_minecraft', {
        instanceId: instanceId,
        username: accounts[0].username,
        javaPath: javaPath
      });
    } catch (e) {
      console.error(e);
      setLaunching(false);
      alert("启动失败: " + e);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', height: '100%', position: 'relative' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ margin: 0, fontSize: '28px', fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)' }}>实例</h1>
          <p style={{ margin: '4px 0 0 0', color: 'rgba(255, 255, 255, 0.5)', fontSize: '14px' }}>
            管理、创建和启动您的 Minecraft 游戏环境。
          </p>
        </div>
        
        <Dialog open={isDialogOpen} onOpenChange={(_e, data) => setIsDialogOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button appearance="primary" icon={<Plus size={16} />} size="large" style={{ backgroundColor: '#60CDFF', color: '#000' }}>新建实例</Button>
          </DialogTrigger>
          <DialogSurface style={{ backgroundColor: '#2B2B2B', border: '1px solid rgba(255,255,255,0.1)' }}>
            <DialogBody>
              <DialogTitle style={{ color: 'white' }}>创建新实例</DialogTitle>
              <DialogContent>
                <div style={{ padding: '16px 0', display: 'flex', flexDirection: 'column', gap: '16px' }}>
                  <div>
                    <Label htmlFor="inst-name" style={{ color: '#ccc', marginBottom: '4px', display: 'block' }}>实例名称</Label>
                    <Input 
                      id="inst-name"
                      placeholder="例如：我的生存世界" 
                      value={newName}
                      onChange={(_e, data) => setNewName(data.value)}
                      style={{ width: '100%' }}
                    />
                  </div>
                  <div>
                    <Label htmlFor="inst-version" style={{ color: '#ccc', marginBottom: '4px', display: 'block' }}>游戏版本</Label>
                    <Input 
                      id="inst-version"
                      placeholder="1.20.1" 
                      value={newVersion}
                      onChange={(_e, data) => setNewVersion(data.value)}
                      style={{ width: '100%' }}
                    />
                  </div>
                  <div>
                    <Label style={{ color: '#ccc', marginBottom: '4px', display: 'block' }}>Mod 加载器</Label>
                    <Dropdown 
                      value={newLoader}
                      onOptionSelect={(_e, data) => {
                        setNewLoader(data.optionValue as string);
                        if (data.optionValue !== 'Fabric') setUsePreset(false);
                      }}
                      style={{ width: '100%' }}
                    >
                      <Option value="Vanilla">Vanilla (原版纯净)</Option>
                      <Option value="Fabric">Fabric (现代模组生态)</Option>
                      <Option value="Forge" disabled>Forge (暂未实现)</Option>
                    </Dropdown>
                  </div>
                  
                  {newLoader === 'Fabric' && (
                    <div style={{ 
                      marginTop: '8px', 
                      backgroundColor: 'rgba(96, 205, 255, 0.05)', 
                      padding: '12px', 
                      borderRadius: '8px',
                      border: '1px solid rgba(96, 205, 255, 0.1)'
                    }}>
                      <Switch 
                        checked={usePreset} 
                        onChange={(_e, data) => setUsePreset(data.checked)} 
                        label={
                          <span style={{ display: 'flex', alignItems: 'center', gap: '6px', color: '#fff' }}>
                            <Zap size={16} color="#60CDFF" /> 自动预装性能优化套件
                          </span>
                        } 
                      />
                      <Text size={200} style={{ color: 'rgba(255,255,255,0.6)', display: 'block', marginLeft: '32px', marginTop: '4px' }}>
                        勾选后将自动为您安装 Sodium (钠), Iris (光影) 和 Lithium (锂) 等必备的帧数优化模组。
                      </Text>
                    </div>
                  )}
                </div>
              </DialogContent>
              <DialogActions>
                <DialogTrigger disableButtonEnhancement>
                  <Button appearance="secondary" disabled={creating}>取消</Button>
                </DialogTrigger>
                <Button appearance="primary" onClick={handleCreate} disabled={creating} style={{ backgroundColor: '#60CDFF', color: '#000' }}>
                  {creating ? <Spinner size="tiny" /> : '立即创建'}
                </Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>
      </div>

      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', flex: 1 }}>
          <Spinner size="huge" />
        </div>
      ) : instances.length === 0 ? (
        <div style={{ 
          display: 'flex', 
          flexDirection: 'column', 
          alignItems: 'center', 
          justifyContent: 'center',
          padding: '80px 0', 
          color: 'rgba(255,255,255,0.4)',
          backgroundColor: 'rgba(0,0,0,0.2)',
          borderRadius: '12px',
          border: '1px dashed rgba(255,255,255,0.05)'
        }}>
          <Box size={64} style={{ marginBottom: '16px', opacity: 0.3 }} />
          <Text size={500} weight="semibold" style={{ color: 'rgba(255,255,255,0.7)' }}>你还没有创建任何实例</Text>
          <Text size={300} style={{ marginTop: '8px' }}>点击右上角的“新建实例”开始你的冒险吧</Text>
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: '20px' }}>
          {instances.map((inst) => (
            <Card 
              key={inst.id} 
              style={{ 
                backgroundColor: 'rgba(255,255,255,0.04)', 
                border: '1px solid rgba(255,255,255,0.08)',
                borderRadius: '12px',
                backdropFilter: 'blur(10px)',
                padding: '20px',
                display: 'flex',
                flexDirection: 'column',
                gap: '16px',
                transition: 'transform 0.2s, background-color 0.2s',
                cursor: 'default'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.06)';
                e.currentTarget.style.transform = 'translateY(-2px)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.04)';
                e.currentTarget.style.transform = 'none';
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
                  <div style={{ 
                    width: '48px', height: '48px', 
                    backgroundColor: 'rgba(0,0,0,0.3)', 
                    borderRadius: '8px', 
                    display: 'flex', alignItems: 'center', justifyContent: 'center' 
                  }}>
                    <Package size={24} color="#60CDFF" />
                  </div>
                  <div>
                    <Text weight="semibold" size={500} style={{ color: 'white', display: 'block' }}>{inst.name}</Text>
                    <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
                      <Tag size="small" appearance="outline" style={{ color: '#aaa', borderColor: 'rgba(255,255,255,0.2)' }}>{inst.mc_version}</Tag>
                      <Tag size="small" appearance="outline" style={{ color: inst.loader === 'Fabric' ? '#ffdf89' : '#aaa', borderColor: 'rgba(255,255,255,0.2)' }}>
                        {inst.loader}
                      </Tag>
                    </div>
                  </div>
                </div>
              </div>

              <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 'auto', paddingTop: '8px' }}>
                <div style={{ display: 'flex', gap: '8px' }}>
                  <Button appearance="transparent" icon={<SettingsIcon size={18} color="rgba(255,255,255,0.7)" />} />
                  <Button appearance="transparent" icon={<Trash2 size={18} color="#ff6b6b" />} onClick={() => handleDelete(inst.id)} />
                </div>
                <Button 
                  appearance="primary" 
                  icon={<Play size={18} />} 
                  disabled={launching}
                  onClick={() => handleLaunch(inst.id)}
                  style={{ backgroundColor: '#60CDFF', color: '#000', padding: '6px 16px' }}
                >
                  启动
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* 实时日志终端面板 */}
      {launching && (
        <div style={{
          position: 'absolute',
          bottom: 0, left: 0, right: 0,
          backgroundColor: 'rgba(10, 10, 10, 0.95)',
          backdropFilter: 'blur(20px)',
          borderRadius: '12px',
          padding: '20px',
          fontFamily: 'Consolas, monospace',
          fontSize: '13px',
          height: '250px',
          overflowY: 'auto',
          border: '1px solid rgba(255,255,255,0.1)',
          boxShadow: '0 -10px 40px rgba(0,0,0,0.3)',
          zIndex: 1000,
          display: 'flex',
          flexDirection: 'column'
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '12px' }}>
            <Text weight="semibold" style={{ color: '#60CDFF' }}>启动控制台</Text>
            <ProgressBar thickness="large" style={{ width: '150px' }} />
          </div>
          <div style={{ color: '#4CAF50', marginBottom: '8px' }}>--- 正在校验并下载资源，首次启动时间可能较长，请耐心等待 ---</div>
          <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '2px' }}>
            {logs.map((log, i) => {
              let color = '#cccccc';
              if (log.includes('ERROR') || log.includes('Exception') || log.includes('failed')) color = '#ff6b6b';
              if (log.includes('WARN')) color = '#ffdf89';
              if (log.includes('INFO')) color = '#60CDFF';
              
              return (
                <div key={i} style={{ color, whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                  {log}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};

export default Instances;
