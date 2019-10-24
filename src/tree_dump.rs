pub trait TreeDump {
    fn print(&self) {
        self.print_with_indent(0, " : ");
    }
    fn print_with_indent(&self, indent: usize, indent_style: &str);
}