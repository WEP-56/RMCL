import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type LaunchStatus = 'idle' | 'preparing' | 'running' | 'error';

export interface ProgressPayload {
  instance_id: string;
  task: string;
  progress: number;
  text: string;
}

interface LauncherContextType {
  launchStatus: LaunchStatus;
  activeInstanceId: string | null;
  progressData: ProgressPayload | null;
  launchInstance: (instanceId: string, username: string, javaPath: string) => Promise<void>;
  resetLaunch: () => void;
}

const LauncherContext = createContext<LauncherContextType | undefined>(undefined);

export const LauncherProvider = ({ children }: { children: ReactNode }) => {
  const [launchStatus, setLaunchStatus] = useState<LaunchStatus>('idle');
  const [activeInstanceId, setActiveInstanceId] = useState<string | null>(null);
  const [progressData, setProgressData] = useState<ProgressPayload | null>(null);

  useEffect(() => {
    const unlistenProgress = listen<ProgressPayload>('mc-progress', (event) => {
      setProgressData(event.payload);
    });

    const unlistenExit = listen<number>('mc-exit', (event) => {
      setLaunchStatus('idle');
      setProgressData({
        instance_id: activeInstanceId || '',
        task: '已停止运行',
        progress: 1.0,
        text: `退出码: ${event.payload}`,
      });
      // 稍微延迟后清空状态，让用户能看到结束状态
      setTimeout(() => {
        setActiveInstanceId(null);
        setProgressData(null);
      }, 3000);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenExit.then((f) => f());
    };
  }, [activeInstanceId]);

  const launchInstance = async (instanceId: string, username: string, javaPath: string) => {
    if (launchStatus !== 'idle' && launchStatus !== 'error') return;
    
    try {
      setActiveInstanceId(instanceId);
      setLaunchStatus('preparing');
      setProgressData({
        instance_id: instanceId,
        task: '准备启动环境...',
        progress: -1.0,
        text: '正在初始化'
      });
      
      await invoke('launch_minecraft', {
        instanceId,
        username,
        javaPath
      });
      
      setLaunchStatus('running');
      setProgressData({
        instance_id: instanceId,
        task: '游戏运行中',
        progress: 1.0,
        text: 'Minecraft is running'
      });
    } catch (e) {
      console.error(e);
      setLaunchStatus('error');
      setProgressData({
        instance_id: instanceId,
        task: '启动失败',
        progress: 1.0,
        text: String(e)
      });
    }
  };

  const resetLaunch = () => {
    setLaunchStatus('idle');
    setActiveInstanceId(null);
    setProgressData(null);
  };

  return (
    <LauncherContext.Provider value={{ launchStatus, activeInstanceId, progressData, launchInstance, resetLaunch }}>
      {children}
    </LauncherContext.Provider>
  );
};

export const useLauncher = () => {
  const context = useContext(LauncherContext);
  if (context === undefined) {
    throw new Error('useLauncher must be used within a LauncherProvider');
  }
  return context;
};