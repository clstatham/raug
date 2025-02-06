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
    then: AnySignalOpt,
    else_: AnySignalOpt,
}

impl Cond {
    /// Creates a new `Cond` processor.
    pub fn new(signal_type: SignalType) -> Self {
        Self {
            cond: false,
            then: AnySignalOpt::default_of_type(&signal_type),
            else_: AnySignalOpt::default_of_type(&signal_type),
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
        outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        for (cond, then, else_, mut out) in raug_macros::iter_proc_io_as!(
            inputs as [bool, Any, Any],
            outputs as [Any]
        ) {
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
                out.clone_from_opt_ref(self.then.as_ref());
            } else {
                out.clone_from_opt_ref(self.else_.as_ref());
            }
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
            a: AnySignalOpt,
            b: AnySignalOpt,
        }

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "` processor for the given type.")]
            pub fn new(signal_type: SignalType) -> Self {
                Self {
                    a: AnySignalOpt::default_of_type(&signal_type),
                    b: AnySignalOpt::default_of_type(&signal_type),
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
                outputs: ProcessorOutputs,
            ) -> Result<(), ProcessorError> {
                for (a, b, out) in raug_macros::iter_proc_io_as!(
                    inputs as [Any, Any],
                    outputs as [bool]
                )
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
                        *out = None;
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
                        *out = None;
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
                        (AnySignalOpt::Bool(Some(a)), AnySignalOpt::Bool(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignalOpt::Int(Some(a)), AnySignalOpt::Int(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignalOpt::Float(Some(a)), AnySignalOpt::Float(Some(b))) => {
                            *out = Some(*a $op *b);
                        }
                        (AnySignalOpt::Midi(Some(a)), AnySignalOpt::Midi(Some(b))) => {
                            *out = Some(*a $op *b);
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Select {
    signal_type: SignalType,
    num_outputs: usize,
}

impl Select {
    pub fn new(signal_type: SignalType, num_outputs: usize) -> Self {
        Self {
            signal_type,
            num_outputs,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Select {
    fn input_spec(&self) -> Vec<SignalSpec> {
        vec![
            SignalSpec::new("in", self.signal_type),
            SignalSpec::new("index", SignalType::Int),
        ]
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        (0..self.num_outputs)
            .map(|i| SignalSpec::new(format!("out{}", i), self.signal_type))
            .collect()
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(input_signal) = inputs.input(0) else {
            return Ok(());
        };
        let Some(index) = inputs.input_as::<i64>(1) else {
            return Ok(());
        };

        for (sample_index, index) in index.iter().enumerate() {
            let Some(index) = index else {
                for j in 0..self.num_outputs {
                    outputs.output(j).set_none(sample_index);
                }
                continue;
            };
            let index = *index as usize;
            for j in 0..self.num_outputs {
                if j == index {
                    if let Some(input) = input_signal.get(sample_index) {
                        outputs.output(j).set(sample_index, input.to_owned());
                    } else {
                        outputs.output(j).set_none(sample_index);
                    }
                } else {
                    outputs.output(j).set_none(sample_index);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Merge {
    signal_type: SignalType,
    num_inputs: usize,
}

impl Merge {
    pub fn new(signal_type: SignalType, num_inputs: usize) -> Self {
        Self {
            signal_type,
            num_inputs,
        }
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl Processor for Merge {
    fn input_spec(&self) -> Vec<SignalSpec> {
        let mut inputs = Vec::with_capacity(self.num_inputs + 1);
        inputs.push(SignalSpec::new("index", SignalType::Int));
        for i in 0..self.num_inputs {
            inputs.push(SignalSpec::new(format!("in{}", i), self.signal_type));
        }
        inputs
    }

    fn output_spec(&self) -> Vec<SignalSpec> {
        vec![SignalSpec::new("out", self.signal_type)]
    }

    fn process(
        &mut self,
        inputs: ProcessorInputs,
        mut outputs: ProcessorOutputs,
    ) -> Result<(), ProcessorError> {
        let Some(index) = inputs.input_as::<i64>(0) else {
            return Ok(());
        };

        for (sample_index, index) in index.iter().enumerate() {
            let Some(index) = index else {
                outputs.output(0).set_none(sample_index);
                continue;
            };
            let index = *index as usize;
            for i in 0..self.num_inputs {
                let Some(input_signal) = inputs.input(i + 1) else {
                    outputs.output(0).set_none(sample_index);
                    continue;
                };
                if let Some(input) = input_signal.get(sample_index) {
                    if i == index {
                        outputs.output(0).set(sample_index, input.to_owned());
                        break;
                    }
                } else {
                    outputs.output(0).set_none(sample_index);
                    break;
                }
            }
        }

        Ok(())
    }
}
