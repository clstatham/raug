use raug::{graph::node::AsNodeOutput, prelude::*};

use crate::prelude::*;

// pub trait OutputExt {
//     fn channel<T: Signal + Default + Clone>(&self) -> OutputChannel<T>;

//     fn powf(&self, b: impl IntoOutputExt) -> Node;
//     fn sqrt(&self) -> Node;
//     fn sin(&self) -> Node;
//     fn cos(&self) -> Node;
//     fn tan(&self) -> Node;
//     fn asin(&self) -> Node;
//     fn acos(&self) -> Node;
//     fn atan(&self) -> Node;
//     fn sinh(&self) -> Node;
//     fn cosh(&self) -> Node;
//     fn tanh(&self) -> Node;
//     fn atan2(&self, b: impl IntoOutputExt) -> Node;
//     fn hypot(&self, b: impl IntoOutputExt) -> Node;
//     fn abs(&self) -> Node;
//     fn ceil(&self) -> Node;
//     fn floor(&self) -> Node;
//     fn round(&self) -> Node;
//     fn trunc(&self) -> Node;
//     fn fract(&self) -> Node;
//     fn recip(&self) -> Node;
//     fn signum(&self) -> Node;
//     fn max(&self, b: impl IntoOutputExt) -> Node;
//     fn min(&self, b: impl IntoOutputExt) -> Node;
//     fn clamp(&self, min: impl IntoOutputExt, max: impl IntoOutputExt) -> Node;

//     fn some(&self) -> Node;
//     fn unwrap_or(&self, b: impl IntoOutputExt) -> Node;

//     fn lt<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
//     fn gt<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
//     fn le<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
//     fn ge<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
//     fn eq<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
//     fn ne<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;

//     fn toggle(&self) -> Node;
//     fn trig_to_gate(&self, length: impl IntoOutputExt) -> Node;
//     fn smooth(&self, factor: impl IntoOutputExt) -> Node;
//     fn scale(&self, min: impl IntoOutputExt, max: impl IntoOutputExt) -> Node;
// }

pub trait GraphExt {
    fn powf<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy;

    fn sqrt<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn sin<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn cos<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn tan<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn asin<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn acos<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn atan<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn sinh<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn cosh<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn tanh<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn atan2<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy;

    fn hypot<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy;

    fn abs<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn ceil<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn floor<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn round<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn trunc<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn fract<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn recip<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn signum<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy;

    fn max<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy;

    fn min<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy;

    fn clamp<A, AO, B, BO, C, CO>(&mut self, a: A, min: B, max: C) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        CO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
        C: AsNodeOutput<CO> + Copy;
}

impl GraphExt for Graph {
    fn powf<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
    {
        let node = self.node(Powf::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn sqrt<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Sqrt::default());
        self.connect(a, node.input(0));
        node
    }

    fn sin<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Sin::default());
        self.connect(a, node.input(0));
        node
    }

    fn cos<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Cos::default());
        self.connect(a, node.input(0));
        node
    }

    fn tan<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Tan::default());
        self.connect(a, node.input(0));
        node
    }

    fn asin<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Asin::default());
        self.connect(a, node.input(0));
        node
    }

    fn acos<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Acos::default());
        self.connect(a, node.input(0));
        node
    }

    fn atan<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Atan::default());
        self.connect(a, node.input(0));
        node
    }

    fn sinh<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Sinh::default());
        self.connect(a, node.input(0));
        node
    }

    fn cosh<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Cosh::default());
        self.connect(a, node.input(0));
        node
    }

    fn tanh<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Tanh::default());
        self.connect(a, node.input(0));
        node
    }

    fn atan2<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
    {
        let node = self.node(Atan2::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn hypot<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
    {
        let node = self.node(Hypot::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn abs<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Abs::default());
        self.connect(a, node.input(0));
        node
    }

    fn ceil<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Ceil::default());
        self.connect(a, node.input(0));
        node
    }

    fn floor<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Floor::default());
        self.connect(a, node.input(0));
        node
    }

    fn round<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Round::default());
        self.connect(a, node.input(0));
        node
    }

    fn trunc<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Trunc::default());
        self.connect(a, node.input(0));
        node
    }

    fn fract<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Fract::default());
        self.connect(a, node.input(0));
        node
    }

    fn recip<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Recip::default());
        self.connect(a, node.input(0));
        node
    }

    fn signum<A, AO>(&mut self, a: A) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
    {
        let node = self.node(Signum::default());
        self.connect(a, node.input(0));
        node
    }

    fn max<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
    {
        let node = self.node(Max::<f32>::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn min<A, AO, B, BO>(&mut self, a: A, b: B) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
    {
        let node = self.node(Min::<f32>::default());
        self.connect(a, node.input(0));
        self.connect(b, node.input(1));
        node
    }

    fn clamp<A, AO, B, BO, C, CO>(&mut self, a: A, min: B, max: C) -> NodeIndex
    where
        AO: AsNodeOutputIndex<ProcessorNode>,
        BO: AsNodeOutputIndex<ProcessorNode>,
        CO: AsNodeOutputIndex<ProcessorNode>,
        A: AsNodeOutput<AO> + Copy,
        B: AsNodeOutput<BO> + Copy,
        C: AsNodeOutput<CO> + Copy,
    {
        let node = self.node(Clamp::<f32>::default());
        self.connect(a, node.input(0));
        self.connect(min, node.input(1));
        self.connect(max, node.input(2));
        node
    }
}
