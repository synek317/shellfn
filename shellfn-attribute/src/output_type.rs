pub enum OutputType {
    T,
    Iter,
    Vec,
    Void,
}

impl Default for OutputType {
    fn default() -> Self {
        OutputType::T
    }
}
