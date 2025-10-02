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
