// pages/WorkerNodes.jsx
import { Server, HardDrive, MemoryStick, Cpu } from 'lucide-react';

export const WorkerNodes = () => {
  const workers = [
    {
      id: 'worker1',
      name: 'Worker 1',
      ip: '192.168.1.101',
      status: 'Ready',
      cpu: '45%',
      memory: '60%',
      disk: '32%'
    },
    {
      id: 'worker2',
      name: 'Worker 2',
      ip: '192.168.1.102',
      status: 'Ready',
      cpu: '28%',
      memory: '45%',
      disk: '25%'
    }
  ];

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Worker Nodes</h1>
        <p className="text-slate-400">Monitor and manage cluster worker nodes</p>
      </div>

      <div className="space-y-4">
        {workers.map(worker => (
          <div key={worker.id} className="bg-slate-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                <Server className="w-8 h-8 text-green-400" />
                <div>
                  <h3 className="text-lg font-medium text-white">{worker.name}</h3>
                  <p className="text-sm text-slate-400">{worker.ip}</p>
                </div>
              </div>
              <span className="px-3 py-1 rounded-full bg-green-400/10 text-green-400 text-sm">
                {worker.status}
              </span>
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div className="flex items-center gap-3">
                <Cpu className="w-5 h-5 text-blue-400" />
                <div>
                  <div className="text-sm font-medium text-white">CPU</div>
                  <div className="text-xs text-slate-400">{worker.cpu}</div>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <MemoryStick className="w-5 h-5 text-purple-400" />
                <div>
                  <div className="text-sm font-medium text-white">Memory</div>
                  <div className="text-xs text-slate-400">{worker.memory}</div>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <HardDrive className="w-5 h-5 text-yellow-400" />
                <div>
                  <div className="text-sm font-medium text-white">Disk</div>
                  <div className="text-xs text-slate-400">{worker.disk}</div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
