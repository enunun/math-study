# mise 公式の Debian イメージ。git、curl、CA 証明書と mise を含み、
# PATH に /mise/shims、MISE_DATA_DIR に /mise が設定済み。
FROM jdxcode/mise:2026.9.12-debian

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	openssh-client \
	procps \
	&& rm -rf /var/lib/apt/lists/*

# ツール(Node、pnpm、gh、rtk など)は mise.toml と mise.lock のとおりにイメージへ入れる。
WORKDIR /tmp/mise-bootstrap
COPY mise.toml mise.lock ./
RUN mise trust \
	&& mise install --locked \
	&& rm -rf /mise/cache /tmp/mise-bootstrap

# 起動後は、リポジトリの mise.toml をグローバル設定として読む。
# これにより、/workspaces の外のディレクトリでも同じツールを使える。
# グローバル設定のルートも /workspaces にして、タスクをリポジトリ直下で実行させる
# (未指定だとホームディレクトリで実行される)。
ENV MISE_GLOBAL_CONFIG_FILE=/workspaces/mise.toml \
	MISE_GLOBAL_CONFIG_ROOT=/workspaces \
	MISE_TRUSTED_CONFIG_PATHS=/workspaces

WORKDIR /workspaces
