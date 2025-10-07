import { Handle, Position } from "@xyflow/react";

export function nodeInputsJsx(inputNames: string[]) {
    return inputNames.map((inputName: string, inputIndex: number) => (
        <div
            key={inputName}
            style={{
                position: "relative",
                marginBottom: "20px",
                textAlign: "left",
                display: "flex",
                alignItems: "center",
                justifyContent: "flex-start",
            }}
        >
            <Handle
                type="target"
                position={Position.Left}
                id={inputIndex.toString()}
            />
            <span style={{ marginLeft: "15px", fontSize: "12px" }}>
                {inputName}
            </span>
        </div>
    ));
}

export function nodeOutputsJsx(outputNames: string[]) {
    return outputNames.map((outputName: string, outputIndex: number) => (
        <div
            key={outputName}
            style={{
                position: "relative",
                marginBottom: "20px",
                textAlign: "right",
                display: "flex",
                alignItems: "center",
                justifyContent: "flex-end",
            }}
        >
            <span
                style={{
                    marginRight: "15px",
                    fontSize: "12px",
                }}
            >
                {outputName}
            </span>
            <Handle
                type="source"
                position={Position.Right}
                id={outputIndex.toString()}
            />
        </div>
    ));
}

export default function ProcessorNode(props: any) {
    const { data } = props;
    const { name, inputNames, outputNames } = data;
    return (
        <div className={"processor-node-" + name.toLowerCase()}>
            <div style={{ textAlign: "center", fontWeight: "bold" }}>
                <strong>{name}</strong>
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
