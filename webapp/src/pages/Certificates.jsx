import React, { useState, useEffect, useCallback } from 'react';
import { ShieldCheck, Clock, AlertCircle, Info } from 'lucide-react';
import CustomAlert from '../components/CustomAlert';
import CustomTooltip from '../components/CustomTooltip';


const CertificateTypeInfo = {
    'root-ca': {
        description: 'Root Certificate Authority - The top-level certificate that establishes the chain of trust',
        importance: 'Critical component for cluster security',
    },
    'kubernetes-ca': {
        description: 'Kubernetes Certificate Authority - Issues certificates for Kubernetes components',
        importance: 'Required for secure communication within the cluster',
    },
    'ca-chain': {
        description: 'CA Chain Certificate - Combines root and Kubernetes CA certificates',
        importance: 'Used to verify the complete certificate chain',
    }
};

export const Certificates = () => {
    const [certificates, setCertificates] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        const fetchData = async () => {
            try {
                const response = await fetch(`http://localhost:3000/api/certificates`);
                const result = await response.json();
                if (result.data) {
                    setCertificates(result.data);
                }
            } catch (error) {
                console.error('Error fetching certificate data:', error);
                setError('Failed to load certificates');
            } finally {
                setLoading(false);
            }
        };
        fetchData();
    }, []);

    const hasWarnings = certificates.some(cert => cert.status === 'Warning');
    const daysUntilExpiry = (expiryDate) => {
        const days = Math.ceil((new Date(expiryDate) - new Date()) / (1000 * 60 * 60 * 24));
        return days;
    };

    if (loading) {
        return (
            <div className="p-6">
                <div className="animate-pulse flex space-x-4">
                    <div className="flex-1 space-y-4 py-1">
                        <div className="h-4 bg-slate-700 rounded w-3/4"></div>
                        <div className="space-y-2">
                            <div className="h-4 bg-slate-700 rounded"></div>
                            <div className="h-4 bg-slate-700 rounded w-5/6"></div>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="p-6 z-20">
            <div className="mb-6">
                <h1 className="text-2xl font-semibold text-white">Certificates</h1>

                {certificates ?
                    <p className="text-slate-400">Manage cluster certificates and security.</p>
                    :
                    <p className="text-slate-400">No Certificates generated or imported.</p>
                }
            </div>

            {hasWarnings && (
                <CustomAlert
                    title="Certificate Warnings"
                >
                    Some certificates require attention. Please review their status below.
                </CustomAlert>
            )}

            <div className="space-y-4 mt-6 overflow-x-hidden">
                {certificates.map((cert, index) => {
                    const daysLeft = daysUntilExpiry(cert.expires);
                    const certInfo = CertificateTypeInfo[cert.name];

                    return (
                        <div key={index} className="bg-slate-800 rounded-lg p-6 transition-all hover:bg-slate-700/50 z-20">
                            <div className="flex items-center justify-between mb-4">
                                <div className="flex items-center gap-4">
                                    <CustomTooltip content={
                                        <div>
                                            <p>{certInfo?.description}</p>
                                            <p className="text-xs mt-1 text-yellow-400">{certInfo?.importance}</p>
                                        </div>
                                    }>
                                        <ShieldCheck className={`w-8 h-8 ${cert.status === 'Valid' ? 'text-green-400' : 'text-yellow-400'
                                            }`} />
                                    </CustomTooltip>
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <h3 className="text-lg font-medium text-white">{cert.name}</h3>
                                            <CustomTooltip content={
                                                <div>
                                                    <p>Type: {cert.cert_type}</p>
                                                    <p>Issuer: {cert.issuer}</p>
                                                </div>
                                            }>
                                                <Info className="w-4 h-4 text-slate-400 cursor-help" />
                                            </CustomTooltip>
                                        </div>
                                        <p className="text-sm text-slate-400">{cert.cert_type}</p>
                                    </div>
                                </div>
                                <CustomTooltip content={
                                    cert.status === 'Warning'
                                        ? 'Certificate requires attention'
                                        : 'Certificate is valid and healthy'
                                }>
                                    <span className={`px-3 py-1 rounded-full ${cert.status === 'Valid'
                                        ? 'bg-green-400/10 text-green-400'
                                        : 'bg-yellow-400/10 text-yellow-400'
                                        } text-sm`}>
                                        {cert.status}
                                    </span>
                                </CustomTooltip>
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <CustomTooltip content={
                                    <div>
                                        <p>Days until expiry: {daysLeft}</p>
                                        <p className="text-xs mt-1">
                                            {daysLeft <= 30
                                                ? 'Certificate will expire soon'
                                                : 'Certificate expiration is healthy'}
                                        </p>
                                    </div>
                                }>
                                    <div className="flex items-center gap-3">
                                        <Clock className="w-5 h-5 text-blue-400" />
                                        <div>
                                            <div className="text-sm text-slate-400">Expires</div>
                                            <div className="text-sm font-medium text-white">
                                                {new Date(cert.expires).toLocaleDateString()}
                                            </div>
                                        </div>
                                    </div>
                                </CustomTooltip>

                                <div className="flex items-center gap-3">
                                    <AlertCircle className="w-5 h-5 text-purple-400" />
                                    <div>
                                        <div className="text-sm text-slate-400">Issuer</div>
                                        <div className="text-sm font-medium text-white">{cert.issuer}</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};
