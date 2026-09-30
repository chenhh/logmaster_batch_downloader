fn main() {
    #[cfg(feature = "gui")]
    {
        slint_build::compile("ui/logmaster_ui.slint").unwrap();
    }
}
