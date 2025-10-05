class RaugWorker extends AudioWorkletProcessor {
    constructor({ processorOptions }) {
        super();
        this.channels = processorOptions.channels;
        this.blockSamples = processorOptions.blockSamples;
        this.queueSamples = processorOptions.queueSamples;
        this.samples = new Float32Array(processorOptions.audioBuffer);
        this.flags = new Int32Array(processorOptions.controlBuffer);
        this.version = Atomics.load(this.flags, 3);
    }

    process(_, outputs) {
        const out = outputs[0];
        const frames = out[0].length;
        const samplesNeeded = frames * this.channels;
        const available = Atomics.load(this.flags, 2);

        if (available < samplesNeeded) {
            this.port.postMessage({ type: 'log', msg: `Underrun: need ${samplesNeeded}, have ${available}` });
            this.port.postMessage({ type: 'need', samples: samplesNeeded });
            for (let ch = 0; ch < out.length; ch++) out[ch].fill(0);
            return true;
        }

        const readIdx = Atomics.load(this.flags, 1);

        for (let frame = 0; frame < frames; frame++) {
            for (let ch = 0; ch < this.channels; ch++) {
                const idx = (readIdx + frame * this.channels + ch) % this.queueSamples;
                out[ch][frame] = this.samples[idx];
            }
        }

        const consumed = samplesNeeded;
        Atomics.store(this.flags, 1, (readIdx + consumed) % this.queueSamples);
        Atomics.sub(this.flags, 2, consumed);

        const newVersion = Atomics.load(this.flags, 3);
        if (newVersion !== this.version) {
            this.version = newVersion;
            this.port.postMessage({ type: 'need', samples: samplesNeeded });
        }

        return true;
    }
}

registerProcessor('raug-worker', RaugWorker);