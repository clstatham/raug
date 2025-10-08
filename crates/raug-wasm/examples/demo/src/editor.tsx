import { graphHandler } from "./graph-handler";
import { errorMessage, logMessage } from "./log";
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
import "./index.css";

import { createWithEqualityFn } from "zustand/traditional";
import { shallow } from "zustand/shallow";
import ProcessorNode from "./nodes/processor-node";
import { Edge, FloatParam, Node } from "../../../pkg/raug_wasm";
import NumberNode from "./nodes/number-node";
import CustomEdge from "./edge";
import DacNode from "./nodes/dac-node";
import { Button } from "./components/ui/button";
import { useTheme } from "./theme-provider";

export const useEditorStore = createWithEqualityFn((set: any, get: any) => ({
    nodes: [] as any[],
    edges: [] as any[],
    isRunning: false,

    toggleRunning() {
        if (get().isRunning) {
            set({ isRunning: false });
            graphHandler.stop();
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
        for (const change of changes) {
            if (change.type === "remove") {
                logMessage("Removing edge:", change);
                const edge = get().edges.find((e: any) => e.id === change.id);
                if (edge) {
                    const ids = change.id.split("-");
                    if (ids.length === 4) {
                        const sourceId = parseInt(ids[0], 10);
                        const sourceHandle = parseInt(ids[1], 10);
                        const targetId = parseInt(ids[2], 10);
                        const targetHandle = parseInt(ids[3], 10);

                        const sourceNode = new Node(sourceId);
                        const targetNode = new Node(targetId);

                        graphHandler.graph?.disconnectRaw(
                            sourceNode,
                            sourceHandle,
                            targetNode,
                            targetHandle
                        );
                    } else {
                        errorMessage(
                            `Invalid edge id format: ${change.id}, expected format: sourceId-sourceHandle-targetId-targetHandle`
                        );
                    }
                } else {
                    errorMessage(`Edge not found: ${change.id}`);
                }
            } else {
                logMessage("Edge change:", change);
            }
        }

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

    addDacNode(raugNode: Node, index: number, position?: Position) {
        const node = {
            id: raugNode.id().toString(),
            data: {
                label: "DAC",
                node: raugNode,
                name: "DAC",
                index: index,
            },
            type: "dac",
            position: position ?? {
                x: Math.random() * 250,
                y: Math.random() * 250,
            },
        };

        logMessage("Adding DAC node:", node);

        set((state: any) => {
            return {
                nodes: [...state.nodes, node],
            };
        });
    },

    createDacNode(): string | null {
        const index = graphHandler.graph?.numAudioOutputs() ?? 0;
        const raugNode = graphHandler.graph?.audioOutput()!;

        if (!raugNode) {
            errorMessage("Failed to create DAC node");
            return null;
        }

        get().addDacNode(raugNode, index);

        return raugNode.id().toString();
    },

    addNumberNode(raugNode: Node, raugParam: FloatParam, position?: Position) {
        const node = {
            id: raugNode.id().toString(),
            data: {
                label: "Number",
                node: raugNode,
                set: (v: number) => {
                    raugParam.set(v);
                },
                get: () => {
                    return raugParam.get();
                },
            },
            type: "number",
            position: position ?? {
                x: Math.random() * 250,
                y: Math.random() * 250,
            },
        };

        logMessage("Adding number param node:", node, "at position", position);

        set((state: any) => {
            return {
                nodes: [...state.nodes, node],
            };
        });
    },

    createNumberNode(value: number): string | null {
        const raugNode = graphHandler.graph?.floatParam(value)!;

        if (!raugNode) {
            errorMessage("Failed to create number param");
            return null;
        }

        get().addNumberNode(
            raugNode,
            graphHandler.graph?.getFloatParam(raugNode)!
        );

        return raugNode.id().toString();
    },

    addProcessorNode(raugNode: Node, position: Position) {
        const nodeName = graphHandler.graph?.nodeName(raugNode) ?? "Node";
        const nodeOutputs = graphHandler.graph?.nodeOutputNames(raugNode) ?? [];
        const nodeInputs = graphHandler.graph?.nodeInputNames(raugNode) ?? [];

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
            errorMessage(`Failed to create node of type ${name}`);
            return null;
        }

        get().addProcessorNode(raugNode);

        return raugNode.id().toString();
    },

    addEdge(raugEdge: Edge) {
        const sourceNode = raugEdge.sourceNode();
        const targetNode = raugEdge.targetNode();
        const sourceOutput = raugEdge.sourceOutputIndex();
        const targetInput = raugEdge.targetInputIndex();

        const id = `${sourceNode.id()}-${sourceOutput}-${targetNode.id()}-${targetInput}`;

        const connection = {
            id: id,
            type: "custom",
            source: sourceNode.id().toString(),
            sourceHandle: sourceOutput.toString(),
            target: targetNode.id().toString(),
            targetHandle: targetInput.toString(),
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
            errorMessage(
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

/**
Topological sort of the nodes in the graph, returning x-positions based on in-degree and y-positions based on breadth.
*/
function sortNodesTopologically(): Map<number, Position> | null {
    if (!graphHandler) {
        logMessage("Graph handler is not ready yet.");
        return null;
    }

    const nodes = graphHandler.graph?.allNodes();
    const edges = graphHandler.graph?.allEdges();

    if (!nodes || !edges) {
        logMessage("Graph is empty.");
        return null;
    }

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
        errorMessage("Graph has at least one cycle, topological sort failed.");
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
            positions.set(nodeId, { x: b, y: index });
        });
    }

    return positions;
}

export function populateNodes() {
    if (!graphHandler) {
        errorMessage("Graph handler is not ready yet.");
        return;
    }

    const store = useEditorStore.getState();

    const nodes = graphHandler.graph?.allNodes();
    const edges = graphHandler.graph?.allEdges();
    const audioOutputs = graphHandler.graph?.allAudioOutputs();

    if (!nodes || !edges) {
        logMessage("Graph is empty, nothing to populate.");
        return;
    }

    logMessage(
        `Populating editor with ${nodes.length} nodes and ${edges.length} edges.`
    );

    const positions = sortNodesTopologically();

    for (const node of nodes) {
        const position = positions?.get(node.id()) ?? { x: 0, y: 0 };
        position.x *= 200;
        position.y *= 100;

        if (graphHandler.graph?.isFloatParam(node)) {
            store.addNumberNode(
                node,
                graphHandler.graph?.getFloatParam(node)!,
                position
            );
        } else if (audioOutputs?.some((n) => n.id() === node.id())) {
            const index = audioOutputs.findIndex((n) => n.id() === node.id());
            store.addDacNode(node, index, position);
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
        createProcessorNode: state.createProcessorNode,
        createEdge: state.createEdge,
        addProcessorNode: state.addProcessorNode,
        addNumberNode: state.addNumberNode,
        createNumberNode: state.createNumberNode,
        addEdge: state.addEdge,
        addFloatParamNode: state.addFloatParamNode,
    });

    const store = useEditorStore(selector, shallow);

    const nodeTypes = {
        processor: ProcessorNode,
        number: NumberNode,
        dac: DacNode,
    };

    const edgeTypes = {
        custom: CustomEdge,
    };

    const { setTheme } = useTheme();

    return (
        <div
            className="editor-container"
            style={{ height: "500px", width: "100%" }}
        >
            <ReactFlow
                className="editor-flow"
                nodeTypes={nodeTypes}
                edgeTypes={edgeTypes}
                nodes={store.nodes}
                edges={store.edges}
                onNodesChange={store.onNodeChanges}
                onEdgesChange={store.onEdgeChanges}
                onConnect={store.onConnect}
                fitView
            >
                <Panel position="bottom-left">
                    <Button
                        onClick={() => {
                            store.toggleRunning();
                        }}
                    >
                        {store.isRunning ? "Stop" : "Play"}
                    </Button>
                </Panel>
                <Panel position="top-left">
                    <Button
                        onClick={() => {
                            const nodeId = prompt(
                                "Enter processor node type (e.g., Oscillator, Gain, etc.):"
                            );
                            if (nodeId) {
                                store.createProcessorNode(nodeId);
                            }
                        }}
                    >
                        Add Node
                    </Button>
                    <Button
                        onClick={() => {
                            const value = prompt("Enter value:", "0.0");
                            if (value !== null) {
                                const floatValue = parseFloat(value);
                                if (isNaN(floatValue)) {
                                    errorMessage(
                                        "Invalid float value entered."
                                    );
                                    return;
                                }
                                store.createNumberNode(floatValue);
                            }
                        }}
                    >
                        Add Number
                    </Button>
                </Panel>
                <Panel position="top-right">
                    <Button
                        onClick={() => {
                            setTheme("light");
                        }}
                    >
                        Light Mode
                    </Button>
                    <Button
                        onClick={() => {
                            setTheme("dark");
                        }}
                    >
                        Dark Mode
                    </Button>
                    <Button
                        onClick={() => {
                            setTheme("system");
                        }}
                    >
                        System Theme
                    </Button>
                </Panel>
                <Background />
            </ReactFlow>
        </div>
    );
}
