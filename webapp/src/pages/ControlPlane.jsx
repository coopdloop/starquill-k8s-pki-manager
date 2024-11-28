// pages/ControlPlane.jsx
import { Shield, Activity, CpuIcon, Database } from 'lucide-react';

export const ControlPlane = () => {
  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Control Plane</h1>
        <p className="text-slate-400">Control plane components and health status</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="bg-slate-800 rounded-lg p-6">
          <div className="flex items-center gap-4 mb-4">
            <Shield className="w-8 h-8 text-blue-400" />
            <div>
              <h3 className="text-lg font-medium text-white">API Server</h3>
              <p className="text-sm text-slate-400">v1.26.1</p>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-slate-400">Status</span>
              <span className="text-green-400">Healthy</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-slate-400">Uptime</span>
              <span className="text-white">15d 4h 23m</span>
            </div>
          </div>
        </div>

        <div className="bg-slate-800 rounded-lg p-6">
          <div className="flex items-center gap-4 mb-4">
            <Database className="w-8 h-8 text-purple-400" />
            <div>
              <h3 className="text-lg font-medium text-white">etcd</h3>
              <p className="text-sm text-slate-400">3.5.6</p>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-slate-400">Status</span>
              <span className="text-green-400">Healthy</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-slate-400">DB Size</span>
              <span className="text-white">256 MB</span>
            </div>
          </div>
        </div>

        <div className="bg-slate-800 rounded-lg p-6">
          <div className="flex items-center gap-4 mb-4">
            <CpuIcon className="w-8 h-8 text-yellow-400" />
            <div>
              <h3 className="text-lg font-medium text-white">Scheduler</h3>
              <p className="text-sm text-slate-400">v1.26.1</p>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-slate-400">Status</span>
              <span className="text-green-400">Healthy</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-slate-400">Active Workers</span>
              <span className="text-white">3/3</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
