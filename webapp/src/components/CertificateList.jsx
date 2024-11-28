import { StatusBadge } from "./ui/StatusBadge";

export const CertificateList = ({ certificates }) => (
  <div className="space-y-2">
    {certificates.map((cert, idx) => (
      <div key={idx} className="bg-slate-700 rounded-lg p-3">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="text-white font-medium">{cert.cert_type}</h4>
            <p className="text-slate-400 text-sm">
              {new Date(cert.last_updated).toLocaleString()}
            </p>
          </div>
          <StatusBadge status={cert.status} />
        </div>
      </div>
    ))}
  </div>
);
