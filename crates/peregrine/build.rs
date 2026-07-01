use std::env;

fn main() {
    // 仅在 Windows 构建时嵌入 exe 图标资源。
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        let _ = embed_resource::compile("assets.rc", embed_resource::NONE);
    }
}
