# PR コメント返信と Resolve（gh）

Push 成功後（または修正が既に remote にあるとき）に、**今回対応した該当コメント**へ返信し、可能なスレッドは Resolve する。

## 対象の決め方

| URL 種別 | 返信先 | Resolve |
|----------|--------|---------|
| `#discussion_r<id>` | そのコメント（またはスレッド親）への reply | そのスレッドを Resolve |
| `#pullrequestreview-<id>` | レビューに紐づく**今回対応した**各インラインコメントへ reply。レビュー本文のみの指摘なら PR 会話へコメント | 対応した各インラインスレッドを Resolve |
| `#issuecomment-<id>` | 同一 PR の Issue コメントとして返信（引用で対象を明示） | なし（Resolve API 無し） |

- 無関係な未解決スレッドは触らない
- 「修正しない」と方針決定したものも、理由を短く返信してから Resolve してよい（放置しない）

## 返信本文

短く具体的に書く。目安:

- 対応した場合: 何をどう直したか + コミット SHA（短ハッシュ可）
- 対応しない場合: 理由を 1–2 文
- 複数指摘を 1 コミットで直した場合: スレッドごとにその指摘分だけ述べる

## インラインコメントへ返信

```bash
gh api repos/<owner>/<repo>/pulls/<n>/comments/<comment_id>/replies \
  -f body='<返信本文>'
```

`<comment_id>` はスレッド内のどのコメントでもよいが、原則は対象となったレビューコメントの REST `id`（`discussion_r` の数字）を使う。

## レビュー単位

1. レビューのコメント一覧を取得（[comment-fetch.md](comment-fetch.md)）
2. 今回の方針・差分で対応したコメントだけに reply
3. それぞれ Resolve

## Issue コメントへ返信

Issue コメントに thread reply API は無いので、PR 会話に新規コメントする:

```bash
gh api repos/<owner>/<repo>/issues/<n>/comments \
  -f body='<返信本文>'
```

本文先頭で対象を示す（例: `> 元コメントへの対応` や対象 URL）。

## スレッドを Resolve

GraphQL の `thread.id`（`PRRT_...`）が必要。REST の comment id（`databaseId`）からスレッドを引く:

```bash
gh api graphql -f query='
query($o:String!,$r:String!,$n:Int!){
  repository(owner:$o,name:$r){
    pullRequest(number:$n){
      reviewThreads(first:100){
        nodes{
          id isResolved
          comments(first:50){ nodes{ databaseId } }
        }
      }
    }
  }
}' -f o=<owner> -f r=<repo> -F n=<n> \
  --jq '.data.repository.pullRequest.reviewThreads.nodes[] | select(.comments.nodes[].databaseId == <comment_id>) | {id, isResolved}'
```

Resolve:

```bash
gh api graphql -f query='
mutation($id:ID!){
  resolveReviewThread(input:{threadId:$id}){
    thread{ id isResolved }
  }
}' -f id=<thread_node_id>
```

既に `isResolved: true` なら Resolve 呼び出しはスキップし、返信だけ行う（二重操作しない）。

## 失敗時

- 返信または Resolve が権限エラー等で失敗したら、成功分と失敗分をサマリに分け、ユーザーへ再実行や権限確認を促す
- コードの Push 自体は成功扱いのままにしてよい（Step 6 だけリトライ可能）
