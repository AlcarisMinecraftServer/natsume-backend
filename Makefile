# ============================================================================
# Natsume Backend – Makefile
# ============================================================================
#   make setup     – rustup / cargo-edit / cargo-watch をインストール
#   make deps      – 依存クレートを追加 (初回のみ)
#   make build     – 全 crate ビルド
#   make run       – API サーバーを起動 (crates/api)
#   make watch     – コード変更を自動再ビルド & 再起動
#   make test      – 全テスト実行
#   make fmt       – rustfmt
#   make clippy    – 静的解析 (warnings = error)
# ============================================================================

CARGO ?= cargo
API_CRATE := api

setup:
	@rustup --version >/dev/null 2>&1 || (echo "[!] rustup not found" && exit 1)
	@$(CARGO) install --locked cargo-edit cargo-watch sqlx-cli --no-default-features --features postgres || true
	@echo "[✓] Toolchain ready"

deps:
	@cd crates/$(API_CRATE) && \
		$(CARGO) add axum tokio --features tokio/full || true && \
		$(CARGO) add serde serde_json once_cell tracing-subscriber || true && \
		$(CARGO) add shared --path ../shared || true
	@echo "[✓] Dependencies ensured"

.PHONY: build run watch clean

build:
	$(CARGO) build --workspace

run:
	$(CARGO) run -p $(API_CRATE)

watch:
	$(CARGO) watch -p $(API_CRATE) -x "run -p $(API_CRATE)"

clean:
	$(CARGO) clean

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test:
	$(CARGO) test --workspace

migrate:
	@sqlx migrate run --source crates/migration --database-url $$DATABASE_URL

.DEFAULT_GOAL := run
