import { Log } from "./log";
import { graphHandler } from "./graph-handler";
import Editor, { populateNodes } from "./editor";
import { createRoot } from "react-dom/client";
import { ThemeProvider } from "./theme-provider";

export default function RaugWasmDemoApp() {
    return (
        <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
            <div style={{ padding: "20px", fontFamily: "Arial, sans-serif" }}>
                <h1>raug-wasm Demo</h1>
                <Editor />
                <h2>Log</h2>
                <Log />
            </div>
        </ThemeProvider>
    );
}

window.addEventListener("load", async () => {
    try {
        await graphHandler.init();
        populateNodes();

        const container = document.getElementById("app");
        if (container) {
            const root = createRoot(container);
            root.render(<RaugWasmDemoApp />);
        } else {
            throw new Error("App container not found");
        }
    } catch (e) {
        console.error("Error during initialization:", e);
    }
});
