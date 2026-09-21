# 数学の学習サイト

数学の内容を一方的に配信する学習サイトである．公開先は[https://enunun.github.io/math-study/](https://enunun.github.io/math-study/)である．

## 状態

- 実装済み(サイト)：Astro 7とStarlightのサイト，折り畳みのコンポーネント(`Proof`，`Detail`，`Remark`)，記法のテストページ．
- 実装済み(開発基盤)：リントと整形，コミット時の検査，E2Eテスト，GitHub Pagesへの公開．
- 実装済み(数式)：MathJax 4による数式の描画(自作マクロ，証明図，読み上げ用の`aria-label`)．未定義のマクロと構文の誤りは，ビルドの失敗にする．
- 実装済み(定理)：定義，補題，命題，定理，系(`Definition`，`Lemma`，`Proposition`，`Theorem`，`Corollary`)の自動採番と，同じページやほかのページからの参照(`Ref`)．
- 実装済み(式番号)：別行立ての式の`\label`による自動採番と，`Ref`による参照．
- 予定：Rustで書いた多項式電卓とグラフ(Wasm)．

サイトの仕組み(どの部品が何をしているか，どこが自作か，どう使うか)は，[仕組みの解説](https://enunun.github.io/math-study/dev/internals/)にある．ソースは`site/src/content/docs/dev/internals.mdx`である．

技術選定の理由と，確認した事実は，[docs/tech-decisions.md](docs/tech-decisions.md)(英語)にまとめてある．

## 技術構成

| 領域             | 使うもの                            |
| ---------------- | ----------------------------------- |
| サイト           | Astro 7，Starlight，MDX             |
| ツール管理       | mise(Node，pnpm，gh，rtk，lefthook) |
| 整形とリント     | oxfmt，oxlint                       |
| 文章の検査       | textlint，remark-lint，markdownlint |
| コミット時の検査 | lefthook                            |
| ブラウザでの確認 | Playwright                          |
| 公開             | GitHub Actions，GitHub Pages        |

## 開発環境

devcontainerで開く．Dockerfileは，mise公式のDebianイメージを土台に，ツールとPlaywrightのChromiumを入れる．コンテナの作成後に，`mise run setup`が依存パッケージ，型定義，ブラウザ，Gitのフックを整える．

コンテナの外では，miseを入れて`mise run setup`を実行する．Playwrightのブラウザは，Linuxではシステムのライブラリも必要になる．

## コマンド

作業は，`mise run`のタスクで行う．一覧は`mise tasks`で表示する．

| タスク                | 内容                                                                   |
| --------------------- | ---------------------------------------------------------------------- |
| `mise run dev`        | 開発サーバーを起動する(`http://localhost:4321/math-study/`)            |
| `mise run build`      | サイトをビルドする                                                     |
| `mise run preview`    | ビルドしたサイトを，公開時と同じbaseパスで確認する                     |
| `mise run check`      | 整形，リント，型検査，単体テスト，ビルドを検査する．CIと同じ内容である |
| `mise run lint`       | oxlint，markdownlint，remark-lint，textlintを実行する                  |
| `mise run fmt`        | コードと文書を整形する                                                 |
| `mise run test`       | 単体テストを実行する                                                   |
| `mise run e2e`        | ブラウザで動作を確認する                                               |
| `mise run screenshot` | ページのスクリーンショットを撮る                                       |

変更したら，`mise run check`を通す．表示や動作に関わる変更は，`mise run e2e`も通す．手順の詳細は，`.claude/skills/verify-site/SKILL.md`にある．

## ディレクトリ構成

| パス             | 内容                                                                              |
| ---------------- | --------------------------------------------------------------------------------- |
| `site/`          | Astroのサイト．`src/content/docs/`に文書，`src/components/`にコンポーネントを置く |
| `e2e/`           | E2Eテスト，配信サーバー，スクリーンショットの道具                                 |
| `docs/`          | 技術選定の記録などの開発者向けの文書                                              |
| `.claude/`       | Claude Codeの設定と，Skill                                                        |
| `.devcontainer/` | 開発コンテナの定義                                                                |
| `.github/`       | GitHub Actionsのワークフローと，Dependabotの設定                                  |

## 検査と規則

コミット時に，lefthookがステージしたファイルを検査する．CIは，`mise run check`で同じ検査をリポジトリ全体に対して実行する．

- コードは，oxlintの厳格な設定で検査する．規則を外すときは，`.oxlintrc.jsonc`に理由をコメントで書く．
- 文書は，`site/src/content`と`README.md`をtextlintで，MDXをremark-lintで，`README.md`と`docs/`をmarkdownlintで検査する．
- 日本語の文章は，常体で書く．句読点は「，」と「．」にする．和文と欧文の間には，スペースを入れない．和文の後ろのコロンは，全角の「：」にする．AIが書いたように見える書き方を避ける．
- Claude用の文書(`CLAUDE.md`，`.claude/skills/`，`docs/`)は英語で書く．日本語で書くのは，READMEとサイトの文書である．
- 記法の一覧と，動作確認の項目は，テストページ(`/math-study/dev/notation/`)にある．サイドバーと検索には出ない．

## 公開

`main`へpushすると，GitHub Actionsがビルドして，GitHub Pagesへ公開する．個人で開発するため，`main`だけで作業し，ブランチとプルリクエストは使わない．
