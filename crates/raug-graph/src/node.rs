use crate::TypeInfo;

pub trait AbstractNode {
    fn name(&self) -> Option<String> {
        None
    }
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn input_type(&self, index: u32) -> Option<TypeInfo>;
    fn output_type(&self, index: u32) -> Option<TypeInfo>;
    fn input_name(&self, index: u32) -> Option<&str>;
    fn output_name(&self, index: u32) -> Option<&str>;
}
