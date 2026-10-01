fn main() {
    #[cfg(feature = "gui")]
    {
        slint_build::compile("ui/logmaster_ui.slint").unwrap();
    }

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("ui/icons/tpc.ico");
        let _ = res.compile();
    }
}
