import { nodeInputsJsx, useNodeDeletionEffect } from "./processor-node";

export default function DacNode(props: any) {
    const { data } = props;
    const { index } = data;

    useNodeDeletionEffect(props);

    return (
        <div className={"dac-node"}>
            <div className="node-title">
                <strong className="node-title__text">Output {index}</strong>
            </div>
            <div className="node-ports">
                <div>{nodeInputsJsx([""])}</div>
            </div>
        </div>
    );
}
