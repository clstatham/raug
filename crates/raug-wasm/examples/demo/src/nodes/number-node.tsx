import {
    nodeInputsJsx,
    nodeOutputsJsx,
    useNodeDeletionEffect,
} from "./processor-node";
import DragLabel from "./utils/drag-number";

type NumberNodeProps = {
    data: {
        setValue: (v: number) => void;
        getValue: () => number;
        speed: number;
    };
};

export default function NumberNode(props: NumberNodeProps) {
    const { setValue, getValue, speed } = props.data;

    useNodeDeletionEffect(props);

    return (
        <div className={"number-node"}>
            <div className="node-title">
                <strong className="node-title__text">Number</strong>
            </div>
            <div className="node-ports">
                <div>{nodeInputsJsx([])}</div>
                <div id="number-node-value" className="number-node__value">
                    <DragLabel
                        setValue={setValue}
                        getValue={getValue}
                        speed={speed}
                    />
                </div>
                <div>{nodeOutputsJsx([""])}</div>
            </div>
        </div>
    );
}
