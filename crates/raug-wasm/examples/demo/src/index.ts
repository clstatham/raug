import { errorMessage } from './log';
import GraphHandler from './graph-handler';
import Editor from './editor';

class Main {
    private readonly graphHandler: GraphHandler;

    constructor() {
        this.graphHandler = new GraphHandler(8, 128, 2);
    }

    async init(): Promise<void> {
        await this.graphHandler.init();
    }

    async start(): Promise<void> {
        await this.graphHandler.start();
    }

    async stop(): Promise<void> {
        await this.graphHandler.stop();
    }
}

async function run(): Promise<void> {
    const main = new Main();
    await main.init();

    const startBtn = document.getElementById('start');
    if (startBtn instanceof HTMLButtonElement) {
        startBtn.addEventListener('click', () => {
            main.start().catch((error) => errorMessage('Failed to start audio graph', error));
        });
    }

    const stopBtn = document.getElementById('stop');
    if (stopBtn instanceof HTMLButtonElement) {
        stopBtn.addEventListener('click', () => {
            main.stop().catch((error) => errorMessage('Failed to stop audio graph', error));
        });
    }

    const editorContainer = document.getElementById('editor');
    if (editorContainer instanceof HTMLElement) {
        const editor = new Editor(editorContainer);
        editor.cy.add({
            group: 'nodes',
            data: { id: 'a', label: 'Node A' },
            position: { x: 100, y: 100 },
        });
    }
}

run().catch((error) => errorMessage('Application failed to launch', error));
