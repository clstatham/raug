import { createWithEqualityFn } from "zustand/traditional";
import { nanoid } from "nanoid";
import { Label } from "./components/ui/label";

const useLogStore = createWithEqualityFn((set, get): any => ({
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
    useLogStore.getState().appendLogEntry({
        message: formatted,
        level: "info",
        timestamp: new Date(),
    });
}

export function errorMessage(message: string, ...args: any[]): void {
    const formatted = formatArgs(message, args);
    console.error(formatted);
    useLogStore.getState().appendLogEntry({
        message: formatted,
        level: "error",
        timestamp: new Date(),
    });
}

export function Log() {
    const entries = useLogStore((state) => state.entries);
    return (
        <div className="border rounded p-4 h-64 overflow-y-auto">
            {entries.map((entry: LogEntry) => (
                <div
                    key={nanoid()}
                    className={`mb-2 ${
                        entry.level === "error" ? "text-red-600" : ""
                    }`}
                >
                    <Label className="text-xs font-mono">
                        [{entry.timestamp.toLocaleTimeString()}]&nbsp;
                        {entry.message}
                    </Label>
                </div>
            ))}
        </div>
    );
}
