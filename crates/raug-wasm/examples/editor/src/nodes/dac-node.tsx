import ProcessorNode from "./processor-node";

export default function DacNode(props: any) {
    const { data } = props;
    const { index } = data;

    return (
        <ProcessorNode
            {...props}
            data={{
                ...data,
                name: `DAC ${index + 1}`,
                inputNames: ["🔊"],
                outputNames: [],
            }}
        />
    );
}
