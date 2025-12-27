use std::io::Result;

fn main() -> Result<()> {
    // 告诉 Cargo 当 proto 文件变化时重新运行此脚本
    println!("cargo:rerun-if-changed=proto/");
    
    // 确保输出目录存在
    std::fs::create_dir_all("src/proto/")?;
    
    // 编译 proto 文件到 src/pb 目录
    prost_build::Config::new()
        .out_dir("src/proto/")
        .compile_protos(
            &[
                "proto/declare.proto", 
                "proto/cmd.proto", 
                "proto/rsrc.proto",
                "proto/transform.proto", 
                "proto/conception.proto", 
                "proto/designation.proto",
                "proto/shape.proto",
                "proto/formats.proto"
            ], 
            &["proto/"]  // 设置 proto 目录作为根搜索路径，这样所有相对导入都能正确解析
        )?;
        
    Ok(())
}