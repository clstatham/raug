import { graphHandler } from "./graph-handler";
import { logMessage } from "./log";
import {
    addEdge as addFlowEdge,
    applyEdgeChanges,
    applyNodeChanges,
    Background,
    Connection,
    EdgeChange,
    Panel,
    ReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { createWithEqualityFn } from "zustand/traditional";
import { shallow } from "zustand/shallow";
import ProcessorNode from "./processor-node";
import { Edge, FloatParam, Node } from "../../../pkg/raug_wasm";
import {
    forceCenter,
    forceCollide,
    forceLink,
    forceManyBody,
    forceSimulation,
} from "d3-force";
import { useEffect, useRef, useState } from "react";
import NumberNode from "./number-node";

export const useEditorStore = createWithEqualityFn((set: any, get: any) => ({
    nodes: [] as any[],
    edges: [] as any[],
    isRunning: false,
    edgeReconnectSuccessful: true,

    toggleRunning() {
        if (get().isRunning) {
            graphHandler.stop();
            set({ isRunning: false });
        } else {
            graphHandler.start();
            set({ isRunning: true });
        }
    },

    onNodeChanges(changes: any[]) {
        set({
            nodes: applyNodeChanges(changes, get().nodes),
        });
    },

    onEdgeChanges(changes: EdgeChange[]) {
        set({
            edges: applyEdgeChanges(changes, get().edges),
        });
    },

    onConnect(connection: Connection) {
        logMessage("Connecting nodes:", connection);

        get().createEdge(
            connection.source!,
            connection.sourceHandle!,
            connection.target!,
            connection.targetHandle!
        );
    },

    addNumberParamNode(
        raugNode: Node,
        raugParam: FloatParam,
        position?: Position
    ) {
        const nodeOutputs = graphHandler.nodeOutputNames(raugNode) ?? [];
        const nodeInputs = graphHandler.nodeInputNames(raugNode) ?? [];

        const node = {
            id: raugNode.id().toString(),
            data: {
                label: "Number",
                node: raugNode,
                name: "Number",
                inputNames: nodeInputs,
                outputNames: nodeOutputs,
                setValue: (v: number) => {
                    raugParam.set(v);
                },
                getValue: () => {
                    return raugParam.get();
                },
            },
            type: "numberParam",
            position: position ?? {
                x: Math.random() * 250,
                y: Math.random() * 250,
            },
            style: {
                padding: 10,
                border: "1px solid #777",
                borderRadius: 5,
                background: "#fff",
            },
            width: 150,
            height: 80,
        };

        logMessage("Adding number param node:", node, "at position", position);

        set((state: any) => {
            return {
                nodes: [...state.nodes, node],
            };
        });
    },

    addProcessorNode(raugNode: Node, position?: Position) {
        const nodeName = graphHandler.nodeName(raugNode) ?? "Node";
        const nodeOutputs = graphHandler.nodeOutputNames(raugNode) ?? [];
        const nodeInputs = graphHandler.nodeInputNames(raugNode) ?? [];

        const numOutputs = nodeOutputs.length;
        const numInputs = nodeInputs.length;

        const spacing = Math.max(numOutputs, numInputs);
        const height = spacing * 40 + 20;

        const node = {
            id: raugNode.id().toString(),
            data: {
                label: nodeName,
                node: raugNode,
                name: nodeName,
                inputNames: nodeInputs,
                outputNames: nodeOutputs,
            },
            type: "processor",
            position: position ?? {
                x: Math.random() * 250,
                y: Math.random() * 250,
            },
            style: {
                padding: 10,
                border: "1px solid #777",
                borderRadius: 5,
                background: "#fff",
            },
            width: 150,
            height: height,
        };

        logMessage("Adding node:", node, "at position", position);

        set((state: any) => {
            return {
                nodes: [...state.nodes, node],
            };
        });
    },

    createProcessorNode(name: string): string | null {
        const raugNode = graphHandler.createNode(name)!;

        if (!raugNode) {
            logMessage(`Failed to create node of type ${name}`);
            return null;
        }

        get().addProcessorNode(raugNode);

        return raugNode.id().toString();
    },

    addEdge(raugEdge: Edge) {
        const sourceNode = raugEdge.sourceNode();
        const targetNode = raugEdge.targetNode();

        const connection: Connection = {
            source: sourceNode.id().toString(),
            sourceHandle: raugEdge.sourceOutputIndex().toString(),
            target: targetNode.id().toString(),
            targetHandle: raugEdge.targetInputIndex().toString(),
        };

        logMessage("Adding edge:", connection);

        set((state: any) => {
            const edges = addFlowEdge(connection, state.edges);
            return {
                edges,
            };
        });
    },

    createEdge(
        sourceId: string,
        sourceHandle: string,
        targetId: string,
        targetHandle: string
    ) {
        const source = parseInt(sourceId, 10);
        const target = parseInt(targetId, 10);
        const sourceOutput = parseInt(sourceHandle, 10);
        const targetInput = parseInt(targetHandle, 10);

        const sourceNode = new Node(source);
        const targetNode = new Node(target);

        const edge = graphHandler.connectNodes(
            sourceNode,
            sourceOutput,
            targetNode,
            targetInput
        );

        if (!edge) {
            logMessage(
                `Failed to create edge from node ${source} to node ${target}`
            );
            return;
        }

        get().addEdge(edge);
    },
}));

type Position = {
    x: number;
    y: number;
};

/*
Topological sort of the nodes in the graph, returning x-positions based on in-degree and y-position based on breadth.
*/
function sortNodesTopologically(): Map<number, Position> | null {
    if (!graphHandler) {
        logMessage("Graph handler is not ready yet.");
        return null;
    }

    const nodes = graphHandler.allNodes();
    const edges = graphHandler.allEdges();

    const inDegree = new Map<number, number>();
    const breadth = new Map<number, number>();
    const adjList = new Map<number, number[]>();

    for (const node of nodes) {
        inDegree.set(node.id(), 0);
        adjList.set(node.id(), []);
    }

    for (const edge of edges) {
        const targetId = edge.targetNode().id();
        const sourceId = edge.sourceNode().id();
        inDegree.set(targetId, (inDegree.get(targetId) ?? 0) + 1);
        adjList.get(sourceId)?.push(targetId);
        breadth.set(targetId, 0);
    }

    const sorted: number[] = [];
    const queue: number[] = [];

    for (const [nodeId, deg] of inDegree.entries()) {
        if (deg === 0) {
            queue.push(nodeId);
        }
    }

    while (queue.length > 0) {
        const nodeId = queue.shift()!;
        sorted.push(nodeId);

        for (const neighbor of adjList.get(nodeId) ?? []) {
            inDegree.set(neighbor, (inDegree.get(neighbor) ?? 0) - 1);
            if (inDegree.get(neighbor) === 0) {
                queue.push(neighbor);
            }
            breadth.set(neighbor, (breadth.get(nodeId) ?? 0) + 1);
        }
    }

    if (sorted.length !== nodes.length) {
        logMessage("Graph has at least one cycle, topological sort failed.");
        return null;
    }

    const positions = new Map<number, Position>();
    const breadthLevels = new Map<number, number[]>();

    for (const nodeId of sorted) {
        const b = breadth.get(nodeId) ?? 0;
        if (!breadthLevels.has(b)) {
            breadthLevels.set(b, []);
        }
        breadthLevels.get(b)!.push(nodeId);
    }

    for (const [b, nodeIds] of breadthLevels.entries()) {
        nodeIds.forEach((nodeId, index) => {
            positions.set(nodeId, { x: sorted.indexOf(nodeId), y: index });
        });
    }

    return positions;
}

export function populateNodes() {
    if (!graphHandler) {
        logMessage("Graph handler is not ready yet.");
        return;
    }

    const store = useEditorStore.getState();

    const nodes = graphHandler.allNodes();
    const edges = graphHandler.allEdges();

    logMessage(
        `Populating editor with ${nodes.length} nodes and ${edges.length} edges.`
    );

    const x_positions = sortNodesTopologically();

    for (const node of nodes) {
        const { x, y } = x_positions?.get(node.id()) ?? { x: 0, y: 0 };
        const position = {
            x: x * 200,
            y: y * 200,
        };

        if (graphHandler.graph?.isFloatParam(node)) {
            store.addNumberParamNode(
                node,
                graphHandler.graph?.getFloatParam(node)!,
                position
            );
        } else {
            store.addProcessorNode(node, position);
        }
    }

    for (const edge of edges) {
        store.addEdge(edge);
    }
}

export default function Editor() {
    const selector = (state: any) => ({
        nodes: state.nodes,
        edges: state.edges,
        isRunning: state.isRunning,
        toggleRunning: state.toggleRunning,
        handleCommand: state.handleCommand,
        onNodeChanges: state.onNodeChanges,
        onEdgeChanges: state.onEdgeChanges,
        onConnect: state.onConnect,
        createNode: state.createNode,
    });

    const store = useEditorStore(selector, shallow);
    const [nodes, setNodes] = useState(store.nodes);
    const previousNodesRef = useRef(nodes);

    useEffect(() => {
        previousNodesRef.current = nodes;
    }, [nodes]);

    useEffect(() => {
        setNodes((current: any[]) => {
            const currentById = new Map(
                current.map((node: any) => [node.id, node])
            );

            return store.nodes.map((node: any) => {
                return currentById.get(node.id) ?? node;
            });
        });
    }, [store.nodes]);

    useEffect(() => {
        if (!store.nodes.length) {
            setNodes([]);
            return;
        }

        const anyDragging = store.nodes.some((node: any) => node.dragging);

        if (anyDragging) {
            setNodes((current: any[]) => {
                const currentById = new Map(
                    current.map((node: any) => [node.id, node])
                );

                return store.nodes.map((node: any) => {
                    if (node.dragging) {
                        return node;
                    }

                    return (
                        currentById.get(node.id) ?? {
                            ...node,
                            position: node.position ?? { x: 0, y: 0 },
                        }
                    );
                });
            });
            return;
        }

        type SimulationNode = {
            id: string;
            x: number;
            y: number;
            vx?: number;
            vy?: number;
        };

        const previousNodes = previousNodesRef.current;

        const baseNodes =
            previousNodes.length === store.nodes.length
                ? previousNodes
                : store.nodes;

        const simNodes: SimulationNode[] = baseNodes.map((node: any) => ({
            id: node.id,
            x: node.position?.x ?? Math.random() * 400,
            y: node.position?.y ?? Math.random() * 400,
        }));

        const nodeLookup = new Map(simNodes.map((node) => [node.id, node]));

        const linkData = store.edges.map((edge: any) => ({
            source: edge.source,
            target: edge.target,
        }));

        const simulation = forceSimulation(simNodes)
            .force(
                "link",
                forceLink(linkData)
                    .id((d: any) => d.id)
                    .distance(200)
                    .strength(0.02)
            )
            .force("charge", forceManyBody().strength(-50))
            .force("collide", forceCollide().radius(10).strength(0.5))
            .force("center", forceCenter(400, 200).strength(0.1))
            .alpha(1)
            .alphaDecay(0.02);

        let stopped = false;

        const handleTick = () => {
            if (stopped) {
                return;
            }

            setNodes(
                store.nodes.map((node: any) => {
                    const simNode = nodeLookup.get(node.id);
                    return {
                        ...node,
                        position: {
                            x: simNode?.x ?? node.position?.x ?? 0,
                            y: simNode?.y ?? node.position?.y ?? 0,
                        },
                    };
                })
            );

            if (simulation.alpha() < 0.01) {
                stopped = true;
                simulation.stop();
            }
        };

        simulation.on("tick", handleTick);

        return () => {
            stopped = true;
            simulation.stop();
        };
    }, [store.nodes, store.edges]);

    const nodeTypes = {
        processor: ProcessorNode,
        numberParam: NumberNode,
    };

    return (
        <div style={{ height: 400, border: "1px solid #cccccc" }}>
            <ReactFlow
                nodeTypes={nodeTypes}
                nodes={nodes}
                edges={store.edges}
                onNodesChange={store.onNodeChanges}
                onEdgesChange={store.onEdgeChanges}
                onConnect={store.onConnect}
                fitView
            >
                <Panel position="top-left">
                    <button
                        onClick={() => {
                            store.toggleRunning();
                        }}
                    >
                        {store.isRunning ? "Stop" : "Play"}
                    </button>
                </Panel>
                <Background />
            </ReactFlow>
        </div>
    );
}
