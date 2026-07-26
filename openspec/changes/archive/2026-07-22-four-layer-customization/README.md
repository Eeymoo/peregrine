# four-layer-customization

引入 Rhai 嵌入式脚本作为物料运行时，将现有 12 种 CrosshairStyle 迁移为内置 Rhai 物料。新增 Element / Material / Layer / Profile 四层数据模型，支持多图层叠加与用户自定义物料。Rhai 脚本支持时间、鼠标位置、键盘状态、随机数等动态输入。前端预览改走后端 IPC，彻底消除 TS 与 Rust 两次手写几何的 WYSIWYG 问题。
