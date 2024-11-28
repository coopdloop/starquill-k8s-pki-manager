// pages/Certificates.jsx
import { ShieldCheck, Clock, AlertCircle } from 'lucide-react';

export const Certificates = () => {
  const certificates = [
    {
      name: 'API Server Certificate',
      expires: '2024-12-31',
      status: 'Valid',
      type: 'Server',
      issuer: 'Kubernetes CA'
    },
    {
      name: 'etcd Peer Certificate',
      expires: '2024-12-31',
      status: 'Valid',
      type: 'Peer',
      issuer: 'Kubernetes CA'
    },
    {
      name: 'Kubelet Client Certificate',
      expires: '2024-12-31',
      status: 'Warning',
      type: 'Client',
      issuer: 'Kubernetes CA'
    }
  ];

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-white">Certificates</h1>
        <p className="text-slate-400">Manage cluster certificates and security</p>
      </div>

      <div className="space-y-4">
        {certificates.map((cert, index) => (
          <div key={index} className="bg-slate-800 rounded-lg p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-4">
                <ShieldCheck className={`w-8 h-8 ${
                  cert.status === 'Valid' ? 'text-green-400' : 'text-yellow-400'
                }`} />
                <div>
                  <h3 className="text-lg font-medium text-white">{cert.name}</h3>
                  <p className="text-sm text-slate-400">{cert.type}</p>
                </div>
              </div>
              <span className={`px-3 py-1 rounded-full ${
                cert.status === 'Valid'
                  ? 'bg-green-400/10 text-green-400'
                  : 'bg-yellow-400/10 text-yellow-400'
              } text-sm`}>
                {cert.status}
              </span>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="flex items-center gap-3">
                <Clock className="w-5 h-5 text-blue-400" />
                <div>
                  <div className="text-sm text-slate-400">Expires</div>
                  <div className="text-sm font-medium text-white">{cert.expires}</div>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <AlertCircle className="w-5 h-5 text-purple-400" />
                <div>
                  <div className="text-sm text-slate-400">Issuer</div>
                  <div className="text-sm font-medium text-white">{cert.issuer}</div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
