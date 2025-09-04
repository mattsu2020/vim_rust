# Repository Guidelines

## プロジェクト構成 / モジュール
- ルート: Rust 2021 のワークスペース。`Cargo.toml` は `members = ["rust_*", "rust/*"]` を管理。
- ライブラリ: ルート crate は `vim_channel`（エントリは `src/channel.rs`）。
- 下位クレート: 各機能は `rust_*` ディレクトリ（例: `rust_buffer`, `rust_spell`）。`rust/` 配下にも一部サブクレート（例: `rust/diff`）。
- テスト: ルートの `tests/` と各クレート内の `tests`/モジュールテスト。
- ドキュメント/CI: `docs/`, `.github/`, `ci/` を参照。

## ビルド / テスト / 開発
- 全体ビルド: `cargo build --workspace`（OS 別依存は自動切替）。
- ルートのみ: `cargo build -p vim_channel`。
- テスト（全体）: `cargo test --workspace`。
- テスト（個別）: `cargo test -p rust_buffer` など。
- 静的解析: `cargo clippy --workspace -D warnings`。
- フォーマット: `cargo fmt --all`。

## コーディング規約 / 命名
- フォーマット: rustfmt 準拠。未整形差分は PR 前に整形。
- インデント: 半角スペース 4。行末空白は削除。
- 命名: crate/モジュール/関数=snake_case、型/トレイト=UpperCamelCase、定数=SCREAMING_SNAKE_CASE。
- 設計: `unsafe` は最小化。クロスプラットフォーム分岐は `cfg(...)` で明示。

## テスト方針
- フレームワーク: 標準 `#[test]`。失敗の再現→修正→回帰テストを基本。
- 追加場所: ルート機能は `tests/*.rs`、クレート固有は当該クレート配下。
- 命名: 仕様単位でテスト関数を分け、入力/出力が明確になる名称にする（例: `parses_invalid_json_returns_error`）。
- カバレッジ目安: 重要ロジックは分岐網羅を意識。

## コミット / PR ガイドライン
- コミット: 命令形・短く具体的に。必要に応じてスコープを前置（例: `rust_buffer: fix off-by-one`、`feat: add rust_terminal crate`）。
- まとめコミットは避け、論理単位で分割。`Signed-off-by` の付与を推奨（`git commit -s`）。
- PR 要件: 目的/背景、変更点、影響範囲、テスト方法、関連 Issue。CI がグリーンであること。

## セキュリティ / 設定のヒント
- 環境: 安定版ツールチェーン推奨（`rustup default stable`）。
- 依存: 追加時は最小バージョンポリシーとライセンス確認。
- 実行時設定: OS 依存コードは `cfg` でガードし、機能フラグは `features` で明示管理。

## ドキュメント運用（docs/）
- 目的: 仕様メモ、設計判断、TODO、クラウド上の Codex に伝えたい事項を Markdown 化し共有。
- 置き場所: `docs/` 直下。例: `docs/todo-2025-09.md`, `docs/codex-notes-channel.md`。
- 記載指針: 背景→課題→決定→次アクションの順で簡潔に。コードやコマンドはバッククォートで明示。
- 参照: PR 説明から関連 md へリンク。更新差分が CI のレビュー対象になるよう、PR に含めること。
- 補助: 一覧確認は `ls docs/*.md`。検索は `rg -n "TODO|Codex|設計" docs`。
