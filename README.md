# kioku

## features

- pdf, md(notionみたいに編集可能)などをフォルダ構造として保存可能
    - 隣接グラフで保存。深さはmax 4まで
- AIによるpodcast機能(notebooklmt的な)
- RAG
    - AIに質問可能。ドキュメントベースの回答を生成
- アカウント設定
- 今日はどんな学習をしたのかをdashboardに表示するとか
- web searchとかも考えたが一旦保留

## tech

- Rust
    - Cloud Runで運用
    - 定期バッチはVPSからcronでHTTPリクエストを送信
    - Axum, sqlx,
- TiDB
    - ローカルではmariadbで代用
    - TiDB Cloudの無料プラン
- preach
    - Claude Code & Google stictchで雑に実装
    - VPSでnginxの静的ファイル配信
    - pwa, ssg, pre-rendering, swr, i18n-next, ky
    - https://stitch.withgoogle.com/projects/11840511067393172777
    -