.PHONY: build release install uninstall clean test

# 默认构建
build:
	cargo build

# Release 构建
release:
	cargo build --release

# 安装到 ~/.local/bin
install: release
	@mkdir -p ~/.local/bin
	@cp target/release/dijiang ~/.local/bin/dijiang
	@echo "✅ dijiang 已安装到 ~/.local/bin/dijiang"
	@echo "   请确保 ~/.local/bin 在 PATH 中"

# 卸载
uninstall:
	@rm -f ~/.local/bin/dijiang
	@echo "✅ dijiang 已卸载"

# 清理构建产物
clean:
	cargo clean

# 运行测试
test:
	cargo test --test e2e

# 格式化代码
fmt:
	cargo fmt

# 代码检查
check:
	cargo check

# 完整检查（格式化 + 检查 + 测试）
ci: fmt check test
	@echo "✅ 所有检查通过"
