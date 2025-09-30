use std::sync::{Arc, Mutex};

use raug::prelude::*;

use crate::prelude::*;

pub struct InputChannel<T: Signal + Clone + Default> {
    pub(crate) tx: Arc<Mutex<Tx<T>>>,
}

pub struct OutputChannel<T: Signal + Clone + Default> {
    pub(crate) rx: Arc<Mutex<Rx<T>>>,
}

impl<T: Signal + Clone + Default> InputChannel<T> {
    #[inline]
    pub fn send(&self, value: T) -> Result<(), ChannelError> {
        let tx = self.tx.lock().unwrap();
        if tx.tx.try_send(value).is_err() {
            return Err(ChannelError::SendError);
        }
        Ok(())
    }
}

impl<T: Signal + Clone + Default> OutputChannel<T> {
    #[inline]
    pub fn recv(&self) -> T {
        let mut rx = self.rx.lock().unwrap();
        if let Ok(value) = rx.rx.try_recv() {
            rx.last.clone_from(&value);
        }
        rx.last.clone()
    }

    pub fn recv_all(&self) -> Vec<T> {
        let mut rx = self.rx.lock().unwrap();
        let mut values = vec![rx.last.clone()];
        while let Ok(value) = rx.rx.try_recv() {
            rx.last.clone_from(&value);
            values.push(value);
        }
        values
    }
}

pub trait InputExt {
    fn channel<T: Signal + Clone + Default>(&self) -> InputChannel<T>;
}

impl InputExt for Input {
    #[inline]
    #[track_caller]
    fn channel<T: Signal + Clone + Default>(&self) -> InputChannel<T> {
        assert_eq!(
            T::signal_type(),
            self.signal_type(),
            "Signal type must match for channel creation",
        );
        let channel = Channel::new(T::default());
        let tx = channel.tx.clone();
        let channel_node = self.graph().node(channel);
        self.connect(channel_node.output(0));
        InputChannel { tx }
    }
}

pub trait OutputExt {
    fn channel<T: Signal + Default + Clone>(&self) -> OutputChannel<T>;

    fn powf(&self, b: impl IntoOutputExt) -> Node;
    fn sqrt(&self) -> Node;
    fn sin(&self) -> Node;
    fn cos(&self) -> Node;
    fn tan(&self) -> Node;
    fn asin(&self) -> Node;
    fn acos(&self) -> Node;
    fn atan(&self) -> Node;
    fn sinh(&self) -> Node;
    fn cosh(&self) -> Node;
    fn tanh(&self) -> Node;
    fn atan2(&self, b: impl IntoOutputExt) -> Node;
    fn hypot(&self, b: impl IntoOutputExt) -> Node;
    fn abs(&self) -> Node;
    fn ceil(&self) -> Node;
    fn floor(&self) -> Node;
    fn round(&self) -> Node;
    fn trunc(&self) -> Node;
    fn fract(&self) -> Node;
    fn recip(&self) -> Node;
    fn signum(&self) -> Node;
    fn max(&self, b: impl IntoOutputExt) -> Node;
    fn min(&self, b: impl IntoOutputExt) -> Node;
    fn clamp(&self, min: impl IntoOutputExt, max: impl IntoOutputExt) -> Node;

    fn some(&self) -> Node;
    fn unwrap_or(&self, b: impl IntoOutputExt) -> Node;

    fn lt<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
    fn gt<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
    fn le<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
    fn ge<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
    fn eq<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;
    fn ne<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node;

    fn toggle(&self) -> Node;
    fn trig_to_gate(&self, length: impl IntoOutputExt) -> Node;
    fn smooth(&self, factor: impl IntoOutputExt) -> Node;
    fn scale(&self, min: impl IntoOutputExt, max: impl IntoOutputExt) -> Node;
}

macro_rules! choose_node_generics {
    ($graph:expr, $signal_type:expr => $node_type:ident => $($options:ty)*) => {
        match $signal_type {
            $(
                t if t == <$options>::signal_type() => $graph.node($node_type::<$options>::default()),
            )*
            _ => panic!("Unsupported signal type: {:?}", $signal_type),
        }
    };
}

macro_rules! generic_binary_op_impl {
    ($self:ident, $b:ident, $op:ident => $($options:ty)*) => {{
        let graph = $self.graph();
        let b = $b.into_output(&graph);
        assert_eq!(
            $self.signal_type(),
            b.signal_type(),
            "Signal types must match for this operation",
        );
        let node = choose_node_generics!(graph, $self.signal_type() => $op => $($options)*);
        node.input(0).connect($self);
        node.input(1).connect(b);
        node
    }};
}

macro_rules! specific_binary_op_impl {
    ($self:ident, $b:ident, $op:ident => $type:ident) => {{
        let graph = $self.graph();
        assert_eq!(
            $self.signal_type(),
            $type::signal_type(),
            "Signal type must be {} for this operation",
            stringify!($type),
        );
        let b = $b.into_output(&graph);
        assert_eq!(
            $self.signal_type(),
            b.signal_type(),
            "Signal types must match for this operation",
        );
        let node = graph.node($op::default());
        node.input(0).connect($self);
        node.input(1).connect(b);
        node
    }};
}

macro_rules! generic_unary_op_impl {
    ($self:ident, $op:ident => $($options:ty)*) => {{
        let graph = $self.graph();
        let node = choose_node_generics!(graph, $self.signal_type() => $op => $($options)*);
        node.input(0).connect($self);
        node
    }};
}

macro_rules! specific_unary_op_impl {
    ($self:ident, $op:ident => $type:ident) => {{
        let graph = $self.graph();
        assert_eq!(
            $self.signal_type(),
            $type::signal_type(),
            "Signal type must be {} for this operation",
            stringify!($type),
        );
        let node = graph.node($op::default());
        node.input(0).connect($self);
        node
    }};
}

impl OutputExt for Output {
    #[inline]
    #[track_caller]
    fn channel<T: Signal + Default + Clone>(&self) -> OutputChannel<T> {
        assert_eq!(
            T::signal_type(),
            self.signal_type(),
            "Signal type must match for channel creation",
        );
        let channel = Channel::new(T::default());
        let rx = channel.rx.clone();
        let channel_node = self.graph().node(channel);
        self.connect(&channel_node.input(0));
        OutputChannel { rx }
    }

    #[inline]
    #[track_caller]
    fn powf(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Powf => f32)
    }

    #[inline]
    #[track_caller]
    fn sqrt(&self) -> Node {
        specific_unary_op_impl!(self, Sqrt => f32)
    }

    #[inline]
    #[track_caller]
    fn sin(&self) -> Node {
        specific_unary_op_impl!(self, Sin => f32)
    }

    #[inline]
    #[track_caller]
    fn cos(&self) -> Node {
        specific_unary_op_impl!(self, Cos => f32)
    }

    #[inline]
    #[track_caller]
    fn tan(&self) -> Node {
        specific_unary_op_impl!(self, Tan => f32)
    }

    #[inline]
    #[track_caller]
    fn asin(&self) -> Node {
        specific_unary_op_impl!(self, Asin => f32)
    }

    #[inline]
    #[track_caller]
    fn acos(&self) -> Node {
        specific_unary_op_impl!(self, Acos => f32)
    }

    #[inline]
    #[track_caller]
    fn atan(&self) -> Node {
        specific_unary_op_impl!(self, Atan => f32)
    }

    #[inline]
    #[track_caller]
    fn sinh(&self) -> Node {
        specific_unary_op_impl!(self, Sinh => f32)
    }

    #[inline]
    #[track_caller]
    fn cosh(&self) -> Node {
        specific_unary_op_impl!(self, Cosh => f32)
    }

    #[inline]
    #[track_caller]
    fn tanh(&self) -> Node {
        specific_unary_op_impl!(self, Tanh => f32)
    }

    #[inline]
    #[track_caller]
    fn atan2(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Atan2 => f32)
    }

    #[inline]
    #[track_caller]
    fn hypot(&self, b: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, b, Hypot => f32)
    }

    #[inline]
    #[track_caller]
    fn abs(&self) -> Node {
        specific_unary_op_impl!(self, Abs => f32)
    }

    #[inline]
    #[track_caller]
    fn ceil(&self) -> Node {
        specific_unary_op_impl!(self, Ceil => f32)
    }

    #[inline]
    #[track_caller]
    fn floor(&self) -> Node {
        specific_unary_op_impl!(self, Floor => f32)
    }

    #[inline]
    #[track_caller]
    fn round(&self) -> Node {
        specific_unary_op_impl!(self, Round => f32)
    }

    #[inline]
    #[track_caller]
    fn trunc(&self) -> Node {
        specific_unary_op_impl!(self, Trunc => f32)
    }

    #[inline]
    #[track_caller]
    fn fract(&self) -> Node {
        specific_unary_op_impl!(self, Fract => f32)
    }

    #[inline]
    #[track_caller]
    fn recip(&self) -> Node {
        specific_unary_op_impl!(self, Recip => f32)
    }

    #[inline]
    #[track_caller]
    fn signum(&self) -> Node {
        specific_unary_op_impl!(self, Signum => f32)
    }

    #[inline]
    #[track_caller]
    fn max(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Max => f32)
    }

    #[inline]
    #[track_caller]
    fn min(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Min => f32)
    }

    #[inline]
    #[track_caller]
    fn clamp(&self, min: impl IntoOutputExt, max: impl IntoOutputExt) -> Node {
        let graph = self.graph();
        let min = min.into_output(&graph);
        let max = max.into_output(&graph);
        assert_eq!(
            self.signal_type(),
            min.signal_type(),
            "Signal types must match for this operation",
        );
        assert_eq!(
            self.signal_type(),
            max.signal_type(),
            "Signal types must match for this operation",
        );
        let node = choose_node_generics!(graph, self.signal_type() => Clamp => f32);
        node.input(0).connect(self);
        node.input(1).connect(min);
        node.input(2).connect(max);
        node
    }

    #[inline]
    #[track_caller]
    fn some(&self) -> Node {
        generic_unary_op_impl!(self, Some => f32)
    }

    #[inline]
    #[track_caller]
    fn unwrap_or(&self, b: impl IntoOutputExt) -> Node {
        let graph = self.graph();
        let b = b.into_output(&graph);
        assert_eq!(
            self.signal_type(),
            Option::<f32>::signal_type(),
            "LHS Signal type must be Option<f32> for this operation",
        );
        assert_eq!(
            b.signal_type(),
            f32::signal_type(),
            "RHS Signal type must be f32 for this operation",
        );
        let node = graph.node(UnwrapOr::<f32>::default());
        node.input(0).connect(self);
        node.input(1).connect(b);
        node
    }

    fn lt<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Lt => f32)
    }

    fn gt<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Gt => f32)
    }

    fn le<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Le => f32)
    }

    fn ge<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Ge => f32)
    }

    fn eq<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Eq => f32)
    }

    fn ne<T: Signal + PartialOrd>(&self, b: impl IntoOutputExt) -> Node {
        generic_binary_op_impl!(self, b, Ne => f32)
    }

    #[track_caller]
    fn toggle(&self) -> Node {
        let graph = self.graph();
        assert_eq!(
            self.signal_type(),
            bool::signal_type(),
            "Signal type must be bool for this operation"
        );
        let node = graph.node(Toggle::default());
        node.input(0).connect(self);
        node
    }

    #[track_caller]
    fn trig_to_gate(&self, length: impl IntoOutputExt) -> Node {
        let graph = self.graph();
        assert_eq!(
            self.signal_type(),
            bool::signal_type(),
            "Signal type must be bool for this operation"
        );
        let node = graph.node(TrigToGate::default());
        node.input(0).connect(self);
        node.input(1).connect(length);
        node
    }

    #[track_caller]
    fn smooth(&self, factor: impl IntoOutputExt) -> Node {
        specific_binary_op_impl!(self, factor, Smooth => f32)
    }

    #[track_caller]
    fn scale(&self, min: impl IntoOutputExt, max: impl IntoOutputExt) -> Node {
        let graph = self.graph();
        let min = min.into_output(&graph);
        let max = max.into_output(&graph);
        assert_eq!(
            self.signal_type(),
            f32::signal_type(),
            "Signal type must be f32 for this operation"
        );
        assert_eq!(
            min.signal_type(),
            f32::signal_type(),
            "Signal type must be f32 for this operation"
        );
        assert_eq!(
            max.signal_type(),
            f32::signal_type(),
            "Signal type must be f32 for this operation"
        );
        let node = graph.node(Scale::default());
        node.input(0).connect(self);
        node.input(1).connect(min);
        node.input(2).connect(max);
        node
    }
}
