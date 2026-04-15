import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import {
  Button,
  Input,
  Card,
  CardHeader,
  Text,
  Spinner,
  Toast,
  ToastTitle,
  Toaster,
  useToastController,
  useId,
  Slider,
  Label,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Dropdown,
  Option
} from '@fluentui/react-components';
import { Folder, Coffee, Save, RotateCcw, Cpu, Search, Globe } from 'lucide-react';

interface AppSettings {
  javaPath: string;
  maxMemory: number;
  gameDirectory: string | null;
  downloadSource?: string;
}

interface JavaInstallation {
  name: string;
  path: string;
  version: string;
}

const Settings = () => {
  const [settings, setSettings] = useState<AppSettings>({
    javaPath: 'java',
    maxMemory: 2048,
    gameDirectory: null,
    downloadSource: 'Default',
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [javaDialog, setJavaDialog] = useState(false);
  const [javas, setJavas] = useState<JavaInstallation[]>([]);
  const [javasLoading, setJavasLoading] = useState(false);
  
  const toasterId = useId('toaster');
  const { dispatchToast } = useToastController(toasterId);

  const fetchSettings = async () => {
    try {
      setLoading(true);
      const res = await invoke<AppSettings>('get_settings');
      setSettings(res);
    } catch (e) {
      console.error('Failed to load settings', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const handleSave = async () => {
    try {
      setSaving(true);
      await invoke('save_settings', { settings });
      dispatchToast(
        <Toast>
          <ToastTitle>设置已保存</ToastTitle>
        </Toast>,
        { intent: 'success' }
      );
    } catch (e) {
      console.error(e);
      dispatchToast(
        <Toast>
          <ToastTitle>保存失败: {String(e)}</ToastTitle>
        </Toast>,
        { intent: 'error' }
      );
    } finally {
      setSaving(false);
    }
  };

  const handleSelectGameDir = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择共享运行时目录'
      });
      if (selected && typeof selected === 'string') {
        setSettings({ ...settings, gameDirectory: selected });
      }
    } catch (e) {
      console.error('Failed to open dialog', e);
    }
  };

  const handleSelectJavaPath = async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: '选择 Java 可执行文件',
        filters: [{
          name: 'Executable',
          extensions: ['exe', '']
        }]
      });
      if (selected && typeof selected === 'string') {
        setSettings({ ...settings, javaPath: selected });
      }
    } catch (e) {
      console.error('Failed to open dialog', e);
    }
  };

  const scanJavas = async () => {
    try {
      setJavasLoading(true);
      const res = await invoke<JavaInstallation[]>('scan_java_installations');
      setJavas(res);
    } catch (e) {
      console.error(e);
    } finally {
      setJavasLoading(false);
    }
  };

  const resetGameDir = () => {
    setSettings({ ...settings, gameDirectory: null });
  };

  const resetJavaPath = () => {
    setSettings({ ...settings, javaPath: 'java' });
  };

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', flex: 1, height: '100%' }}>
        <Spinner size="huge" />
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', height: '100%', maxWidth: '800px', margin: '0 auto' }}>
      <Toaster toasterId={toasterId} />
      
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ margin: 0, fontSize: '28px', fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)' }}>全局设置</h1>
          <p style={{ margin: '4px 0 0 0', color: 'rgba(255, 255, 255, 0.5)', fontSize: '14px' }}>
            配置游戏路径、Java 运行环境以及性能参数。
          </p>
        </div>
        <Button 
          appearance="primary" 
          icon={<Save size={16} />} 
          onClick={handleSave} 
          disabled={saving}
          style={{ backgroundColor: '#60CDFF', color: '#000' }}
        >
          {saving ? '保存中...' : '保存更改'}
        </Button>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
        
        {/* Game Directory Section */}
        <Card style={{ 
          backgroundColor: 'rgba(255,255,255,0.03)', 
          border: '1px solid rgba(255,255,255,0.08)',
          borderRadius: '12px',
        }}>
          <CardHeader
            header={<Text weight="semibold" size={400} style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '8px' }}><Folder size={18} color="#60CDFF" /> 共享运行时目录</Text>}
            description={<Text size={200} style={{ color: 'gray' }}>自定义共享缓存目录。`assets`、`libraries`、`versions`、`java` 等运行时资源会存储在这里，实例自己的数据仍保存在独立实例目录中。</Text>}
          />
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginTop: '12px' }}>
            <Input 
              value={settings.gameDirectory || ''} 
              placeholder="默认路径 (AppData/Roaming/RustMCLauncher/runtime)" 
              readOnly
              style={{ flex: 1 }}
            />
            <Button appearance="secondary" onClick={handleSelectGameDir}>浏览...</Button>
            <Button appearance="transparent" icon={<RotateCcw size={16} />} onClick={resetGameDir} title="恢复默认" />
          </div>
        </Card>

        {/* Java Path Section */}
        <Card style={{ 
          backgroundColor: 'rgba(255,255,255,0.03)', 
          border: '1px solid rgba(255,255,255,0.08)',
          borderRadius: '12px',
        }}>
          <CardHeader
            header={<Text weight="semibold" size={400} style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '8px' }}><Coffee size={18} color="#ffdf89" /> Java 路径</Text>}
            description={<Text size={200} style={{ color: 'gray' }}>指定启动 Minecraft 所使用的 Java 可执行文件 (javaw.exe) 路径。</Text>}
          />
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginTop: '12px' }}>
            <Input 
              value={settings.javaPath} 
              onChange={(_e, data) => setSettings({ ...settings, javaPath: data.value })}
              placeholder="例如：C:\Program Files\Java\jdk-17\bin\javaw.exe 或 java" 
              style={{ flex: 1 }}
            />
            
            <Dialog open={javaDialog} onOpenChange={(_e, data) => { setJavaDialog(data.open); if(data.open) scanJavas(); }}>
              <DialogTrigger disableButtonEnhancement>
                <Button appearance="secondary" icon={<Search size={16} />}>自动探测</Button>
              </DialogTrigger>
              <DialogSurface>
                <DialogBody>
                  <DialogTitle>选择 Java 版本</DialogTitle>
                  <DialogContent>
                    {javasLoading ? (
                      <div style={{ padding: '24px', textAlign: 'center' }}>
                        <Spinner label="正在扫描本地 Java 安装..." />
                      </div>
                    ) : (
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginTop: '12px', maxHeight: '300px', overflowY: 'auto' }}>
                        {javas.length === 0 ? (
                          <Text>未发现安装的 Java 环境。</Text>
                        ) : (
                          javas.map((j, i) => (
                            <Card key={i} style={{ padding: '8px', cursor: 'pointer', backgroundColor: 'rgba(255,255,255,0.05)' }} onClick={() => {
                              setSettings({ ...settings, javaPath: j.path });
                              setJavaDialog(false);
                            }}>
                              <Text weight="semibold">{j.name}</Text>
                              <Text size={200} style={{ color: '#60CDFF' }}>版本: {j.version}</Text>
                              <Text size={100} style={{ color: 'gray', wordBreak: 'break-all' }}>{j.path}</Text>
                            </Card>
                          ))
                        )}
                      </div>
                    )}
                  </DialogContent>
                  <DialogActions>
                    <DialogTrigger disableButtonEnhancement>
                      <Button appearance="secondary">关闭</Button>
                    </DialogTrigger>
                  </DialogActions>
                </DialogBody>
              </DialogSurface>
            </Dialog>

            <Button appearance="secondary" onClick={handleSelectJavaPath}>浏览...</Button>
            <Button appearance="transparent" icon={<RotateCcw size={16} />} onClick={resetJavaPath} title="恢复默认 (自动探测)" />
          </div>
          <Text size={100} style={{ color: 'rgba(255,255,255,0.4)', marginTop: '8px', display: 'block' }}>
            提示：保持为 "java" 时，启动器将尝试自动从系统环境变量或常见目录中探测合适的 Java 版本。
          </Text>
        </Card>

        {/* Memory Allocation Section */}
        <Card style={{ 
          backgroundColor: 'rgba(255,255,255,0.03)', 
          border: '1px solid rgba(255,255,255,0.08)',
          borderRadius: '12px',
        }}>
          <CardHeader
            header={<Text weight="semibold" size={400} style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '8px' }}><Cpu size={18} color="#ffdf89" /> 内存分配</Text>}
            description={<Text size={200} style={{ color: 'gray' }}>设置分配给 Minecraft 的最大运行内存 (建议 2048MB - 4096MB)。</Text>}
          />
          <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginTop: '12px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <Label>最大内存: {settings.maxMemory} MB</Label>
              <Text size={200} style={{ color: 'rgba(255,255,255,0.5)' }}>{(settings.maxMemory / 1024).toFixed(1)} GB</Text>
            </div>
            <Slider 
              min={1024} 
              max={16384} 
              step={512} 
              value={settings.maxMemory} 
              onChange={(_e, data) => setSettings({ ...settings, maxMemory: data.value })}
            />
          </div>
        </Card>

        {/* Download Source Section */}
        <Card style={{ 
          backgroundColor: 'rgba(255,255,255,0.03)', 
          border: '1px solid rgba(255,255,255,0.08)',
          borderRadius: '12px',
        }}>
          <CardHeader
            header={<Text weight="semibold" size={400} style={{ color: 'white', display: 'flex', alignItems: 'center', gap: '8px' }}><Globe size={18} color="#34D399" /> 下载源 (镜像)</Text>}
            description={<Text size={200} style={{ color: 'gray' }}>为国内网络环境不佳的玩家提供更快的下载速度。更换后会在下一次下载生效。</Text>}
          />
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginTop: '12px' }}>
            <Dropdown 
              value={settings.downloadSource || 'Default'}
              onOptionSelect={(_e, data) => setSettings({ ...settings, downloadSource: data.optionValue as string })}
              style={{ minWidth: '200px' }}
            >
              <Option value="Default">官方默认源 (Mojang/Fabric)</Option>
              <Option value="BMCLAPI">BMCLAPI (国内高速镜像)</Option>
            </Dropdown>
          </div>
        </Card>

      </div>
    </div>
  );
};

export default Settings;
