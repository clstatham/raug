export default class WasmGraphHandler {
    constructor(blocks, frames, channels) {
        this.frames = frames;
        this.channels = channels;
        this.sampleRate = 48000;
        this.blockSamples = frames * channels;
        this.queueSamples = blocks * this.blockSamples;
        this.shared = new SharedArrayBuffer(Float32Array.BYTES_PER_ELEMENT * this.queueSamples);
        this.samples = new Float32Array(this.shared);
        this.meta = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4);
        this.flags = new Int32Array(this.meta); // [writeIndex, readIndex, availableSamples, version]
        this.graph = null;
        this.raug = null;
        this.memory = null;
    }

    async init() {
        if (!this.graph) {
            const raug = await import('./pkg/raug_wasm.js');
            await raug.default();
            this.raug = raug;


            this.graph = new this.raug.Graph();


            const osc = this.graph.sineOscillator();
            this.graph.connectConstant(440.0, osc.input(0));
            this.graph.connectAudioOutput(osc.output(0));
            this.graph.connectAudioOutput(osc.output(0));

            console.log(`[WasmGraphHandler] Graph initialized with ${this.graph.nodeCount()} nodes and ${this.graph.numAudioOutputs()} audio outputs.`);
        }
    }

    async start() {
        this.ctx = new AudioContext({ sampleRate: this.sampleRate });
        this.graph.allocate(this.sampleRate, this.frames);
        this.memory = this.raug.getMemory();
        console.log(`[WasmGraphHandler] AudioContext started at ${this.sampleRate} Hz.`);

        await this.ctx.audioWorklet.addModule('raug-worker.js');
        this.node = new AudioWorkletNode(this.ctx, 'raug-worker', {
            processorOptions: {
                frames: this.frames,
                channels: this.channels,
                audioBuffer: this.shared,
                controlBuffer: this.meta,
                blockSamples: this.blockSamples,
                queueSamples: this.queueSamples,
            },
            numberOfInputs: 0,
            numberOfOutputs: 1,
            outputChannelCount: [this.channels],
        });

        this.node.port.onmessage = (event) => {
            if (event.data.type === 'need') {
                const minSamples = event.data.samples ?? this.blockSamples;
                const target = minSamples * 8;
                while (Atomics.load(this.flags, 2) < target) {
                    this.enqueueBlock();
                }
            } else if (event.data.type === 'log') {
                console.log('[WasmGraphHandler]', event.data.msg);
            } else if (event.data.type === 'error') {
                console.error('[WasmGraphHandler]', event.data.msg);
            }
        };


        this.node.connect(this.ctx.destination);

        this.node.port.postMessage({ type: 'need' });
    }

    async stop() {
        if (this.ctx) {
            await this.ctx.close();
            this.ctx = null;
            this.node = null;
            console.log('[WasmGraphHandler] AudioContext stopped.');
        }
    }

    enqueueBlock() {
        try {
            this.graph.process();
        }
        catch (e) {
            console.error('[WasmGraphHandler] Error during graph processing:', e);
            return;
        }

        const ptr = this.graph.outputBufferPtr();
        const wasmView = new Float32Array(this.memory.buffer, ptr, this.blockSamples);
        const writeIdx = Atomics.load(this.flags, 0);

        for (let i = 0; i < this.blockSamples; i++) {
            this.samples[(writeIdx + i) % this.queueSamples] = wasmView[i];
        }

        Atomics.store(this.flags, 0, (writeIdx + this.blockSamples) % this.queueSamples);
        Atomics.add(this.flags, 2, this.blockSamples);
        Atomics.add(this.flags, 3, 1); // increment version
    }
}