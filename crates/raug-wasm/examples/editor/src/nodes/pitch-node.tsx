export interface PitchNodeProps {
    data: {
        index: number;
        get: () => number;
        set: (value: number) => void;
    };
}

import { useState } from "react";
import ProcessorNode from "./processor-node";
import { Slider } from "@radix-ui/react-slider";
import { NumericScrubber } from "@/components/ui/number-scrubber";

export default function PitchNode(props: PitchNodeProps) {
    const { data } = props;
    const { set: setExternal, get: getExternal } = data;

    const [localPitch, setLocalPitch] = useState(getExternal());

    return (
        <ProcessorNode
            {...props}
            data={{
                ...data,
                name: `Pitch`,
                inputNames: [],
                outputNames: ["pitch"],
                content: (
                    <NumericScrubber
                        className="nodrag"
                        value={getExternal()}
                        onChange={(v) => {
                            setLocalPitch(v);
                            setExternal(v);
                        }}
                        min={0}
                        max={127}
                        step={1}
                    />
                ),
            }}
        />
    );
}
