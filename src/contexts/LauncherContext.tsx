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
  logLines: string[];
  launchInstance: (instanceId: string, accountUuid: string, javaPath: string) => Promise<void>;
  resetLaunch: () => void;
}

const LauncherContext = createContext<LauncherContextType | undefined>(undefined);

export const LauncherProvider = ({ children }: { children: ReactNode }) => {
  const [launchStatus, setLaunchStatus] = useState<LaunchStatus>('idle');
  const [activeInstanceId, setActiveInstanceId] = useState<string | null>(null);
  const [activeTempId, setActiveTempId] = useState<string | null>(null);
  const [progressData, setProgressData] = useState<ProgressPayload | null>(null);
  const [logLines, setLogLines] = useState<string[]>([]);

  useEffect(() => {
    const unlistenProgress = listen<ProgressPayload>('mc-progress', (event) => {
      setProgressData(event.payload);
      
      // If we don't have an active instance ID but we got a progress event,
      // it might be a modpack download. Store the ID temporarily to show progress.
      if (!activeInstanceId && !activeTempId) {
        setActiveTempId(event.payload.instance_id);
      }
    });

    const unlistenLog = listen<string>('mc-log', (event) => {
      setLogLines((current) => {
        const next = [...current, event.payload];
        return next.slice(-400);
      });
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
        setActiveTempId(null);
        setProgressData(null);
      }, 3000);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenLog.then((f) => f());
      unlistenExit.then((f) => f());
    };
  }, [activeInstanceId, activeTempId]);

  const launchInstance = async (instanceId: string, accountUuid: string, javaPath: string) => {
    if (launchStatus !== 'idle' && launchStatus !== 'error') return;
    
    try {
      setActiveInstanceId(instanceId);
      setLaunchStatus('preparing');
      setLogLines([]);
      setProgressData({
        instance_id: instanceId,
        task: '准备启动环境...',
        progress: -1.0,
        text: '正在初始化'
      });
      
      await invoke('launch_minecraft', {
        instanceId,
        accountUuid,
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
    setActiveTempId(null);
    setProgressData(null);
    setLogLines([]);
  };

  return (
    <LauncherContext.Provider value={{ launchStatus, activeInstanceId: activeInstanceId || activeTempId, progressData, logLines, launchInstance, resetLaunch }}>
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
