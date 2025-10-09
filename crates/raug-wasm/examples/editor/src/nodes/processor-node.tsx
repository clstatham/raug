import {
    BaseNode,
    BaseNodeContent,
    BaseNodeFooter,
    BaseNodeHeader,
    BaseNodeHeaderTitle,
} from "@/components/base-node";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuLabel,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { EllipsisVertical } from "lucide-react";
import { LabeledHandle } from "@/components/labeled-handle";
import { Position, useKeyPress, useNodes, useReactFlow } from "@xyflow/react";
import { useEffect } from "react";

export default function ProcessorNode(props: any) {
    const { data } = props;
    const { name, inputNames, outputNames, content } = data;

    const reactFlow = useReactFlow();
    const deleteKey = useKeyPress("Delete");

    const nodes = useNodes();

    const selected = nodes.find((n) => n.id === props.id)?.selected;

    useEffect(() => {
        if (selected && deleteKey) {
            reactFlow.deleteElements({ nodes: [{ id: props.id! }] });
        }
    }, [deleteKey, reactFlow, props]);

    return (
        <BaseNode>
            <BaseNodeHeader className="border-b">
                <BaseNodeHeaderTitle>{name}</BaseNodeHeaderTitle>
                {/* <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button
                            variant="ghost"
                            className="nodrag p-1"
                            aria-label="Node Actions"
                            title="Node Actions"
                        >
                            <EllipsisVertical className="size-4" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuLabel>Node Options</DropdownMenuLabel>
                    </DropdownMenuContent>
                </DropdownMenu> */}
            </BaseNodeHeader>
            {content && (
                <BaseNodeContent className="p-2">{content}</BaseNodeContent>
            )}
            <BaseNodeFooter className="items-stretch px-0 py-1">
                <div className="flex justify-between">
                    <div className="flex flex-col">
                        {inputNames.map(
                            (inputName: string, _inputIndex: number) => (
                                <LabeledHandle
                                    title={inputName}
                                    type="target"
                                    key={inputName}
                                    position={Position.Left}
                                    id={`${_inputIndex}`}
                                />
                            )
                        )}
                    </div>
                    <div className="flex flex-col">
                        {outputNames.map(
                            (outputName: string, _outputIndex: number) => (
                                <LabeledHandle
                                    title={outputName}
                                    type="source"
                                    key={outputName}
                                    position={Position.Right}
                                    id={`${_outputIndex}`}
                                />
                            )
                        )}
                    </div>
                </div>
            </BaseNodeFooter>
        </BaseNode>
    );
}
