# えぬちゃんらんど

数学の内容を一方的に配信する学習サイトである．公開先は[https://enunun.github.io/math-study/](https://enunun.github.io/math-study/)である．

## 機能

- サイト：Astro 7とStarlightで作る．公開するページは，カテゴリ(単発ネタ，ツール)ごとに，ホームのカード一覧とサイドバーのグループに並べる．著書の紹介は，カテゴリの外の1ページである．
- 数式：MathJax 4でビルド時に描画する(自作マクロ，証明図，読み上げ用の`aria-label`)．未定義のマクロと構文の誤りは，ビルドの失敗にする．
- 定理と式番号：定義，補題，命題，定理，系(`Definition`，`Lemma`，`Proposition`，`Theorem`，`Corollary`)と，別行立ての式の`\label`を自動で採番する．同じページやほかのページから`Ref`で参照できる．
- 折り畳み：証明や補足を`Proof`，`Detail`，`Remark`で折り畳む．
- 計算機：Rustで書いた多項式の計算機(式の展開と微分)を，Wasmにしてブラウザで動かす．核は`crates/`にある．
- 図：図のシーン(JSON)から，SVGの図とTikZの出力を作る．核はRustで書いてWasmにし，ビルド時とブラウザで動かす．図を作るページでは，オブジェクトを足して設定すると記事と同じエンジンが描く．シーンのJSONの読み書きと，TikZのダウンロードもできる．設計は`docs/graph-design.md`にある．
- 開発基盤：リントと整形，コミット時の検査，E2Eテスト(axeによるアクセシビリティ検査，リンク切れの検査を含む)，GitHub Pagesへの公開．E2Eが失敗すると，公開されない．

サイトの仕組み(どの部品が何をしているか，どこが自作か，どう使うか)は，[仕組みの解説](https://enunun.github.io/math-study/dev/internals/)にある．ソースは`site/src/content/docs/dev/internals.mdx`である．

技術選定の理由と，確認した事実は，[docs/tech-decisions.md](docs/tech-decisions.md)(英語)にまとめてある．

## 技術構成

| 領域             | 使うもの                                                |
| ---------------- | ------------------------------------------------------- |
| サイト           | Astro 7，Starlight，MDX                                 |
| ツール管理       | mise(Node，pnpm，gh，rtk，lefthook，Rust，wasm-bindgen) |
| 整形とリント     | oxfmt，oxlint                                           |
| 文章の検査       | textlint，remark-lint，markdownlint                     |
| コミット時の検査 | lefthook                                                |
| ブラウザでの確認 | Playwright                                              |
| TikZの出力       | TeX Live(LuaLaTeX)，poppler-utils                       |
| 計算機の核       | Rust，WebAssembly，wasm-bindgen，React                  |
| 公開             | GitHub Actions，GitHub Pages                            |

## 開発環境

devcontainerで開く．Dockerfileは，mise公式のDebianイメージを土台に，ツール(Rustを含む)とPlaywrightのChromiumを入れる．コンテナの作成後に，`mise run setup`が依存パッケージ，型定義，ブラウザ，Gitのフックを整える．

コンテナの外では，miseを入れて`mise run setup`を実行する．Playwrightのブラウザは，Linuxではシステムのライブラリも必要になる．

## コマンド

作業は，`mise run`のタスクで行う．一覧は`mise tasks`で表示する．

| タスク                 | 内容                                                                       |
| ---------------------- | -------------------------------------------------------------------------- |
| `mise run dev`         | 開発サーバーを起動する(`http://localhost:4321/math-study/`)                |
| `mise run build`       | サイトをビルドする                                                         |
| `mise run preview`     | ビルドしたサイトを，公開時と同じbaseパスで確認する                         |
| `mise run check`       | 整形，リント，型検査，単体テスト，ビルドを検査する．CIと同じ内容である     |
| `mise run lint`        | oxlint，rustfmtとclippy，markdownlint，remark-lint，textlintを実行する     |
| `mise run wasm`        | Rustの計算機と，図のシーンの窓口を，Wasmにして，`site/src/wasm/`へ出力する |
| `mise run fmt`         | コードと文書を整形する                                                     |
| `mise run test`        | 単体テストを実行する                                                       |
| `mise run e2e`         | ブラウザで動作を確認する                                                   |
| `mise run screenshot`  | ページのスクリーンショットを撮る                                           |
| `mise run tikz`        | 図のTikZの出力を，LuaLaTeXでコンパイルして，SVGの図と比べる(手元だけ)      |
| `mise run tikz:export` | 図のTikZの出力を，ファイルとPDFに書き出す(手元だけ)                        |

変更したら，`mise run check`を通す．表示や動作に関わる変更は，`mise run e2e`も通す．手順の詳細は，`.claude/skills/verify-site/SKILL.md`にある．

## ディレクトリ構成

| パス             | 内容                                                                              |
| ---------------- | --------------------------------------------------------------------------------- |
| `crates/`        | Rustのクレート．多項式の計算機と，図のシーンの，核とWasmの窓口                    |
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
