import {
    Handle,
    Position,
    useKeyPress,
    useNodes,
    useReactFlow,
} from "@xyflow/react";
import { useEffect } from "react";

export function nodeInputsJsx(inputNames: string[]) {
    return inputNames.map((inputName: string, inputIndex: number) => (
        <div key={inputName} className="node-port node-port--input">
            <Handle
                type="target"
                position={Position.Left}
                id={inputIndex.toString()}
                className="node-port__handle node-port__handle--input"
            />
            <span className="node-port__label node-port__label--input">
                {inputName}
            </span>
        </div>
    ));
}

export function nodeOutputsJsx(outputNames: string[]) {
    return outputNames.map((outputName: string, outputIndex: number) => (
        <div key={outputName} className="node-port node-port--output">
            <span className="node-port__label node-port__label--output">
                {outputName}
            </span>
            <Handle
                type="source"
                position={Position.Right}
                id={outputIndex.toString()}
                className="node-port__handle node-port__handle--output"
            />
        </div>
    ));
}

export const useNodeDeletionEffect = (node: any) => {
    const reactFlow = useReactFlow();
    const deleteKey = useKeyPress("Delete");

    const nodes = useNodes();

    const selected = nodes.find((n) => n.id === node.id)?.selected;

    useEffect(() => {
        if (selected && deleteKey) {
            reactFlow.deleteElements({ nodes: [{ id: node.id! }] });
        }
    }, [deleteKey, reactFlow, node]);
};

export default function ProcessorNode(props: any) {
    const { data } = props;
    const { name, inputNames, outputNames } = data;
    useNodeDeletionEffect(props);
    return (
        <div className="processor-node">
            <div className="node-title">
                <strong className="node-title__text">{name}</strong>
            </div>
            <div className="node-ports">
                <div>{nodeInputsJsx(inputNames)}</div>
                <div>{nodeOutputsJsx(outputNames)}</div>
            </div>
        </div>
    );
}
