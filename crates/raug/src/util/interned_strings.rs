use std::sync::OnceLock;

#[inline]
pub fn interned_short_type_name<T: ?Sized>() -> &'static str {
    static NAME: OnceLock<String> = OnceLock::new();
    NAME.get_or_init(tynm::type_name::<T>)
}
