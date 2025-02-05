//! Control flow processors.

use crate::prelude::*;

/// A processor that outputs the value of the second input if the first input is `true`, otherwise the value of the third input.
///
/// # Inputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `cond` | `Bool` | The condition. |
/// | `1` | `then` | `Any` | The value to output if the condition is `true`. |
/// | `2` | `else` | `Any` | The value to output if the condition is `false`. |
///
/// # Outputs
///
/// | Index | Name | Type | Description |
/// | --- | --- | --- | --- |
/// | `0` | `out` | `Any` | The output value. |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cond {
    cond: bool,
    then: AnySignal,
    else_: AnySignal,
}

impl Cond {
    /// Creates a new `Cond` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            cond: false,
            then: AnySignal::default_of_type(&signal_type),
            else_: AnySignal::default_of_type(&signal_type),
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Cond {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("cond", SignalType::Bool),
            SignalSpec::new("then", self.then.signal_type()),
            SignalSpec::new("else", self.else_.signal_type()),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.then.signal_type())]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let cond = inputs.input_as::<bool>(0)?;
        let then = inputs.input(1)?.as_any_signal_ref();
        let else_ = inputs.input(2)?.as_any_signal_ref();

        self.cond = cond.unwrap_or(self.cond);

        if let Some(then) = then {
            if then.signal_type() != self.then.signal_type() {
                return Err(ProcessorError::InputSpecMismatch {
                    index: 1,
                    expected: self.then.signal_type(),
                    actual: then.signal_type(),
                });
            }
            self.then.clone_from_ref(then);
        }

        if let Some(else_) = else_ {
            if else_.signal_type() != self.else_.signal_type() {
                return Err(ProcessorError::InputSpecMismatch {
                    index: 2,
                    expected: self.else_.signal_type(),
                    actual: else_.signal_type(),
                });
            }
            self.else_.clone_from_ref(else_);
        }

        if self.cond {
            outputs.set_output(0, self.then)?;
        } else {
            outputs.set_output(0, self.else_)?;
        }

        Ok(())
    }
}

macro_rules! comparison_op {
    ($doc:literal, $name:ident, $op:tt) => {
        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[doc = $doc]
        pub struct $name {
            a: AnySignal,
            b: AnySignal,
        }

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor for the given type.")]
            pub fn new(signal_type: SignalType) -> Self {
                Self {
                    a: AnySignal::default_of_type(&signal_type),
                    b: AnySignal::default_of_type(&signal_type),
                }
            }
        }

        #[cfg_attr(feature = "serde", typetag::serde)]
        impl Processor for $name {
            fn input_spec(&self) -> Vec<SignalSpec> {
                vec![
                    SignalSpec::new("a", self.a.signal_type()),
                    SignalSpec::new("b", self.b.signal_type()),
                ]
            }

            fn output_spec(&self) -> Vec<SignalSpec> {
                vec![SignalSpec::new("out", SignalType::Bool)]
            }

            fn process(
                &mut self,
                inputs: ProcessorInputs,
                mut outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                let a = inputs.input(0)?.as_any_signal_ref();
                let b = inputs.input(1)?.as_any_signal_ref();
                {
                    if let Some(a) = a {
                        if a.signal_type() != self.a.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 0,
                                expected: self.a.signal_type(),
                                actual: a.signal_type(),
                            });
                        }
                        self.a.clone_from_ref(a);
                    } else {
                        outputs.set_output_none(0);
                        return Ok(());
                    }

                    if let Some(b) = b {
                        if b.signal_type() != self.b.signal_type() {
                            return Err(ProcessorError::InputSpecMismatch {
                                index: 1,
                                expected: self.b.signal_type(),
                                actual: b.signal_type(),
                            });
                        }
                        self.b.clone_from_ref(b);
                    } else {
                        outputs.set_output_none(0);
                        return Ok(());
                    }

                    if self.a.signal_type() != self.b.signal_type() {
                        return Err(ProcessorError::InputSpecMismatch {
                            index: 0,
                            expected: self.a.signal_type(),
                            actual: self.b.signal_type(),
                        });
                    }

                    match (&self.a, &self.b) {
                        (AnySignal::Bool(a), AnySignal::Bool(b)) => {
                            outputs.set_output_as(0, *a $op *b)?;
                        }
                        (AnySignal::Int(a), AnySignal::Int(b)) => {
                            outputs.set_output_as(0, *a $op *b)?;
                        }
                        (AnySignal::Float(a), AnySignal::Float(b)) => {
                            outputs.set_output_as(0, *a $op *b)?;
                        }
                        (AnySignal::Midi(a), AnySignal::Midi(b)) => {
                            outputs.set_output_as(0, *a $op *b)?;
                        }
                        _ => unreachable!(),
                    }
                }

                Ok(())
            }
        }
    };
}

comparison_op!(
    r#"
A processor that outputs `true` if `a` is less than `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    Less,
    <
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is greater than `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    Greater,
    >
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    Equal,
    ==
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is not equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    NotEqual,
    !=
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is less than or equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    LessOrEqual,
    <=
);

comparison_op!(
    r#"
A processor that outputs `true` if `a` is greater than or equal to `b`, otherwise `false`.

# Inputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `a` | `Any` | The first value to compare. |
| `1` | `b` | `Any` | The second value to compare. |

# Outputs

| Index | Name | Type | Description |
| --- | --- | --- | --- |
| `0` | `out` | `Any` | The result of the comparison. |
"#,
    GreaterOrEqual,
    >=
);
