fn main() {
    println!("cargo:rerun-if-changed=assets/icons/app.ico");

    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icons/app.ico");
        resource.set("FileDescription", "Rarust Archive Browser");
        resource.set("ProductName", "Rarust");

        if let Err(error) = resource.compile() {
            panic!("failed to compile Windows resources: {error}");
        }
    }
}
