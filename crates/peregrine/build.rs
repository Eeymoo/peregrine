fn main() {
    // 仅在 Windows target 构建时嵌入 exe 图标资源。
    // embed-resource 仅在 [target.'cfg(windows)'.build-dependencies] 中声明，
    // 因此 build.rs 中的调用也必须用 cfg(target_os = "windows") 守卫。
    #[cfg(target_os = "windows")]
    {
        let _ = embed_resource::compile("assets.rc", embed_resource::NONE);
    }
}
