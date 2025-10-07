import { nodeInputsJsx, nodeOutputsJsx } from "./processor-node";
import DragLabel from "./utils/drag-number";

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
