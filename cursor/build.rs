use std::io::Result;
use std::path::Path;
use std::process::Command;

fn check_and_install_deps() -> Result<()> {
    let scripts_dir = Path::new("scripts");
    let node_modules = scripts_dir.join("node_modules");

    // 如果 node_modules 不存在，运行 npm install
    if !node_modules.exists() {
        println!("cargo:warning=Installing HTML minifier dependencies...");

        let status = Command::new("npm")
            .current_dir(scripts_dir)
            .arg("install")
            .status()?;

        if !status.success() {
            panic!("Failed to install npm dependencies");
        }
        println!("cargo:warning=Dependencies installed successfully");
    }
    Ok(())
}

fn minify_html() -> Result<()> {
    println!("cargo:warning=Minifying HTML files...");

    let status = Command::new("node")
        .args(&["scripts/minify-html.js"])
        .status()?;

    if !status.success() {
        panic!("HTML minification failed");
    }
    Ok(())
}

fn main() -> Result<()> {
    // Proto 文件处理
    println!("cargo:rerun-if-changed=src/message.proto");
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config
        .compile_protos(&["src/message.proto"], &["src/"])
        .unwrap();

    // HTML 文件处理
    println!("cargo:rerun-if-changed=static/tokeninfo.html");
    println!("cargo:rerun-if-changed=scripts/minify-html.js");
    println!("cargo:rerun-if-changed=scripts/package.json");

    // 检查并安装依赖
    check_and_install_deps()?;

    // 运行 HTML 压缩
    minify_html()?;

    Ok(())
}
