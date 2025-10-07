import { useState, useRef, useCallback, useEffect } from "react";

type DragLabelProps = {
    setValue: (v: number) => void;
    getValue: () => number;
    speed: number;
};

export default function DragLabel({
    setValue,
    getValue,
    speed = 0.01,
}: DragLabelProps) {
    const [snapshot, setSnapshot] = useState(getValue());
    const [isDragging, setIsDragging] = useState(false);
    const startXRef = useRef(0);
    const baseValueRef = useRef(snapshot);

    const onStart = useCallback(
        (e: React.MouseEvent<HTMLSpanElement>) => {
            e.preventDefault();
            e.stopPropagation();
            const value = getValue();
            baseValueRef.current = value;
            startXRef.current = e.clientX;
            setSnapshot(value);
            setIsDragging(true);
        },
        [getValue]
    );

    useEffect(() => {
        if (!isDragging) {
            setSnapshot(getValue());
        }

        function onMove(e: MouseEvent) {
            if (isDragging) {
                const delta = e.clientX - startXRef.current;
                const newValue = baseValueRef.current + delta * speed;
                setSnapshot(newValue);
                setValue(newValue);
            }
        }

        function onEnd() {
            setIsDragging(false);
        }

        window.addEventListener("mousemove", onMove);
        window.addEventListener("mouseup", onEnd);

        return () => {
            window.removeEventListener("mousemove", onMove);
            window.removeEventListener("mouseup", onEnd);
        };
    }, [isDragging, setValue, getValue, speed]);

    return (
        <span
            style={{ cursor: "ns-resize", userSelect: "none" }}
            onMouseDown={onStart}
            className="nodrag"
        >
            {snapshot.toFixed(2)}
        </span>
    );
}
