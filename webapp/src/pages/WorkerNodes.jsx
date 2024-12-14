// pages/WorkerNodes.jsx
import { Server, HardDrive, MemoryStick, Cpu, Loader2, ShieldCheck, AlertTriangle } from 'lucide-react';
import React, { useState, useEffect } from 'react';

const CustomTooltip = ({ children, content }) => {
    const [show, setShow] = useState(false);

    return (
        <div className="relative inline-block">
            <div
                onMouseEnter={() => setShow(true)}
                onMouseLeave={() => setShow(false)}
            >
                {children}
            </div>
            {show && (
                <div className="absolute z-50 w-64 p-2 text-sm bg-slate-900 text-white rounded-md shadow-lg -translate-x-1/2 left-1/2 mt-1">
                    {content}
                </div>
            )}
        </div>
    );
};

const MetricBar = ({ value, colorClass = "bg-blue-400" }) => {
    const percentage = parseInt(value);
    return (
        <div className="w-full h-2 bg-slate-700 rounded-full overflow-hidden mt-1">
            <div
                className={`h-full ${colorClass} transition-all duration-500`}
                style={{ width: `${percentage}%` }}
            />
        </div>
    );
};

export const WorkerNodes = () => {
    const [workers, setWorkers] = useState([]);
    const [clusterData, setClusterData] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [workersResponse, clusterResponse] = await Promise.all([
                    fetch('http://localhost:3000/api/worker-nodes'),
                    fetch('http://localhost:3000/api/cluster')
                ]);
                const [workersResult, clusterResult] = await Promise.all([
                    workersResponse.json(),
                    clusterResponse.json()
                ]);

                // Join the worker node data with the relevant cluster data
                const joinedWorkers = workersResult.data.map(worker => {
                    const clusterWorker = clusterResult.data.workers.find(w => w.ip === worker.ip);
                    return {
                        ...worker,
                        certs: clusterWorker?.certs || [],
                        isReachable: !clusterResult.data.connectivity.unreachable_nodes.includes(worker.ip)
                    };
                });

                console.log(joinedWorkers)
                setWorkers(joinedWorkers);
                setClusterData(clusterResult.data);
            } catch (error) {
                console.error('Error fetching data:', error);
                setError('Failed to load worker nodes and cluster data');
            } finally {
                setLoading(false);
            }
        };

        fetchData();
        // Uncomment for live updates:
        // const interval = setInterval(fetchData, 5000);
        // return () => clearInterval(interval);
    }, []);

    if (loading) {
        return (
            <div className="p-6">
                <div className="flex items-center gap-2 text-slate-400">
                    <Loader2 className="w-4 h-4 animate-spin" />
                    Loading worker nodes...
                </div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="p-6">
                <div className="flex items-center gap-2 text-red-400">
                    <AlertTriangle className="w-4 h-4" />
                    {error}
                </div>
            </div>
        );
    }


    return (
        <div className="p-6">
            <div className="mb-6">
                <h1 className="text-2xl font-semibold text-white">Worker Nodes</h1>
                <p className="text-slate-400">Monitor and manage cluster worker nodes</p>
            </div>

            {/* Connectivity Overview */}
            <div className="mb-6 bg-slate-800 rounded-lg p-4">
                <div className="grid grid-cols-3 gap-4">
                    <div>
                        <p className="text-sm text-slate-400">Total Nodes</p>
                        <p className="text-xl font-medium text-white">{clusterData?.connectivity.total_nodes - 1}</p>
                    </div>
                    <div>
                        <p className="text-sm text-slate-400">Available</p>
                        <p className="text-xl font-medium text-green-400">{clusterData?.connectivity.available_nodes - 1}</p>
                    </div>
                    <div>
                        <p className="text-sm text-slate-400">Unreachable</p>
                        <p className="text-xl font-medium text-red-400">
                            {clusterData?.connectivity.unreachable_nodes.length}
                        </p>
                    </div>
                </div>
            </div>

            <div className="space-y-4">
                {workers.map(worker => (
                    <div key={worker.id} className="bg-slate-800 rounded-lg p-6 transition-all hover:bg-slate-700/50">
                        <div className="flex items-center justify-between mb-6">
                            <div className="flex items-center gap-4">
                                <CustomTooltip content={
                                    <div>
                                        <p className="font-medium mb-1">Node Information</p>
                                        <p>Status: {worker.status}</p>
                                        <p>Node ID: {worker.id}</p>
                                        {worker.certs?.length > 0 && (
                                            <div className="mt-2">
                                                <p className="font-medium">Certificates:</p>
                                                {worker.certs.map((cert, i) => (
                                                    <div key={i} className="ml-2 mt-1">
                                                        • {cert.name} ({cert.status})
                                                    </div>
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                }>
                                    <Server className={`w-8 h-8 ${worker.isReachable ? 'text-green-400' : 'text-red-400'}`} />
                                </CustomTooltip>
                                <div>
                                    <h3 className="text-lg font-medium text-white">{worker.name}</h3>
                                    <p className="text-sm text-slate-400">{worker.ip}</p>
                                </div>
                            </div>
                            <div className="flex items-center gap-3">
                                {worker.certs?.some(cert => cert.status === 'Valid') && (
                                    <CustomTooltip content="Node certificates are valid">
                                        <ShieldCheck className="w-5 h-5 text-green-400" />
                                    </CustomTooltip>
                                )}
                                <span className={`px-3 py-1 rounded-full ${worker.isReachable
                                    ? 'bg-green-400/10 text-green-400'
                                    : 'bg-red-400/10 text-red-400'
                                    } text-sm`}>
                                    {worker.isReachable ? 'Connected' : 'Unreachable'}
                                </span>
                            </div>
                        </div>
                        {worker.isReachable &&
                            <div className="grid grid-cols-3 gap-6">
                                <CustomTooltip content={`CPU Usage: ${worker.metrics.cpu}`}>
                                    <div className="bg-slate-700/50 rounded-lg p-4">
                                        <div className="flex items-center justify-between mb-2">
                                            <div className="flex items-center gap-2">
                                                <Cpu className="w-5 h-5 text-blue-400" />
                                                <div className="text-sm font-medium text-white">CPU</div>
                                            </div>
                                            <div className="text-sm text-blue-400">{worker.metrics.cpu}</div>
                                        </div>
                                        <MetricBar value={worker.metrics.cpu} colorClass="bg-blue-400" />
                                    </div>
                                </CustomTooltip>

                                <CustomTooltip content={`Memory Usage: ${worker.metrics.memory}`}>
                                    <div className="bg-slate-700/50 rounded-lg p-4">
                                        <div className="flex items-center justify-between mb-2">
                                            <div className="flex items-center gap-2">
                                                <MemoryStick className="w-5 h-5 text-purple-400" />
                                                <div className="text-sm font-medium text-white">Memory</div>
                                            </div>
                                            <div className="text-sm text-purple-400">{worker.metrics.memory}</div>
                                        </div>
                                        <MetricBar value={worker.metrics.memory} colorClass="bg-purple-400" />
                                    </div>
                                </CustomTooltip>

                                <CustomTooltip content={`Disk Usage: ${worker.metrics.disk}`}>
                                    <div className="bg-slate-700/50 rounded-lg p-4">
                                        <div className="flex items-center justify-between mb-2">
                                            <div className="flex items-center gap-2">
                                                <HardDrive className="w-5 h-5 text-yellow-400" />
                                                <div className="text-sm font-medium text-white">Disk</div>
                                            </div>
                                            <div className="text-sm text-yellow-400">{worker.metrics.disk}</div>
                                        </div>
                                        <MetricBar value={worker.metrics.disk} colorClass="bg-yellow-400" />
                                    </div>
                                </CustomTooltip>
                            </div>
                        }
                    </div>
                ))}
            </div>
        </div>
    );
};
