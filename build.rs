fn main() {
    // Only build for Windows
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("static/icon.ico");
        res.compile().unwrap();
    }
}
