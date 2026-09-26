# えぬちゃんらんど

数学の内容を一方的に配信する学習サイトである．公開先は[https://enunun.github.io/math-study/](https://enunun.github.io/math-study/)である．

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

## 記事の追加

1. カテゴリのディレクトリ(`site/src/content/docs/topics/`や`tools/`)に，MDXファイルを作る．ファイル名がURLになる．frontmatterには，`title`と`description`を書く．ほかのページから定理や式を参照されるなら，短い`pageId`も付ける．`pageId`は，サイト内で重複させない．
2. 本文は，`.claude/skills/write-content/SKILL.md`の規則と，記法のページ(`/math-study/dev/notation/`)の書き方に従う．図は，シーンのJSONを`site/src/figures/`に置き，`<Figure src="…" />`で載せる．PDFなどの既存の記事を書き起こしたページには，冒頭の`Aside`で，Claude Codeによる書き起こしであることを示す．
3. ホーム(`site/src/content/docs/index.mdx`)のカテゴリの見出しの下に，`LinkCard`を足す．サイドバーはディレクトリから作られるので，変更は要らない．
4. `mise run check`と`mise run e2e`を通し，`mise run screenshot`で見た目を確かめてから，pushする．

新しいカテゴリを作るときは，`site/astro.config.ts`の`sidebar`にグループを足し，ホームに見出しとカードの一覧を足す．開発用のページ(`dev/`)は，frontmatterでサイドバーと検索から外す．そのうえで，ホームの「開発用のページ」にカードを足し，`e2e/home.spec.ts`の`DEV_PAGES`にも加える．

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
