import React, { useState, useEffect, useCallback } from 'react';
import { ConnectionLines } from './ConnectionLines';
import { NodeRenderer } from './NodeRenderer';
import { ClusterControls } from './ClusterControls';

const VisualizationContainer = ({ nodes, clusterData, onRearrange, setNodes }) => {
    const [containerBounds, setContainerBounds] = useState({
        width: 0,
        height: 0,
    });
    const [hoveredNode, setHoveredNode] = useState(null);

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


    const handleMouseMove = useCallback(
        (e, draggedNode, dragOffset) => {
            if (draggedNode && containerBounds) {
                e.preventDefault();

                // Calculate new position relative to container
                let newX = e.clientX - containerBounds.left - dragOffset.x;
                let newY = e.clientY - containerBounds.top - dragOffset.y;

                // Add padding for bounds checking
                const padding = 50;
                newX = Math.max(padding, Math.min(containerBounds.width - padding, newX));
                newY = Math.max(padding, Math.min(containerBounds.height - padding, newY));

                setNodes((prev) => {
                    if (!prev) return prev;

                    if (draggedNode.type === 'control') {
                        return {
                            ...prev,
                            controlPlane: { ...prev.controlPlane, x: newX, y: newY },
                        };
                    } else {
                        return {
                            ...prev,
                            workers: prev.workers.map((worker) =>
                                worker.id === draggedNode.id ? { ...worker, x: newX, y: newY } : worker
                            ),
                        };
                    }
                });
            }
        },
        [containerBounds, setNodes]
    );

    return (
        <div className="relative w-full h-[500px] rounded-xl bg-slate-900 border border-slate-800">
            {/* Controls */}
            <div className="z-10">
                <ClusterControls onRearrange={onRearrange} />
            </div>

            <div className="visualization-container absolute inset-0 h-full p-3">
                <div className="relative w-full h-full bg-slate-800/50">
                    <ConnectionLines nodes={nodes} />
                    {nodes && (
                        <>
                            <NodeRenderer
                                clusterData={clusterData}
                                nodeType="control"
                                position={nodes.controlPlane}
                                workerId={null}
                                hoveredNode={hoveredNode}
                                setHoveredNode={setHoveredNode}
                                setNodes={setNodes}
                                handleMouseMove={handleMouseMove}
                            />
                            {nodes.workers.map((worker) => (
                                <NodeRenderer
                                    key={worker.id}
                                    clusterData={clusterData}
                                    nodeType="worker"
                                    position={worker}
                                    workerId={worker.id}
                                    hoveredNode={hoveredNode}
                                    setHoveredNode={setHoveredNode}
                                    setNodes={setNodes}
                                    handleMouseMove={handleMouseMove}
                                />
                            ))}
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};

export default VisualizationContainer;
