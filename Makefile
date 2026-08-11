.PHONY: build release install uninstall clean test test-pi-extension validate-skills fmt check ci

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
	cargo test --workspace

test-pi-extension:
	node crates/configurator/tests/pi_extension_contract.mjs "$(CURDIR)/crates/configurator/templates/extensions/dijiang/index.ts"

validate-skills: build
	./target/debug/dijiang skills --validate

# 格式化代码
fmt:
	cargo fmt --all

# 代码检查
check:
	cargo check --workspace

# 完整检查（patch hygiene + 检查 + 全仓测试 + 扩展/技能契约）
ci: build
	git diff --check
	cargo check --workspace
	cargo test --workspace
	$(MAKE) test-pi-extension
	./target/debug/dijiang skills --validate
	@echo "✅ 所有检查通过"
