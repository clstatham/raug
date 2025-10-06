const formatArgs = (args: unknown[]): string => {
    return args
        .map((arg) => {
            if (typeof arg === 'string') {
                return arg;
            }

            try {
                return JSON.stringify(arg);
            } catch (_error) {
                return String(arg);
            }
        })
        .join(' ');
};

const appendLogLine = (text: string, color?: string): void => {
    const logElement = document.getElementById('log');
    if (!logElement) {
        return;
    }

    const logLine = document.createElement('div');
    if (color) {
        logLine.style.color = color;
    }
    logLine.textContent = text;
    logElement.appendChild(logLine);
    logElement.scrollTop = logElement.scrollHeight;
};

export function logMessage(message: string, ...args: unknown[]): void {
    const formatted = args.length > 0 ? `${message} ${formatArgs(args)}` : message;
    appendLogLine(formatted);
    console.log(message, ...args);
}

export function errorMessage(message: string, ...args: unknown[]): void {
    const formatted = args.length > 0 ? `${message} ${formatArgs(args)}` : message;
    appendLogLine(formatted, 'red');
    console.error(message, ...args);
}
