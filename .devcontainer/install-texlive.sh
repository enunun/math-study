#!/bin/sh
# TeX Liveを，公式のinstall-tlで入れる．TikZの出力を，LuaLaTeXでコンパイルして確かめるために使う．
#
# 入れるもの：LaTeXとLuaLaTeXの本体，pgf(TikZ)，standalone，日本語の組版(LuaTeX-ja)と原ノ味フォント．
# 版は，install-tlが決める最新のリリースである．入れた後の各パッケージの版は，tlmgr info --only-installedで見られる．
# Dockerfileから呼ぶほか，実行中のコンテナで，直接実行してもよい(rootで実行する)．
set -eu

MIRROR="${TEXLIVE_MIRROR:-https://mirror.ctan.org/systems/texlive/tlnet}"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

curl -fsSL "$MIRROR/install-tl-unx.tar.gz" -o "$WORK/install-tl.tar.gz"
tar -xzf "$WORK/install-tl.tar.gz" -C "$WORK"
INSTALLER="$(find "$WORK" -maxdepth 1 -type d -name 'install-tl-*' | head -n 1)"
YEAR="$(sed -n 's/^our \$ReleaseYear *= *\([0-9]*\);.*/\1/p' "$INSTALLER/tlpkg/TeXLive/TLConfig.pm")"
ROOT="/opt/texlive/$YEAR"

# 文書と，ソースは入れない．インストール先の下に，設定と一時のファイルを置く．
cat > "$WORK/texlive.profile" <<PROFILE
selected_scheme scheme-infraonly
TEXDIR $ROOT
TEXMFLOCAL /opt/texlive/texmf-local
TEXMFSYSCONFIG $ROOT/texmf-config
TEXMFSYSVAR $ROOT/texmf-var
TEXMFCONFIG ~/.texlive$YEAR/texmf-config
TEXMFVAR ~/.texlive$YEAR/texmf-var
TEXMFHOME ~/texmf
instopt_adjustpath 0
instopt_adjustrepo 1
instopt_letter 0
instopt_portable 0
instopt_write18_restricted 1
tlpdbopt_autobackup 0
tlpdbopt_create_formats 1
tlpdbopt_desktop_integration 0
tlpdbopt_file_assocs 0
tlpdbopt_generate_updmap 0
tlpdbopt_install_docfiles 0
tlpdbopt_install_srcfiles 0
tlpdbopt_post_code 1
tlpdbopt_sys_bin /usr/local/bin
tlpdbopt_sys_info /usr/local/share/info
tlpdbopt_sys_man /usr/local/share/man
PROFILE

perl "$INSTALLER/install-tl" --profile="$WORK/texlive.profile" --no-interaction --repository "$MIRROR"

TLMGR="$ROOT/bin/x86_64-linux/tlmgr"
"$TLMGR" install \
	collection-basic \
	collection-latex \
	collection-luatex \
	pgf standalone xkeyval xcolor \
	luatexja haranoaji \
	fontspec unicode-math lm lm-math luaotfload luatexbase lualibs
# 実行ファイルを，/usr/local/binに写す(リンクを張る)．
"$TLMGR" path add
