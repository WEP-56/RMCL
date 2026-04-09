import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { FluentProvider, webDarkTheme } from '@fluentui/react-components';
import { BrowserRouter } from 'react-router-dom';
import { LauncherProvider } from './contexts/LauncherContext';
import App from './App';
import './index.css';

const root = createRoot(document.getElementById('root') as HTMLElement);

root.render(
  <StrictMode>
    <FluentProvider theme={webDarkTheme} style={{ height: '100vh', backgroundColor: 'transparent' }}>
      <BrowserRouter>
        <LauncherProvider>
          <App />
        </LauncherProvider>
      </BrowserRouter>
    </FluentProvider>
  </StrictMode>
);
