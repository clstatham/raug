use raug::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListError {
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}

#[processor(derive(Default))]
pub fn get<T>(
    #[input] list: &List<T>,
    #[input] index: &f32,
    #[output] out: &mut T,
) -> ProcResult<()>
where
    T: Signal + Default + Clone,
{
    let index = *index as i32;
    if index < 0 || index >= list.len() as i32 {
        return Err(ProcessorError::new(ListError::IndexOutOfBounds(
            index as usize,
        )));
    }

    out.clone_from(&list[index as usize]);
    Ok(())
}
