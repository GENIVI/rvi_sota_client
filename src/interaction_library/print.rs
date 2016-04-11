pub trait Print: Sized {
    fn pretty_print(&self) -> String;
}
