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
import ProcessorNode from "./nodes/processor-node";
import { Edge, FloatParam, Node } from "../../../pkg/raug_wasm";
import NumberNode from "./nodes/number-node";

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

    addProcessorNode(raugNode: Node, position: Position) {
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
        // nodeIds.forEach((nodeId, index) => {
        //     positions.set(nodeId, { x: sorted.indexOf(nodeId), y: index });
        // });
        nodeIds.forEach((nodeId, index) => {
            positions.set(nodeId, { x: b, y: index });
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

    const positions = sortNodesTopologically();

    for (const node of nodes) {
        const position = positions?.get(node.id()) ?? { x: 0, y: 0 };
        position.x *= 200;
        position.y *= 100;

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

    const nodeTypes = {
        processor: ProcessorNode,
        numberParam: NumberNode,
    };

    return (
        <div style={{ height: 400, border: "1px solid #cccccc" }}>
            <ReactFlow
                nodeTypes={nodeTypes}
                nodes={store.nodes}
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
