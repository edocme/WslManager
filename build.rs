fn main() {
    if cfg!(target_os = "windows") {
        let _ = embed_resource::compile("app.rc", None::<&str>);
    }
}
