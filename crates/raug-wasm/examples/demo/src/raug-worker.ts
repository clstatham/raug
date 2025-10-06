interface RaugWorkletOptions {
    channels: number;
    blockSamples: number;
    queueSamples: number;
    audioBuffer: SharedArrayBuffer;
    controlBuffer: SharedArrayBuffer;
}

type WorkletMessage =
    | { type: 'error'; msg: string }
    | { type: 'log'; msg: string }
    | { type: 'need'; samples: number };

class RaugWorker extends AudioWorkletProcessor {
    private readonly channels: number;
    private readonly blockSamples: number;
    private readonly queueSamples: number;
    private readonly samples: Float32Array;
    private readonly flags: Int32Array;
    private version: number;

    constructor(options: AudioWorkletNodeOptions) {
        super();
        const processorOptions = options.processorOptions as RaugWorkletOptions;
        this.channels = processorOptions.channels;
        this.blockSamples = processorOptions.blockSamples;
        this.queueSamples = processorOptions.queueSamples;
        this.samples = new Float32Array(processorOptions.audioBuffer);
        this.flags = new Int32Array(processorOptions.controlBuffer);
        
        this.version = Atomics.load(this.flags, 3);
    }

    private postMessage(message: WorkletMessage): void {
        this.port.postMessage(message);
    }

    private postError(message: string): void {
        this.postMessage({ type: 'error', msg: message });
    }

    private postLog(message: string): void {
        this.postMessage({ type: 'log', msg: message });
    }

    private requestSamples(samples: number): void {
        this.postMessage({ type: 'need', samples });
    }

    process(
        _inputs: Float32Array[][],
        outputs: Float32Array[][],
        _parameters: Record<string, Float32Array>
    ): boolean {
        if (outputs.length === 0 || outputs[0].length === 0) {
            return true;
        }

        const out = outputs[0];
        const frames = out[0].length;
        const samplesNeeded = frames * this.channels;
        const available = Atomics.load(this.flags, 2);

        if (available < samplesNeeded) {
            this.postError(`Underrun: need ${samplesNeeded}, have ${available}`);
            this.requestSamples(samplesNeeded);
            out.forEach((channel) => channel.fill(0));
            return true;
        }

        const readIdx = Atomics.load(this.flags, 1);

        for (let frame = 0; frame < frames; frame += 1) {
            for (let ch = 0; ch < this.channels; ch += 1) {
                const idx = (readIdx + frame * this.channels + ch) % this.queueSamples;
                out[ch][frame] = this.samples[idx];
            }
        }

        Atomics.store(this.flags, 1, (readIdx + samplesNeeded) % this.queueSamples);
        Atomics.sub(this.flags, 2, samplesNeeded);

        const newVersion = Atomics.load(this.flags, 3);
        if (newVersion !== this.version) {
            this.version = newVersion;
            this.requestSamples(samplesNeeded);
        }

        return true;
    }
}

registerProcessor('raug-worker', RaugWorker);