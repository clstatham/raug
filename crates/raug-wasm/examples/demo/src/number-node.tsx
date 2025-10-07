import { useCallback, useEffect, useRef, useState } from "react";
import { nodeInputsJsx, nodeOutputsJsx } from "./processor-node";

type NumberNodeProps = {
    data: {
        name: string;
        inputNames: string[];
        outputNames: string[];
        setValue: (v: number) => void;
        getValue: () => number;
    };
};

export default function NumberNode(props: NumberNodeProps) {
    const { name, inputNames, outputNames, setValue, getValue } = props.data;
    return (
        <div className={"number-node-" + name.toLowerCase()}>
            <div style={{ textAlign: "center", fontWeight: "bold" }}>
                <strong>Number</strong>
            </div>
            <div
                id="number-node-value"
                style={{ fontSize: "24px", textAlign: "center" }}
            >
                <DragLabel setValue={setValue} getValue={getValue} />
            </div>
            <div
                style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "flex-start",
                }}
            >
                <div>{nodeInputsJsx(inputNames)}</div>
                <div>{nodeOutputsJsx(outputNames)}</div>
            </div>
        </div>
    );
}

type DragLabelProps = {
    setValue: (v: number) => void;
    getValue: () => number;
};

function DragLabel({ setValue, getValue }: DragLabelProps) {
    const [snapshot, setSnapshot] = useState<number>(getValue());
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
                const newValue = baseValueRef.current + delta;
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
    }, [isDragging, setValue]);

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
