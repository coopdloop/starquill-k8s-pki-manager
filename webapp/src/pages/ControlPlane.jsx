// pages/ControlPlane.jsx
import {
    Shield, CpuIcon, Database, AlertCircle,
    CheckCircle, XCircle, RefreshCw
} from 'lucide-react';
import React, { useState, useEffect } from 'react';
import { Modal } from '../components/Modal';
import { MetricCard } from '../components/MetricCard';
import CustomTooltip from '../components/CustomTooltip';
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

export const ControlPlane = () => {
    const [controlPlaneInfo, setControlPlaneInfo] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [selectedComponent, setSelectedComponent] = useState(null);
    const [showCertificates, setShowCertificates] = useState(false);

    useEffect(() => {
        fetchData();
    }, []);

    const fetchData = async () => {
        try {
            const response = await fetch('http://localhost:3000/api/control-plane');
            const result = await response.json();
            setControlPlaneInfo(result.data);
            setError(null);
        } catch (error) {
            setError('Failed to load control plane data');
            console.error('Error:', error);
        } finally {
            setLoading(false);
        }
    };

    const ControlPlaneMetrics = () => {
        if (!controlPlaneInfo) return null;

        const prepareMetricsData = (metrics) => {
            if (!metrics) return [];

            return [
                {
                    name: 'API Server',
                    'Requests/sec': metrics.api_server?.requests_per_second || 0,
                    'Request Latency (ms)': metrics.api_server?.request_latency_ms || 0,
                    'Active Watches': metrics.api_server?.active_watches || 0
                },
                {
                    name: 'Scheduler',
                    'Active Workers': metrics.scheduler?.active_workers || 0,
                    'Scheduling Latency (ms)': metrics.scheduler?.scheduling_latency_ms || 0,
                    'Pending Pods': metrics.scheduler?.pending_pods || 0
                },
                {
                    name: 'etcd',
                    'Connections': metrics.etcd?.active_connections || 0,
                    'Ops/sec': metrics.etcd?.operations_per_second || 0,
                    'Latency (ms)': metrics.etcd?.latency_ms || 0
                }
            ];
        };

        const metricsData = prepareMetricsData(controlPlaneInfo.metrics);

        return (
            <div className="bg-slate-800 rounded-lg p-6 space-y-6 mt-8">
                <h3 className="text-xl font-semibold text-white mb-4">Control Plane Metrics</h3>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="bg-slate-700 rounded p-4">
                        <h4 className="text-sm font-medium text-slate-300 mb-2">API Server</h4>
                        <div className="space-y-2">
                            <MetricItem
                                label="Requests/sec"
                                value={controlPlaneInfo.metrics?.api_server?.requests_per_second || 'N/A'}
                            />
                            <MetricItem
                                label="Request Latency"
                                value={`${controlPlaneInfo.metrics?.api_server?.request_latency_ms || 'N/A'} ms`}
                            />
                            <MetricItem
                                label="Active Watches"
                                value={controlPlaneInfo.metrics?.api_server?.active_watches || 'N/A'}
                            />
                        </div>
                    </div>

                    <div className="bg-slate-700 rounded p-4">
                        <h4 className="text-sm font-medium text-slate-300 mb-2">Scheduler</h4>
                        <div className="space-y-2">
                            <MetricItem
                                label="Active Workers"
                                value={controlPlaneInfo.metrics?.scheduler?.active_workers || 'N/A'}
                            />
                            <MetricItem
                                label="Scheduling Latency"
                                value={`${controlPlaneInfo.metrics?.scheduler?.scheduling_latency_ms || 'N/A'} ms`}
                            />
                            <MetricItem
                                label="Pending Pods"
                                value={controlPlaneInfo.metrics?.scheduler?.pending_pods || 'N/A'}
                            />
                        </div>
                    </div>

                    <div className="bg-slate-700 rounded p-4">
                        <h4 className="text-sm font-medium text-slate-300 mb-2">etcd</h4>
                        <div className="space-y-2">
                            <MetricItem
                                label="Active Connections"
                                value={controlPlaneInfo.metrics?.etcd?.active_connections || 'N/A'}
                            />
                            <MetricItem
                                label="Operations/sec"
                                value={controlPlaneInfo.metrics?.etcd?.operations_per_second || 'N/A'}
                            />
                            <MetricItem
                                label="Latency"
                                value={`${controlPlaneInfo.metrics?.etcd?.latency_ms || 'N/A'} ms`}
                            />
                        </div>
                    </div>
                </div>

                <div className="h-[300px] w-full">
                    <ResponsiveContainer width="100%" height="100%">
                        <BarChart data={metricsData}>
                            <XAxis dataKey="name" />
                            <YAxis />
                            <Tooltip
                                contentStyle={{
                                    backgroundColor: '#1e293b',
                                    borderColor: '#334155'
                                }}
                                labelStyle={{ color: 'white' }}
                                itemStyle={{ color: '#94a3b8' }}
                            />
                            <Bar dataKey="Requests/sec" stackId="a" fill="#3b82f6" />
                            <Bar dataKey="Request Latency (ms)" stackId="a" fill="#10b981" />
                            <Bar dataKey="Active Watches" stackId="a" fill="#6366f1" />
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            </div>
        );
    };

    const MetricItem = ({ label, value }) => (
        <div className="flex justify-between text-sm">
            <span className="text-slate-400">{label}</span>
            <span className="text-white font-medium">{value}</span>
        </div>
    );

    const renderMetrics = (metrics) => {
        // Define a mapping for more readable metric names
        const metricNameMap = {
            'DbSize': 'Database Size',
            'CpuUsage': 'CPU Usage',
            'MemoryUsage': 'Memory Usage',
            'RequestLatency': 'Request Latency',
            'RequestRate': 'Request Rate'
        };

        if (!metrics) return <p className="text-slate-400">No metrics available</p>;

        return (
            <div className="space-y-2">
                {Object.entries(metrics).map(([key, value]) => (
                    <div key={key} className="flex justify-between text-sm">
                        <span className="text-slate-400">
                            {metricNameMap[key] || key}
                        </span>
                        <span className="text-white">{value}</span>
                    </div>
                ))}
            </div>
        );
    };

    const ComponentDetails = ({ component }) => {
        if (!component) return null;

        return (
            <div className="space-y-6">
                <div className="grid grid-cols-2 gap-4">
                    <div className="bg-slate-700 rounded p-4">
                        <h4 className="text-sm font-medium text-slate-300 mb-2">Version</h4>
                        <p className="text-white">{component.version}</p>
                    </div>
                    <div className="bg-slate-700 rounded p-4">
                        <h4 className="text-sm font-medium text-slate-300 mb-2">Health Status</h4>
                        <div className="flex items-center gap-2">
                            {component.status === 'Healthy' ? (
                                <CheckCircle className="w-4 h-4 text-green-400" />
                            ) : (
                                <XCircle className="w-4 h-4 text-red-400" />
                            )}
                            <span className="text-white">{component.status}</span>
                        </div>
                    </div>
                </div>

                <div className="bg-slate-700 rounded p-4">
                    <h4 className="text-sm font-medium text-slate-300 mb-2">Metrics</h4>
                    {renderMetrics(component.metrics)}
                </div>

                {component.extra_info && (
                    <div className="bg-slate-700 rounded p-4">
                        <h4 className="text-sm font-medium text-slate-300 mb-2">Additional Info</h4>
                        <p className="text-white">{component.extra_info}</p>
                    </div>
                )}
            </div>
        );
    };

    const CertificateModal = () => (
        <Modal
            isOpen={showCertificates}
            onClose={() => setShowCertificates(false)}
            title="Control Plane Certificates"
        >
            <div className="space-y-4">
                {controlPlaneInfo?.certificates?.map((cert, index) => (
                    <div key={index} className="bg-slate-700 rounded p-4">
                        <div className="flex justify-between items-center mb-2">
                            <h4 className="font-medium text-white">{cert.name}</h4>
                            <span className={`px-2 py-1 rounded-full text-xs ${cert.status === 'Valid'
                                ? 'bg-green-400/10 text-green-400'
                                : 'bg-yellow-400/10 text-yellow-400'
                                }`}>
                                {cert.status}
                            </span>
                        </div>
                        <div className="grid grid-cols-2 gap-4 text-sm">
                            <div>
                                <span className="text-slate-400">Expires</span>
                                <p className="text-white">{new Date(cert.expires).toLocaleDateString()}</p>
                            </div>
                            <div>
                                <span className="text-slate-400">Issuer</span>
                                <p className="text-white">{cert.issuer}</p>
                            </div>
                        </div>
                    </div>
                ))}
            </div>
        </Modal>
    );

    if (loading) {
        return (
            <div className="p-6">
                <div className="flex items-center gap-2 text-slate-400">
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    Loading control plane data...
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="p-6">
                <div className="flex items-center gap-2 text-red-400">
                    <AlertCircle className="w-4 h-4" />
                    {error}
                </div>
            </div>
        );
    }

    return (
        <div className="p-6">
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 className="text-2xl font-semibold text-white">Control Plane</h1>
                    <p className="text-slate-400">Control plane components and health status</p>
                </div>
                <div className="flex gap-2">
                    <CustomTooltip content="View Certificates">
                        <button
                            onClick={() => setShowCertificates(true)}
                            className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 transition-colors"
                        >
                            <Shield className="w-5 h-5 text-blue-400" />
                        </button>
                    </CustomTooltip>
                    <CustomTooltip content="Refresh Data">
                        <button
                            onClick={fetchData}
                            className="p-2 rounded-lg bg-slate-700 hover:bg-slate-600 transition-colors"
                        >
                            <RefreshCw className="w-5 h-5 text-green-400" />
                        </button>
                    </CustomTooltip>
                </div>
            </div>

            <div className="flex flex-col gap-6">
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <MetricCard
                        icon={Shield}
                        title="API Server"
                        value={controlPlaneInfo?.api_server.version}
                        status={controlPlaneInfo?.api_server.status}
                        secondaryValue={{
                            label: "Uptime",
                            value: controlPlaneInfo?.api_server.uptime
                        }}
                        onClick={() => setSelectedComponent(controlPlaneInfo?.api_server)}
                    />

                    <MetricCard
                        icon={Database}
                        title="etcd"
                        value={controlPlaneInfo?.etcd.version}
                        status={controlPlaneInfo?.etcd.status}
                        secondaryValue={{
                            label: "Extra Info",
                            value: controlPlaneInfo?.etcd.extra_info
                        }}
                        onClick={() => setSelectedComponent(controlPlaneInfo?.etcd)}
                    />

                    <MetricCard
                        icon={CpuIcon}
                        title="Scheduler"
                        value={controlPlaneInfo?.scheduler.version}
                        status={controlPlaneInfo?.scheduler.status}
                        secondaryValue={{
                            label: "Uptime",
                            value: controlPlaneInfo?.scheduler.uptime
                        }}
                        onClick={() => setSelectedComponent(controlPlaneInfo?.scheduler)}
                    />
                </div>
            </div>

            <div className="flex flex-col gap-6">
                {/* Existing component cards */}
                <ControlPlaneMetrics />
            </div>

            <Modal
                isOpen={!!selectedComponent}
                onClose={() => setSelectedComponent(null)}
                title="Component Details"
            >
                <ComponentDetails component={selectedComponent} />
            </Modal>

            <CertificateModal />
        </div>
    );
};
