// pages/Settings.jsx
import { Settings as SettingsIcon, Save } from 'lucide-react';

export const Settings = () => {
  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Settings</h1>
        <p className="text-slate-400">Configure cluster settings and preferences</p>
      </div>

      <div className="bg-slate-800 rounded-lg p-6">
        <form className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-white mb-2">
              Cluster Name
            </label>
            <input
              type="text"
              className="w-full bg-slate-700 rounded-lg px-4 py-2 text-white"
              placeholder="production-cluster"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-white mb-2">
              API Server URL
            </label>
            <input
              type="text"
              className="w-full bg-slate-700 rounded-lg px-4 py-2 text-white"
              placeholder="https://api.cluster.local:6443"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-white mb-2">
              Certificate Renewal Period
            </label>
            <select className="w-full bg-slate-700 rounded-lg px-4 py-2 text-white">
              <option>30 days</option>
              <option>60 days</option>
              <option>90 days</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-white mb-2">
              Auto-scaling
            </label>
            <div className="flex items-center gap-4">
              <label className="flex items-center gap-2">
                <input type="radio" name="scaling" className="text-blue-500" />
                <span className="text-white">Enabled</span>
              </label>
              <label className="flex items-center gap-2">
                <input type="radio" name="scaling" className="text-blue-500" />
                <span className="text-white">Disabled</span>
              </label>
            </div>
          </div>

          <div className="pt-4">
            <button
              type="submit"
              className="bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 flex items-center gap-2"
            >
              <Save className="w-4 h-4" />
              Save Settings
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
