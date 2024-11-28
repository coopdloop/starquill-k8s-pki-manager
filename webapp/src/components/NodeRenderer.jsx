import { Server, Shield, CheckCircle2, XCircle, AlertCircle, RefreshCw } from 'lucide-react';
import { useState, useCallback, useEffect } from 'react'

export const NodeRenderer = ({
    clusterData,
    nodeType,
    position,
    workerId,
    hoveredNode,
    setHoveredNode,
    setNodes,
    handleMouseMove,
}) => {

    const [draggedNode, setDraggedNode] = useState(null);
    const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });

    const isControlPlane = nodeType === 'control';
    const nodeData = isControlPlane
        ? clusterData?.control_plane
        : clusterData.workers.find((w, index) => `worker${index + 1}` === workerId);



    const handleDragStart = useCallback(
        (e, nodeType, workerId) => {
            e.preventDefault();
            e.stopPropagation();

            const bounds = e.currentTarget.getBoundingClientRect();
            setDraggedNode({
                type: nodeType,
                id: workerId || 'control', // Ensure control plane has an id
            });
            setDragOffset({
                x: e.clientX - bounds.left,
                y: e.clientY - bounds.top,
            });
        },
        []
    );

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

    const getCertStatusIcon = (status) => {
        switch (status) {
            case 'Distributed': return <CheckCircle2 className="w-4 h-4 text-green-400" />;
            case 'Generated': return <AlertCircle className="w-4 h-4 text-yellow-400" />;
            default: return <XCircle className="w-4 h-4 text-red-400" />;
        }
    };

    return (
        <div
            className="absolute cursor-move group"
            style={{
                left: position.x,
                top: position.y,
                transform: 'translate(-50%, -50%)'
            }}
            onMouseDown={(e) => handleDragStart(e, nodeType, workerId)}
            onMouseMove={(e) => handleMouseMove(e, draggedNode, dragOffset)}
            onMouseUp={() => setDraggedNode(null)}
            onMouseEnter={() => setHoveredNode(workerId || nodeType)}
            onMouseLeave={() => setHoveredNode(null)}
        >
            <div className={`flex flex-col items-center p-4 rounded-lg transition-transform ${hoveredNode === (workerId || nodeType) ? 'scale-110' : ''
                }`}>
                {isControlPlane ? (
                    <Shield className="w-12 h-12 text-blue-400" />
                ) : (
                    <Server className="w-12 h-12 text-green-400" />
                )}
                <span className="text-white text-sm font-medium mt-2">
                    {isControlPlane ? 'Control Plane' : `Worker ${workerId.slice(-1)}`}
                </span>
                <span className="text-slate-400 text-xs">
                    {nodeData?.ip || 'Loading...'}
                </span>
            </div>

            {hoveredNode === (workerId || nodeType) && nodeData?.certs && (
                <div className="absolute z-10 bg-slate-800/95 backdrop-blur-sm text-white p-4 rounded-xl shadow-xl -translate-y-full -translate-x-1/2 top-0 left-1/2 mt-2 w-64 border border-slate-600">
                    <div className="text-sm font-medium mb-3">Certificates</div>
                    {renderCertificateList(nodeData.certs)}
                </div>
            )}
        </div>
    );
};
