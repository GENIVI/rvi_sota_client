pub trait Parse: Sized {
    fn parse(s: String) -> Option<Self>;
}
