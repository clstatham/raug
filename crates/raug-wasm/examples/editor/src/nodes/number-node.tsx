import { Slider } from "@/components/ui/slider";
import ProcessorNode from "./processor-node";
import { useMemo, useState } from "react";
import { NumericScrubber } from "@/components/ui/number-scrubber";

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
    // default max to twice initial value, so that initial value is in middle of slider
    const [max, setMax] = useState(externalValue * 2.0);

    useMemo(() => {
        // if external value changes, or if the min/max change, update local value accordingly
        const clamped = Math.min(max, Math.max(min, externalValue));
        const ratio = (clamped - min) / (max - min);
        setLocalValue(ratio);
    }, [externalValue, min, max]);

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
                    <div className="nodrag">
                        <div className="flex gap-2 mb-2">
                            <div className="flex flex-col">
                                <label className="text-xs text-foreground">
                                    Min
                                </label>
                                <NumericScrubber
                                    className="w-16 px-2 py-1 text-xs border rounded"
                                    value={min}
                                    step={0.01}
                                    onChange={(v) => setMin(v)}
                                />
                            </div>
                            <div className="flex flex-col">
                                <label className="text-xs text-foreground">
                                    Max
                                </label>
                                <NumericScrubber
                                    className="w-16 px-2 py-1 text-xs border rounded"
                                    value={max}
                                    step={0.01}
                                    onChange={(v) => setMax(v)}
                                />
                            </div>
                        </div>
                        <div
                            className="w-full"
                            onPointerDown={(event) => event.stopPropagation()}
                            onPointerMove={(event) => event.stopPropagation()}
                            onClick={(event) => event.stopPropagation()}
                        >
                            <label className="text-xs text-foreground">
                                {"Value: "}
                                {getExternal().toFixed(3)}
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
