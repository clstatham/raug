import { errorMessage, logMessage } from "./log";
import initWasm, { Edge, Graph, Node, getMemory } from "../../../pkg/raug_wasm";

type AudioWorkletMessage =
    | { type: "need"; samples?: number }
    | { type: "log"; msg: string }
    | { type: "error"; msg: string };

export default class GraphHandler {
    private readonly blocks: number;
    private readonly frames: number;
    private readonly channels: number;
    private readonly sampleRate: number;
    private readonly blockSamples: number;
    private readonly queueSamples: number;
    private readonly shared: SharedArrayBuffer;
    private readonly samples: Float32Array;
    private readonly meta: SharedArrayBuffer;
    private readonly flags: Int32Array; // [writeIndex, readIndex, availableSamples, version]
    public graph: Graph | null = null;
    private memory: WebAssembly.Memory | null = null;
    private ctx: AudioContext | null = null;
    private running: boolean = false;

    constructor(blocks: number, frames: number, channels: number) {
        this.blocks = blocks;
        this.frames = frames;
        this.channels = channels;
        this.sampleRate = 48_000;
        this.blockSamples = frames * channels;
        this.queueSamples = blocks * this.blockSamples;

        this.shared = new SharedArrayBuffer(
            Float32Array.BYTES_PER_ELEMENT * this.queueSamples
        );
        this.samples = new Float32Array(this.shared);
        this.meta = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4);
        this.flags = new Int32Array(this.meta);
    }

    async init(): Promise<void> {
        if (this.graph) {
            return;
        }

        await initWasm();

        this.graph = new Graph();

        const osc = this.graph.sineOscillator();
        this.graph.connectFloatParam(440.0, osc.input(0));
        this.graph.connectAudioOutput(osc.output(0));
        this.graph.connectAudioOutput(osc.output(0));

        logMessage(
            `Graph initialized with ${this.graph.nodeCount()} nodes and ${this.graph.numAudioOutputs()} audio outputs.`
        );
    }

    async start(): Promise<void> {
        if (!this.graph) {
            throw new Error(
                "Graph handler has not been initialised. Call init() first."
            );
        }

        this.ctx = new AudioContext({ sampleRate: this.sampleRate });

        this.graph.allocate(this.sampleRate, this.frames);
        this.memory = getMemory() as WebAssembly.Memory;
        logMessage(`AudioContext started at ${this.sampleRate} Hz.`);

        const moduleUrl = new URL("./raug-worker.ts", import.meta.url);
        await this.ctx.audioWorklet.addModule(moduleUrl);

        const node = new AudioWorkletNode(this.ctx, "raug-worker", {
            numberOfInputs: 0,
            numberOfOutputs: 1,
            outputChannelCount: [this.channels],
            processorOptions: {
                channels: this.channels,
                blockSamples: this.blockSamples,
                queueSamples: this.queueSamples,
                audioBuffer: this.shared,
                controlBuffer: this.meta,
            },
        });

        node.port.onmessage = (event: MessageEvent<AudioWorkletMessage>) => {
            const { data } = event;

            switch (data.type) {
                case "need": {
                    const minSamples = data.samples ?? this.blockSamples;
                    const target = minSamples * this.blocks;
                    while (Atomics.load(this.flags, 2) < target) {
                        this.enqueueBlock();
                    }
                    break;
                }
                case "log":
                    logMessage(data.msg);
                    break;
                case "error":
                    errorMessage(data.msg);
                    break;
            }
        };

        node.connect(this.ctx.destination);

        this.running = true;
    }

    async stop(): Promise<void> {
        if (!this.ctx) {
            return;
        }

        this.running = false;

        await this.ctx.close();
        this.ctx = null;

        logMessage("AudioContext stopped.");
    }

    isRunning(): boolean {
        return this.running;
    }

    private enqueueBlock(): void {
        if (!this.graph || !this.memory) {
            errorMessage("Graph handler is not ready yet.");
            return;
        }

        try {
            this.graph.process();
        } catch (error) {
            errorMessage("Error during graph processing:", error);
            return;
        }

        const ptr = this.graph.outputBufferPtr();
        const wasmView = new Float32Array(
            this.memory.buffer,
            ptr,
            this.blockSamples
        );
        const writeIdx = Atomics.load(this.flags, 0);

        for (let i = 0; i < this.blockSamples; i += 1) {
            this.samples[(writeIdx + i) % this.queueSamples] = wasmView[i];
        }

        Atomics.store(
            this.flags,
            0,
            (writeIdx + this.blockSamples) % this.queueSamples
        );
        Atomics.add(this.flags, 2, this.blockSamples);
        Atomics.add(this.flags, 3, 1);
    }

    nodeName(node: Node): string | null {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return null;
        }

        return this.graph.nodeName(node);
    }

    nodeInputNames(node: Node): string[] | null {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return null;
        }

        return this.graph.nodeInputNames(node);
    }

    nodeOutputNames(node: Node): string[] | null {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return null;
        }

        return this.graph.nodeOutputNames(node);
    }

    allNodes(): Node[] {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return [];
        }

        return this.graph.allNodes();
    }

    allEdges(): Edge[] {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return [];
        }

        return this.graph.allEdges();
    }

    createNode(name: string): Node | null {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return null;
        }

        try {
            let fn = this.graph[name as keyof Graph];
            if (typeof fn !== "function") {
                errorMessage(`Unknown processor type: ${name}`);
                return null;
            }

            const node = (fn as () => Node).call(this.graph);
            return node;
        } catch (error) {
            errorMessage(`Failed to create node of type ${name}:`, error);
            return null;
        }
    }

    connectNodes(
        source: Node,
        sourceHandle: number,
        target: Node,
        targetHandle: number
    ): Edge | null {
        if (!this.graph) {
            errorMessage("Graph handler is not ready yet.");
            return null;
        }

        try {
            return this.graph.connectRaw(
                source,
                sourceHandle,
                target,
                targetHandle
            );
        } catch (error) {
            errorMessage(
                `Failed to connect nodes (${sourceHandle} -> ${targetHandle}):`,
                error
            );
            return null;
        }
    }
}

export const graphHandler = new GraphHandler(8, 128, 2);
