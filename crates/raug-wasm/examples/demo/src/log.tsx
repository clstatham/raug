import { createWithEqualityFn } from "zustand/traditional";
import { nanoid } from "nanoid";

const useLogStore = createWithEqualityFn<{
    entries: LogEntry[];
    appendLogEntry: (entry: LogEntry) => void;
}>((set, get) => ({
    entries: [] as LogEntry[],
    appendLogEntry: (entry: LogEntry) => {
        set({
            entries: [...get().entries, entry].slice(-100), // Keep only last 100 entries
        });
    },
}));

interface LogEntry {
    message: string;
    level: "info" | "error" | "warn";
    timestamp: Date;
}

function formatArgs(message: string, args: any[]): string {
    const format = args
        .map((arg) => {
            if (typeof arg === "string") {
                return arg;
            } else if (typeof arg === "object") {
                try {
                    return JSON.stringify(arg);
                } catch {
                    return String(arg);
                }
            } else {
                return String(arg);
            }
        })
        .join(" ");

    return message + (format ? " " + format : "");
}

export function logMessage(message: string, ...args: any[]): void {
    const formatted = formatArgs(message, args);
    console.log(formatted);
    useLogStore
        .getState()
        .appendLogEntry({
            message: formatted,
            level: "info",
            timestamp: new Date(),
        });
}

export function errorMessage(message: string, ...args: any[]): void {
    const formatted = formatArgs(message, args);
    console.error(formatted);
    useLogStore
        .getState()
        .appendLogEntry({
            message: formatted,
            level: "error",
            timestamp: new Date(),
        });
}

export function Log() {
    const entries = useLogStore((state) => state.entries);
    return (
        <div
            style={{
                maxHeight: 200,
                overflowY: "auto",
                backgroundColor: "#f0f0f0",
                padding: "10px",
                border: "1px solid #ccc",
            }}
        >
            {entries.map((entry) => (
                <div
                    key={nanoid()}
                    style={{
                        color:
                            entry.level === "error"
                                ? "red"
                                : entry.level === "warn"
                                ? "orange"
                                : "black",
                        fontFamily: "monospace",
                        marginBottom: "4px",
                    }}
                >
                    [{entry.timestamp.toLocaleTimeString()}] {entry.message}
                </div>
            ))}
        </div>
    );
}
