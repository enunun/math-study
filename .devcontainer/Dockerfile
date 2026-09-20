FROM ubuntu:latest

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
	git \
	curl \
	openssh-client \
	ca-certificates \
	libpq-dev \
	wget \
	&& rm -rf /var/lib/apt/lists/*

# GitHub公式ホストのaptリポジトリ（cli.github.com）からghコマンドを導入する。
# DebianディストリビューションのaptリポジトリにあるghはGitHub CLIとは別物のため使用しない。
RUN mkdir -p -m 755 /etc/apt/keyrings \
	&& out=$(mktemp) && wget -nv -O"$out" https://cli.github.com/packages/githubcli-archive-keyring.gpg \
	&& cat "$out" | tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
	&& chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
	&& mkdir -p -m 755 /etc/apt/sources.list.d \
	&& echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
	&& apt-get update \
	&& apt-get install -y --no-install-recommends gh \
	&& rm -rf /var/lib/apt/lists/*

# rtkの導入
ARG RTK_VERSION=0.45.0

RUN set -eux; \
	case "$(dpkg --print-architecture)" in \
	amd64) rtk_target="x86_64-unknown-linux-musl" ;; \
	arm64) rtk_target="aarch64-unknown-linux-gnu" ;; \
	*) echo "Unsupported architecture" >&2; exit 1 ;; \
	esac; \
	curl -fsSL "https://github.com/rtk-ai/rtk/releases/download/v${RTK_VERSION}/rtk-${rtk_target}.tar.gz" \
	| tar -xz -C /usr/local/bin rtk; \
	chmod +x /usr/local/bin/rtk

WORKDIR /workspaces