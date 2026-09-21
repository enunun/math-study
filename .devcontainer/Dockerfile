# mise公式のDebianイメージ．git，curl，CA証明書とmiseを含み，
# PATHに/mise/shims，MISE_DATA_DIRに/miseが設定済み．
FROM jdxcode/mise:2026.9.12-debian

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	openssh-client \
	procps \
	jq \
	# Rustのテストを，ネイティブで実行するためのリンカ．Wasmのビルドには要らない．
	gcc \
	libc6-dev \
	&& rm -rf /var/lib/apt/lists/*

# ツール(Node，pnpm，gh，rtkなど)はmise.tomlとmise.lockのとおりにイメージへ入れる．
WORKDIR /tmp/mise-bootstrap
COPY mise.toml mise.lock ./
RUN mise trust \
	&& mise install --locked

# Playwright(ブラウザでの動作確認)のChromiumと，実行に必要なシステムのライブラリを入れる．
# 版は，package.jsonの@playwright/testに合わせる．食い違っても，mise run browsersで必要な版が加わる．
ARG PLAYWRIGHT_VERSION=1.63.0
ENV PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright
RUN mise exec -- npx --yes playwright@${PLAYWRIGHT_VERSION} install --with-deps --only-shell chromium \
	&& rm -rf /var/lib/apt/lists/* /root/.npm /mise/cache /tmp/mise-bootstrap

# 起動後は，リポジトリのmise.tomlをグローバル設定として読む．
# これにより，/workspacesの外のディレクトリでも同じツールを使える．
# グローバル設定のルートも/workspacesにして，タスクをリポジトリ直下で実行させる
# (未指定だとホームディレクトリで実行される)．
ENV MISE_GLOBAL_CONFIG_FILE=/workspaces/mise.toml \
	MISE_GLOBAL_CONFIG_ROOT=/workspaces \
	MISE_TRUSTED_CONFIG_PATHS=/workspaces

WORKDIR /workspaces
