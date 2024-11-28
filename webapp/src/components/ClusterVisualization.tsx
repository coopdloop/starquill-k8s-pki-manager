import React, { useState, useEffect, useCallback } from 'react';
import { Server, Shield, CheckCircle2, XCircle, AlertCircle, RefreshCw } from 'lucide-react';
import { NodeModal } from './Modals/NodeModal';
import { NodeCard } from './NodeCard';
import { Button } from './ui/Button'
import { StatusBadge } from './ui/StatusBadge'
import { Notification } from './Notification'
import { ActivityItem } from './ActivityItem'
import { CertificateStats } from './CertificateStats';
import VisualizationContainer from './VisualizationContainer';


const API_PORT = import.meta.env.VITE_API_PORT || 3000
const API_URL = `http://localhost:${API_PORT}/api/cluster`

type Node = {
    x: number;
    y: number;
    ip: string;
}

type NodesState = {
    controlPlane: Node;
    workers: (Node & { id: string })[];
} | null;

type Certificate = {
    status: 'Distributed' | 'Generated';
    cert_type: string;
    last_updated?: string;
}

type ClusterData = {
    control_plane: {
        ip: string;
        certs: Certificate[];
    };
    workers: Array<{
        ip: string;
        certs?: Certificate[];
    }>;
} | null;

type ActivityItem = {
    type: 'success' | 'warning' | 'info';
    message: string;
    timestamp: string;
};

export const ClusterVisualization = () => {
    const [clusterData, setClusterData] = useState<ClusterData>(null);
    const [nodes, setNodes] = useState<NodesState>(null);
    const [draggedNode, setDraggedNode] = useState(null);
    const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
    const [hoveredNode, setHoveredNode] = useState<string | 'control' | null>(null);
    const [selectedNode, setSelectedNode] = useState<string | null>(null);
    const [showNotification, setShowNotification] = useState(false);
    const [notificationType, setNotificationType] = useState('success');
    const [notificationMessage, setNotificationMessage] = useState('');
    const [containerBounds, setContainerBounds] = useState({
        width: 0,
        height: 0,
    });

    const activityItems: ActivityItem[] = [
        { type: 'success', message: 'Control plane certificates distributed', timestamp: '2m ago' },
        { type: 'warning', message: 'Worker node certificate pending', timestamp: '5m ago' },
        { type: 'info', message: 'Started certificate generation', timestamp: '10m ago' },
    ];

    useEffect(() => {
        const fetchData = async () => {
            try {
                const response = await fetch(API_URL);
                const result = await response.json();
                setClusterData(result.data);
            } catch (error) {
                console.error('Error fetching cluster data:', error);
            }
        };

        fetchData();
        // const interval = setInterval(fetchData, 2000);
        // return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        const updateBounds = () => {
            const container = document.querySelector('.visualization-container');
            if (container) {
                const bounds = container.getBoundingClientRect();
                setContainerBounds(bounds);
            }
        };

        // Run after a small delay to ensure DOM is ready
        const timer = setTimeout(updateBounds, 100);
        window.addEventListener('resize', updateBounds);

        return () => {
            clearTimeout(timer);
            window.removeEventListener('resize', updateBounds);
        };
    }, []);

    const handleMouseMove = useCallback((e) => {
        if (draggedNode && containerBounds) {
            e.preventDefault();

            // Calculate new position relative to container
            let newX = e.clientX - containerBounds.left - dragOffset.x;
            let newY = e.clientY - containerBounds.top - dragOffset.y;

            // Add padding for bounds checking
            const padding = 50;
            newX = Math.max(padding, Math.min(containerBounds.width - padding, newX));
            newY = Math.max(padding, Math.min(containerBounds.height - padding, newY));

            setNodes(prev => {
                if (!prev) return prev;

                if (draggedNode.type === 'control') {
                    return {
                        ...prev,
                        controlPlane: { ...prev.controlPlane, x: newX, y: newY }
                    };
                } else {
                    return {
                        ...prev,
                        workers: prev.workers.map(worker =>
                            worker.id === draggedNode.id
                                ? { ...worker, x: newX, y: newY }
                                : worker
                        )
                    };
                }
            });
        }
    }, [draggedNode, dragOffset, containerBounds]);

    useEffect(() => {
        console.log('Nodes state:', nodes);
        console.log('Container bounds:', containerBounds);
    }, [nodes, containerBounds]);

    // Separate connection line component for better organization
    const ConnectionLines = React.memo(({ nodes }: { nodes: NodesState }) => {
        if (!nodes?.controlPlane || !nodes?.workers) return null;

        return (
            <svg className="absolute inset-0"
                style={{ zIndex: 0 }}
                width="100%"
                height="100%"
                preserveAspectRatio="none"
            >
                {nodes.workers.map(worker => (
                    <g key={worker.id}>
                        <defs>
                            <linearGradient
                                id={`line-gradient-${worker.id}`}
                                gradientUnits="userSpaceOnUse"
                                x1={nodes.controlPlane.x}
                                y1={nodes.controlPlane.y}
                                x2={worker.x}
                                y2={worker.y}
                            >
                                <stop offset="0%" stopColor="#4f46e5" />
                                <stop offset="100%" stopColor="#06b6d4" />
                            </linearGradient>
                        </defs>
                        <line
                            x1={nodes.controlPlane.x}
                            y1={nodes.controlPlane.y}
                            x2={worker.x}
                            y2={worker.y}
                            stroke={`url(#line-gradient-${worker.id})`}
                            strokeWidth="2"
                            className="animate-pulse"
                        />
                    </g>
                ))}
            </svg>
        );
    });

    useEffect(() => {
        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', () => setDraggedNode(null));

        return () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', () => setDraggedNode(null));
        };
    }, [handleMouseMove]);


    // Update nodes when clusterData changes
    useEffect(() => {
        console.log('Updating nodes. ClusterData:', clusterData, 'Bounds:', containerBounds);
        if (clusterData && containerBounds && containerBounds.width && containerBounds.height) {
            const totalWorkers = clusterData.workers.length;
            const radius = Math.min(containerBounds.width, containerBounds.height) * 0.3;
            console.log('Calculating positions with:', {
                width: containerBounds.width,
                height: containerBounds.height,
                radius,
            });
            const centerX = containerBounds.width / 2;
            const centerY = containerBounds.height / 2;
            const workerNodes = clusterData.workers.map((worker, index) => {
                const angle = (2 * Math.PI * index) / totalWorkers;
                return {
                    id: `worker${index + 1}`,
                    x: centerX + radius * Math.cos(angle),
                    y: centerY + radius * Math.sin(angle),
                    ip: worker.ip,
                };
            });
            const newNodes = {
                controlPlane: {
                    x: centerX,
                    y: centerY,
                    ip: clusterData.control_plane.ip,
                },
                workers: workerNodes,
            };
            console.log('Setting nodes to:', newNodes);
            setNodes(newNodes);
        }
    }, [clusterData, containerBounds]);

    // Add loading state handling
    if (!clusterData) {
        return <div>Loading cluster data...</div>;
    }

    // Add debug logging
    console.log('Cluster Data:', clusterData);

    const handleDragStart = (
        e: React.MouseEvent,
        nodeType: 'control' | 'worker',
        workerId: string | null = null
    ) => {
        e.preventDefault();
        e.stopPropagation();

        const bounds = e.currentTarget.getBoundingClientRect();
        setDraggedNode({
            type: nodeType,
            id: workerId || 'control'  // Ensure control plane has an id
        });
        setDragOffset({
            x: e.clientX - bounds.left,
            y: e.clientY - bounds.top
        });
    };

    const arrangeInCircle = (containerBounds) => {
        const totalWorkers = clusterData.workers.length;
        const radius = Math.min(containerBounds.width, containerBounds.height) * 0.3;
        const centerX = containerBounds.width / 2;
        const centerY = containerBounds.height / 2;

        const workerNodes = clusterData.workers.map((worker, index) => {
            const angle = (2 * Math.PI * index) / totalWorkers;
            return {
                id: `worker${index + 1}`,
                x: centerX + radius * Math.cos(angle),
                y: centerY + radius * Math.sin(angle),
                ip: worker.ip
            };
        });

        setNodes({
            controlPlane: {
                x: centerX,
                y: centerY,
                ip: clusterData.control_plane.ip
            },
            workers: workerNodes
        });
    };

    const arrangeInGrid = (containerBounds) => {
        const padding = 100;
        const totalWorkers = clusterData.workers.length;
        const cols = Math.ceil(Math.sqrt(totalWorkers));
        const cellWidth = (containerBounds.width - padding * 2) / cols;
        const cellHeight = (containerBounds.height - padding * 2) / cols;

        const workerNodes = clusterData.workers.map((worker, index) => {
            const row = Math.floor(index / cols);
            const col = index % cols;
            return {
                id: `worker${index + 1}`,
                x: padding + col * cellWidth + cellWidth / 2,
                y: padding + row * cellHeight + cellHeight / 2,
                ip: worker.ip
            };
        });

        setNodes({
            controlPlane: {
                x: containerBounds.width / 2,
                y: padding / 2,
                ip: clusterData.control_plane.ip
            },
            workers: workerNodes
        });
    };

    const arrangeRandomly = (containerBounds) => {
        const padding = 100;
        const workerNodes = clusterData.workers.map((worker, index) => ({
            id: `worker${index + 1}`,
            x: padding + Math.random() * (containerBounds.width - padding * 2),
            y: padding + Math.random() * (containerBounds.height - padding * 2),
            ip: worker.ip
        }));

        setNodes({
            controlPlane: {
                x: containerBounds.width / 2,
                y: containerBounds.height / 2,
                ip: clusterData.control_plane.ip
            },
            workers: workerNodes
        });
    };

    const handleRearrange = (layout) => {
        if (!containerBounds) return;

        switch (layout) {
            case 'circle':
                arrangeInCircle(containerBounds);
                break;
            case 'grid':
                arrangeInGrid(containerBounds);
                break;
            case 'random':
                arrangeRandomly(containerBounds);
                break;
        }
    };


    const getCertStatusIcon = (status) => {
        switch (status) {
            case 'Distributed': return <CheckCircle2 className="w-4 h-4 text-green-400" />;
            case 'Generated': return <AlertCircle className="w-4 h-4 text-yellow-400" />;
            default: return <XCircle className="w-4 h-4 text-red-400" />;
        }
    };

    const renderCertificateList = (certs) => (
        <div className="space-y-2">
            {certs.map((cert, idx) => (
                <div key={idx} className="flex items-center space-x-2 p-2 bg-slate-700 rounded">
                    {getCertStatusIcon(cert.status)}
                    <div className="flex-1">
                        <div className="text-sm font-medium">{cert.cert_type}</div>
                        <div className="text-xs text-slate-300">
                            {cert.last_updated
                                ? new Date(cert.last_updated).toLocaleString()
                                : 'Not updated'}
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );


    if (!clusterData) {
        console.log('No cluster data available');
        return <div>Loading cluster data...</div>;
    }

    if (!nodes) {
        console.log('No nodes data available');
    }

    if (!containerBounds) {
        console.log('No container bounds available');
    }

    return (
        <div className="flex flex-col gap-4" >
            {/* Main container */}
            <div className="grid grid-cols-12 gap-6">
                {/* Left side - Visualization */}
                <div className="col-span-8">
                    <VisualizationContainer nodes={nodes} clusterData={clusterData} onRearrange={handleRearrange} setNodes={setNodes} />
                </div>
            </div>

            {/* Info panels */}
            <div className="col-span-4 space-y-4">
                {/* Node Status Panel */}
                <div className="p-4 rounded-xl bg-slate-900 border border-slate-800">
                    <div className="flex items-center justify-between mb-4">
                        <h2 className="text-lg font-medium">Node Status</h2>
                        <Button variant="secondary" size="sm" onClick={() => window.location.reload()}>
                            <RefreshCw className="w-4 h-4" />
                        </Button>
                    </div>
                    <div className="space-y-3">
                        <NodeCard
                            node={{
                                type: 'control',
                                name: 'Control Plane',
                                ip: clusterData.control_plane.ip
                            }}
                            onAction={() => setSelectedNode('control')}
                        />
                        {clusterData.workers.map((worker, index) => (
                            <NodeCard
                                key={`worker${index + 1}`}
                                node={{
                                    type: 'worker',
                                    name: `Worker ${index + 1}`,
                                    ip: worker.ip
                                }}
                                onAction={() => setSelectedNode(`worker${index + 1}`)}
                            />
                        ))}
                    </div>
                </div>

                {/* Certificate Overview Panel */}
                <div className="p-4 rounded-xl bg-slate-900 border border-slate-800">
                    <div className="flex items-center justify-between mb-4">
                        <h2 className="text-lg font-medium">Certificate Overview</h2>
                        <StatusBadge
                            status={
                                clusterData?.control_plane?.certs?.every(c => c.status === 'Distributed')
                                    ? 'Healthy'
                                    : 'Pending'
                            }
                        />
                    </div>
                    {clusterData?.control_plane?.certs && (
                        <CertificateStats
                            total={clusterData.control_plane.certs.length}
                            distributed={clusterData.control_plane.certs.filter(c => c.status === 'Distributed').length}
                            pending={clusterData.control_plane.certs.filter(c => c.status === 'Generated').length}
                        />
                    )}
                </div>

                {/* Activity Panel */}
                <div className="p-4 rounded-xl bg-slate-900 border border-slate-800">
                    <h2 className="text-lg font-medium mb-4">Recent Activity</h2>
                    <div className="space-y-3">
                        {activityItems.map((item, index) => (
                            <ActivityItem key={index} {...item} />
                        ))}
                    </div>
                </div>
            </div>

            {/* Modals */}
            {
                selectedNode && (
                    <NodeModal
                        node={selectedNode === 'control'
                            ? clusterData?.control_plane
                            : clusterData?.workers.find(w => w.ip.includes(selectedNode))
                        }
                        onClose={() => setSelectedNode(null)}
                    />
                )
            }

            {/* Notifications */}
            {
                showNotification && (
                    <div className="fixed bottom-4 right-4 z-50">
                        <Notification
                            type={notificationType}
                            message={notificationMessage}
                            onClose={() => setShowNotification(false)}
                        />
                    </div>
                )
            }
        </div >
    );
}
