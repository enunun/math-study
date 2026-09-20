# mise公式のDebianイメージ．git，curl，CA証明書とmiseを含み，
# PATHに/mise/shims，MISE_DATA_DIRに/miseが設定済み．
FROM jdxcode/mise:2026.9.12-debian

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	openssh-client \
	procps \
	&& rm -rf /var/lib/apt/lists/*

# ツール(Node，pnpm，gh，rtkなど)はmise.tomlとmise.lockのとおりにイメージへ入れる．
WORKDIR /tmp/mise-bootstrap
COPY mise.toml mise.lock ./
RUN mise trust \
	&& mise install --locked \
	&& rm -rf /mise/cache /tmp/mise-bootstrap

# 起動後は，リポジトリのmise.tomlをグローバル設定として読む．
# これにより，/workspacesの外のディレクトリでも同じツールを使える．
# グローバル設定のルートも/workspacesにして，タスクをリポジトリ直下で実行させる
# (未指定だとホームディレクトリで実行される)．
ENV MISE_GLOBAL_CONFIG_FILE=/workspaces/mise.toml \
	MISE_GLOBAL_CONFIG_ROOT=/workspaces \
	MISE_TRUSTED_CONFIG_PATHS=/workspaces

WORKDIR /workspaces
