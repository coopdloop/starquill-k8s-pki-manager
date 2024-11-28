export const CertificateStats = ({ total, distributed, pending }) => (
  <div className="grid grid-cols-3 gap-4">
    <div className="bg-slate-700/50 p-4 rounded-lg">
      <div className="text-2xl font-bold text-white">{total}</div>
      <div className="text-sm text-slate-400">Total</div>
    </div>
    <div className="bg-slate-700/50 p-4 rounded-lg">
      <div className="text-2xl font-bold text-green-400">{distributed}</div>
      <div className="text-sm text-slate-400">Distributed</div>
    </div>
    <div className="bg-slate-700/50 p-4 rounded-lg">
      <div className="text-2xl font-bold text-yellow-400">{pending}</div>
      <div className="text-sm text-slate-400">Pending</div>
    </div>
  </div>
);
