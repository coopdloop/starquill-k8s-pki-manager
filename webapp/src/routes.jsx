import { createBrowserRouter } from 'react-router-dom';
import { Dashboard } from './pages/Dashboard';
import { ControlPlane } from './pages/ControlPlane';
import { WorkerNodes } from './pages/WorkerNodes';
import { Certificates } from './pages/Certificates';
import { Settings } from './pages/Settings';
import { Layout } from './Layout';

export const router = createBrowserRouter([
  {
    path: '/',
    element: <Layout />,
    children: [
      { path: '/', element: <Dashboard /> },
      { path: '/control-plane', element: <ControlPlane /> },
      { path: '/workers', element: <WorkerNodes /> },
      { path: '/certificates', element: <Certificates /> },
      { path: '/settings', element: <Settings /> },
    ],
  },
]);
