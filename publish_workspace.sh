#!/usr/bin/env bash

set -e   # 任意命令失败立即退出

ROOT_DIR=$(pwd)

echo "📦 Rust workspace 自动 dry-run/publish（失败立即停止）"
echo ""

# -----------------------------
# 获取 workspace crates
# -----------------------------
echo "📚 获取 workspace crate 列表..."

CRATES=$(cargo metadata --no-deps --format-version=1 \
    | jq -r '.packages[] | select(.source == null) | .manifest_path')

echo "🧩 发现以下 crates："
echo "$CRATES"
echo ""

# -----------------------------
# 遍历每个 crate（失败立即退出）
# -----------------------------
for MANIFEST in $CRATES; do
    DIR=$(dirname "$MANIFEST")
    NAME=$(grep -E '^\s*name\s*=' "$MANIFEST" | head -n1 | sed 's/.*= *"//; s/"//')

    echo ""
    echo "=============================="
    echo "📦 crate: $NAME"
    echo "📁 path : $DIR"
    echo "=============================="

    cd "$DIR"

    echo "🧪 执行 dry-run..."

    # 捕获错误输出
    if ! OUTPUT=$(cargo publish --dry-run 2>&1); then
        echo "❌ dry-run 失败：$NAME"
        echo "   👉 错误信息："
        echo "$OUTPUT"
        exit 1
    fi

    echo "✔ dry-run 成功：$NAME"

    # 是否发布
    read -p "发布 $NAME 到 crates.io？[y/N] " pub
    if [[ "$pub" =~ ^[yY]$ ]]; then
        echo "🚀 正在发布 $NAME ..."

        if ! OUTPUT=$(cargo publish 2>&1); then
            echo "❌ 发布失败：$NAME"
            echo "   👉 错误信息："
            echo "$OUTPUT"
            exit 1
        fi

        echo "⏳ 等待 crates.io 同步 10 秒..."
        sleep 10
    else
        echo "⏭️ 跳过发布 $NAME"
    fi

    cd "$ROOT_DIR"
done

echo ""
echo "🎉 所有 crates 处理完毕"
