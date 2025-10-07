import {
    SimpleBezierEdge,
    SimpleBezierEdgeProps,
    useEdges,
    useKeyPress,
    useReactFlow,
} from "@xyflow/react";
import { useEffect } from "react";

const useEdgeDeletionEffect = (edge: any) => {
    const reactFlow = useReactFlow();
    const deleteKey = useKeyPress("Delete");

    const edges = useEdges();

    const selected = edges.find((e) => e.id === edge.id)?.selected;

    useEffect(() => {
        if (selected && deleteKey) {
            reactFlow.deleteElements({ edges: [{ id: edge.id! }] });
        }
    }, [deleteKey, reactFlow, edge]);
};

export default function CustomEdge(props: SimpleBezierEdgeProps) {
    useEdgeDeletionEffect(props);

    return <SimpleBezierEdge {...props} />;
}
