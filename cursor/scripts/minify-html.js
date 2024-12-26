#!/usr/bin/env node

const { minify } = require('html-minifier-terser');
const fs = require('fs');
const path = require('path');

// 配置选项
const options = {
  collapseWhitespace: true,
  removeComments: true,
  removeEmptyAttributes: true,
  removeOptionalTags: true,
  removeRedundantAttributes: true,
  removeScriptTypeAttributes: true,
  removeStyleLinkTypeAttributes: true,
  minifyCSS: true,
  minifyJS: true,
  processScripts: ['application/json'],
};

// 处理文件
async function minifyFile(inputPath, outputPath) {
  try {
    const html = fs.readFileSync(inputPath, 'utf8');
    const minified = await minify(html, options);
    fs.writeFileSync(outputPath, minified);
    console.log(`✓ Minified ${path.basename(inputPath)} -> ${path.basename(outputPath)}`);
  } catch (err) {
    console.error(`✗ Error processing ${inputPath}:`, err);
    process.exit(1);
  }
}

// 主函数
async function main() {
  const staticDir = path.join(__dirname, '..', 'static');
  const files = [
    ['tokeninfo.html', 'tokeninfo.min.html'],
  ];

  for (const [input, output] of files) {
    await minifyFile(
      path.join(staticDir, input),
      path.join(staticDir, output)
    );
  }
}

main();