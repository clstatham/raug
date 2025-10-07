use raug::{
    graph::node::{IntoNodeOutput, RaugNodeIndexExt},
    prelude::*,
};

use crate::prelude::*;

pub trait GraphExt {
    fn smooth<A, AO, B, BO>(&mut self, a: A, factor: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn powf<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn sqrt<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn sin<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn cos<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn tan<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn asin<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn acos<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn atan<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn sinh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn cosh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn tanh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn atan2<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn hypot<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn abs<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn ceil<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn floor<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn round<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn trunc<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn fract<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn recip<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn signum<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>;

    fn max<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn min<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>;

    fn clamp<A, AO, B, BO, C, CO>(&mut self, a: A, min: B, max: C) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        CO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
        C: IntoNodeOutput<CO>;
}

impl GraphExt for Graph {
    fn smooth<A, AO, B, BO>(&mut self, a: A, factor: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        let node = self.node(Smooth::default());
        self.connect(a, node.input(0));
        self.connect(factor, node.input(1));
        node
    }

    fn powf<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        let node = self.node(Powf::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn sqrt<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Sqrt::default());
        self.connect(a, node.input(0));
        node
    }

    fn sin<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Sin::default());
        self.connect(a, node.input(0));
        node
    }

    fn cos<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Cos::default());
        self.connect(a, node.input(0));
        node
    }

    fn tan<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Tan::default());
        self.connect(a, node.input(0));
        node
    }

    fn asin<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Asin::default());
        self.connect(a, node.input(0));
        node
    }

    fn acos<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Acos::default());
        self.connect(a, node.input(0));
        node
    }

    fn atan<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Atan::default());
        self.connect(a, node.input(0));
        node
    }

    fn sinh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Sinh::default());
        self.connect(a, node.input(0));
        node
    }

    fn cosh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Cosh::default());
        self.connect(a, node.input(0));
        node
    }

    fn tanh<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Tanh::default());
        self.connect(a, node.input(0));
        node
    }

    fn atan2<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        let node = self.node(Atan2::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn hypot<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        let node = self.node(Hypot::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn abs<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Abs::default());
        self.connect(a, node.input(0));
        node
    }

    fn ceil<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Ceil::default());
        self.connect(a, node.input(0));
        node
    }

    fn floor<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Floor::default());
        self.connect(a, node.input(0));
        node
    }

    fn round<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Round::default());
        self.connect(a, node.input(0));
        node
    }

    fn trunc<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Trunc::default());
        self.connect(a, node.input(0));
        node
    }

    fn fract<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Fract::default());
        self.connect(a, node.input(0));
        node
    }

    fn recip<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Recip::default());
        self.connect(a, node.input(0));
        node
    }

    fn signum<A, AO>(&mut self, a: A) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
    {
        let node = self.node(Signum::default());
        self.connect(a, node.input(0));
        node
    }

    fn max<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        let node = self.node(Max::<f32>::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn min<A, AO, B, BO>(&mut self, a: A, b: B) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
    {
        let node = self.node(Min::<f32>::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn clamp<A, AO, B, BO, C, CO>(&mut self, a: A, min: B, max: C) -> Node
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        CO: AsNodeOutputIndex<ProcessorNode>,
        A: IntoNodeOutput<AO>,
        B: IntoNodeOutput<BO>,
        C: IntoNodeOutput<CO>,
    {
        let node = self.node(Clamp::<f32>::default());
        self.connect(a, node.input(0));
        self.connect(min, node.input(1));
        self.connect(max, node.input(2));
        node
    }
}
