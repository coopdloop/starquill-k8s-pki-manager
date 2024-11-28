import { ClusterVisualization } from '../components/ClusterVisualization';

export const Dashboard = () => {
  return (
    <div className="md:p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Starquill Dashboard</h1>
        <p className="text-slate-400">Manage and monitor your Kubernetes cluster</p>
      </div>
      <ClusterVisualization />
    </div>
  );
};
