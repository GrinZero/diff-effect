rslib build
pnpm build:rust
# 复制 pkg 文件夹到 dist/pkg
mkdir -p dist/pkg && cp -r pkg/* dist/pkg/