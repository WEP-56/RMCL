import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type LaunchStatus = 'idle' | 'preparing' | 'running' | 'error';

interface LauncherContextType {
  launchStatus: LaunchStatus;
  logs: string[];
  launchInstance: (instanceId: string, username: string, javaPath: string) => Promise<void>;
  clearLogs: () => void;
}

const LauncherContext = createContext<LauncherContextType | undefined>(undefined);

export const LauncherProvider = ({ children }: { children: ReactNode }) => {
  const [launchStatus, setLaunchStatus] = useState<LaunchStatus>('idle');
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    const unlistenLog = listen<string>('mc-log', (event) => {
      setLogs((prev) => [...prev.slice(-100), event.payload]); // Keep last 100 logs
    });

    const unlistenExit = listen<number>('mc-exit', (event) => {
      setLaunchStatus('idle');
      setLogs((prev) => [...prev, `[系统] 游戏进程已退出，退出码: ${event.payload}`]);
    });

    return () => {
      unlistenLog.then((f) => f());
      unlistenExit.then((f) => f());
    };
  }, []);

  const launchInstance = async (instanceId: string, username: string, javaPath: string) => {
    if (launchStatus !== 'idle' && launchStatus !== 'error') return;
    
    try {
      setLaunchStatus('preparing');
      setLogs(['[系统] 正在准备启动环境...']);
      
      await invoke('launch_minecraft', {
        instanceId,
        username,
        javaPath
      });
      
      setLaunchStatus('running');
    } catch (e) {
      console.error(e);
      setLaunchStatus('error');
      setLogs((prev) => [...prev, `[系统错误] 启动失败: ${e}`]);
    }
  };

  const clearLogs = () => setLogs([]);

  return (
    <LauncherContext.Provider value={{ launchStatus, logs, launchInstance, clearLogs }}>
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