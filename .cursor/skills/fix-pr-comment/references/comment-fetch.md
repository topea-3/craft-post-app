# PR コメント取得（gh）

## URL パース

| フラグメント | 種別 | ID の取り方 |
|--------------|------|-------------|
| `#discussion_r123` | インラインレビューコメント | `123` |
| `#issuecomment-123` | PR 会話（Issue コメント） | `123` |
| `#pullrequestreview-123` | レビュー単位 | `123` |

共通: `https://github.com/<owner>/<repo>/pull/<n>...` から `owner` / `repo` / `n` を取る。

## インライン（`discussion_r`）

対象コメント:

```bash
gh api repos/<owner>/<repo>/pulls/comments/<comment_id>
```

同一スレッド（親 + 返信）。`parent` を対象 ID、または `in_reply_to_id` のルート ID にする:

```bash
gh api repos/<owner>/<repo>/pulls/<n>/comments --paginate \
  --jq '[.[] | select(.id == <parent> or .in_reply_to_id == <parent>)]'
```

返信の場合は先に対象を取得し、`in_reply_to_id // .id` をスレッド親にする。

## 会話コメント（`issuecomment-`）

```bash
gh api repos/<owner>/<repo>/issues/comments/<comment_id>
```

スレッド相当の返信は Issue コメントではフラットなため、同一 PR の会話一覧から文脈を補う:

```bash
gh api repos/<owner>/<repo>/issues/<n>/comments --paginate
```

## レビュー単位（`pullrequestreview-`）

```bash
gh api repos/<owner>/<repo>/pulls/<n>/reviews/<review_id>
gh api repos/<owner>/<repo>/pulls/<n>/reviews/<review_id>/comments --paginate
```

レビュー本文だけの指摘（インラインコメント無し）は、PR 会話への通常コメントで返信する。

## GraphQL（解決状態・thread id が必要なとき）

REST のコメント ID は GraphQL の `databaseId` に対応する。Resolve 用の `thread.id`（`PRRT_...`）もここから取る。

```bash
gh api graphql -f query='
query($o:String!,$r:String!,$n:Int!){
  repository(owner:$o,name:$r){
    pullRequest(number:$n){
      reviewThreads(first:100){
        nodes{
          id isResolved isOutdated
          comments(first:50){
            nodes{ databaseId body author{login} path line }
          }
        }
      }
    }
  }
}' -f o=<owner> -f r=<repo> -F n=<n>
```

返信投稿と Resolve の手順は [comment-reply-resolve.md](comment-reply-resolve.md) を参照する。
