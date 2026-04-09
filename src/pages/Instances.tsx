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
  Switch
} from '@fluentui/react-components';
import { Play, Plus, Trash2, Box } from 'lucide-react';

interface Instance {
  id: string;
  name: string;
  mc_version: string;
  loader: string;
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
  const [newName, setNewName] = useState('New Instance');
  const [newVersion, setNewVersion] = useState('1.20.1');
  const [newLoader, setNewLoader] = useState('Vanilla');
  const [usePreset, setUsePreset] = useState(false);

  const [launching, setLaunching] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);

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
      setLogs((prev) => [...prev.slice(-20), event.payload]);
    });

    const unlistenExit = listen<number>('mc-exit', (event) => {
      setLaunching(false);
      setLogs((prev) => [...prev, `[INFO] Game exited with code: ${event.payload}`]);
    });

    return () => {
      unlisten.then(f => f());
      unlistenExit.then(f => f());
    };
  }, []);

  const handleCreate = async () => {
    try {
      await invoke('create_instance', { 
        name: newName, 
        mcVersion: newVersion, 
        loader: newLoader,
        usePerformancePreset: usePreset
      });
      setIsDialogOpen(false);
      fetchData();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke('delete_instance', { id });
      fetchData();
    } catch (e) {
      console.error(e);
    }
  };

  const handleLaunch = async (version: string) => {
    if (accounts.length === 0) {
      alert("请先在账号页面添加一个离线账号！");
      return;
    }
    
    try {
      setLaunching(true);
      setLogs(['[INFO] Starting launch sequence...']);
      
      // Mock Java path for Linux testing. In prod, auto detect or ask user
      const javaPath = 'java'; 
      
      await invoke('launch_minecraft', {
        versionId: version,
        username: accounts[0].username, // Use first account for demo
        javaPath: javaPath
      });
    } catch (e) {
      console.error(e);
      setLaunching(false);
      alert("Launch failed: " + e);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', height: '100%' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h1 style={{ margin: 0, fontSize: '28px', fontWeight: 600 }}>游戏实例</h1>
        
        <Dialog open={isDialogOpen} onOpenChange={(_e, data) => setIsDialogOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button appearance="primary" icon={<Plus size={16} />}>新建实例</Button>
          </DialogTrigger>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>创建新实例</DialogTitle>
              <DialogContent>
                <div style={{ padding: '16px 0', display: 'flex', flexDirection: 'column', gap: '12px' }}>
                  <Input 
                    placeholder="实例名称" 
                    value={newName}
                    onChange={(_e, data) => setNewName(data.value)}
                  />
                  <Input 
                    placeholder="Minecraft 版本 (如: 1.20.1)" 
                    value={newVersion}
                    onChange={(_e, data) => setNewVersion(data.value)}
                  />
                  <div>
                    <Text weight="semibold" size={200} style={{ display: 'block', marginBottom: '4px' }}>Mod 加载器</Text>
                    <Dropdown 
                      value={newLoader}
                      onOptionSelect={(_e, data) => setNewLoader(data.optionValue as string)}
                      style={{ width: '100%' }}
                    >
                      <Option value="Vanilla">Vanilla (原版)</Option>
                      <Option value="Fabric">Fabric</Option>
                      <Option value="Forge" disabled>Forge (敬请期待)</Option>
                    </Dropdown>
                  </div>
                  
                  {newLoader === 'Fabric' && (
                    <div style={{ marginTop: '8px' }}>
                      <Switch 
                        checked={usePreset} 
                        onChange={(_e, data) => setUsePreset(data.checked)} 
                        label="预装性能优化模组 (Sodium, Iris, Lithium)" 
                      />
                      <Text size={100} style={{ color: 'gray', display: 'block', marginLeft: '32px' }}>勾选后将自动为您安装这些优化帧数的必备模组</Text>
                    </div>
                  )}
                </div>
              </DialogContent>
              <DialogActions>
                <DialogTrigger disableButtonEnhancement>
                  <Button appearance="secondary">取消</Button>
                </DialogTrigger>
                <Button appearance="primary" onClick={handleCreate}>创建</Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>
      </div>

      {loading ? (
        <Spinner size="large" />
      ) : instances.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '60px 0', color: 'rgba(255,255,255,0.5)' }}>
          <Box size={48} style={{ marginBottom: '16px', opacity: 0.5 }} />
          <Text size={400} style={{ display: 'block' }}>暂无实例</Text>
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '16px' }}>
          {instances.map((inst) => (
            <Card key={inst.id} style={{ backgroundColor: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)' }}>
              <CardHeader
                header={<Text weight="semibold" size={400}>{inst.name}</Text>}
                description={<Text size={200} style={{ color: 'gray' }}>Minecraft {inst.mc_version} • {inst.loader}</Text>}
                action={<Button appearance="transparent" icon={<Trash2 size={16} color="#ff6b6b" />} onClick={() => handleDelete(inst.id)} />}
              />
              <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '12px' }}>
                <Button 
                  appearance="primary" 
                  icon={<Play size={16} />} 
                  disabled={launching}
                  onClick={() => handleLaunch(inst.mc_version)}
                >
                  启动游戏
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* 实时日志终端面板 */}
      {launching && (
        <div style={{
          marginTop: 'auto',
          backgroundColor: '#0c0c0c',
          borderRadius: '8px',
          padding: '16px',
          fontFamily: 'monospace',
          fontSize: '12px',
          height: '200px',
          overflowY: 'auto',
          border: '1px solid rgba(255,255,255,0.1)'
        }}>
          <ProgressBar thickness="large" style={{ marginBottom: '12px' }} />
          <div style={{ color: '#4CAF50', marginBottom: '8px' }}>--- 正在校验并下载资源，首次启动时间较长，请耐心等待 ---</div>
          {logs.map((log, i) => (
            <div key={i} style={{ color: log.includes('ERROR') ? '#ff6b6b' : '#cccccc', whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
              {log}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default Instances;
