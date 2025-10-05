import WasmGraphHandler from './wasm-graph-handler.js';

class Main {
    constructor() {
        this.graphHandler = new WasmGraphHandler(8, 128, 2);
    }

    async init() {
        await this.graphHandler.init();
    }

    async start() {
        await this.graphHandler.start();
    }

    async stop() {
        await this.graphHandler.stop();
    }
}

async function run() {
    const main = new Main();
    await main.init();

    // Create buttons if they don't exist
    let startBtn = document.getElementById('start');
    if (!startBtn) {
        startBtn = document.createElement('button');
        startBtn.id = 'start';
        startBtn.textContent = 'Start';
        document.body.appendChild(startBtn);
    }

    let stopBtn = document.getElementById('stop');
    if (!stopBtn) {
        stopBtn = document.createElement('button');
        stopBtn.id = 'stop';
        stopBtn.textContent = 'Stop';
        document.body.appendChild(stopBtn);
    }

    startBtn.onclick = async () => await main.start();
    stopBtn.onclick = async () => await main.stop();
}

run().catch(console.error);
