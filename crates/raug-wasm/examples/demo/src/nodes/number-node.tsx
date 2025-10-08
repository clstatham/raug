import { Slider } from "@/components/ui/slider";
import ProcessorNode from "./processor-node";
import { useEffect, useState } from "react";

type NumberNodeProps = {
    data: {
        set: (v: number) => void;
        get: () => number;
    };
};

export default function NumberNode(props: NumberNodeProps) {
    const { set: setExternal, get: getExternal } = props.data;

    const externalValue = getExternal();
    const [localValue, setLocalValue] = useState(0.5); // ratio min:max
    const [min, setMin] = useState(0);
    const [max, setMax] = useState(externalValue * 2.0); // default max to twice initial value

    return (
        <ProcessorNode
            {...props}
            data={{
                ...props.data,
                outerWidth: 400,
                name: "Number",
                inputNames: [],
                outputNames: ["value"],
                content: (
                    <div>
                        <div className="flex gap-2 mb-2">
                            <div className="flex flex-col">
                                <label className="text-xs text-gray-600">
                                    Min
                                </label>
                                <input
                                    type="number"
                                    value={min}
                                    onChange={(e) =>
                                        setMin(Number(e.target.value))
                                    }
                                    className="w-16 px-2 py-1 text-xs border rounded"
                                />
                            </div>
                            <div className="flex flex-col">
                                <label className="text-xs text-gray-600">
                                    Max
                                </label>
                                <input
                                    type="number"
                                    value={max}
                                    onChange={(e) =>
                                        setMax(Number(e.target.value))
                                    }
                                    className="w-16 px-2 py-1 text-xs border rounded"
                                />
                            </div>
                        </div>
                        <div
                            className="nodrag w-full"
                            onPointerDown={(event) => event.stopPropagation()}
                            onPointerMove={(event) => event.stopPropagation()}
                            onClick={(event) => event.stopPropagation()}
                        >
                            <label className="text-xs text-gray-600">
                                {"Value: "}
                                {(min + localValue * (max - min)).toFixed(3)}
                            </label>
                            <Slider
                                className="w-full"
                                min={0.0}
                                max={1.0}
                                step={0.001}
                                value={[localValue]}
                                onValueChange={(v) => {
                                    const next = v[0];
                                    setLocalValue(next);
                                    let value = min + next * (max - min);
                                    setExternal(value);
                                }}
                            />
                        </div>
                    </div>
                ),
            }}
        />
    );
}
