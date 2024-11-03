local daprs = require("daprs")

local graph = daprs.graph_builder()

local out1 = graph:output()
local out2 = graph:output()

local sine = graph:sine_osc()

sine:input_named("frequency"):set(440.0)

sine = sine * graph:constant(0.2)

sine:output(0):connect(out1:input(0))
sine:output(0):connect(out2:input(0))

local runtime = graph:build_runtime()

runtime:run_for(1.0)
