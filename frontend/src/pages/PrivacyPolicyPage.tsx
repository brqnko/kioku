export default function PrivacyPolicyPage() {
  return (
    <div class="bg-background-dark min-h-screen flex flex-col text-text-primary antialiased">
      <header class="sticky top-0 z-40 bg-background-dark/90 backdrop-blur-md h-14 border-b border-border-subtle flex items-center px-6 shrink-0">
        <a
          class="flex items-center gap-2 text-text-secondary hover:text-white transition-colors group cursor-pointer"
          href="/"
        >
          <span class="material-symbols-outlined text-[20px] group-hover:-translate-x-0.5 transition-transform">
            arrow_back
          </span>
          <span class="text-base font-medium">設定に戻る</span>
        </a>
        <div class="ml-auto text-xs text-text-secondary hidden sm:block">
          kioku Legal Documents
        </div>
      </header>

      <main class="flex-grow flex justify-center py-16 px-6">
        <article class="w-full max-w-[800px]">
          <header class="mb-16">
            <h1 class="text-[54px] leading-[1.04] font-bold text-white mb-4 tracking-tight">
              プライバシーポリシー
            </h1>
            <div class="flex items-center gap-4 border-b border-border-subtle pb-8">
              <p class="text-base text-text-secondary">効力発生日: 2023年11月1日</p>
              <span class="h-1 w-1 rounded-full bg-text-disabled" />
              <p class="text-base text-text-secondary">最終更新日: 2023年10月28日</p>
            </div>
          </header>

          <div class="space-y-8 text-base text-text-primary leading-relaxed">
            <section>
              <p class="mb-4">
                kioku（以下「当社」といいます）は、ユーザーの皆様のプライバシーを極めて重要視しています。本プライバシーポリシーは、当社のAI駆動型学習プラットフォーム（以下「本サービス」といいます）において、当社がどのような情報を収集し、それをどのように使用し、保護するかを明記するものです。
              </p>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                1. 収集する情報
              </h2>
              <p class="mb-2 text-text-secondary">
                本サービスを効果的に提供するため、以下の情報を収集します。
              </p>
              <ul class="list-disc list-outside ml-6 space-y-2 text-text-secondary">
                <li>
                  <strong class="text-white font-medium">アカウント情報:</strong>{" "}
                  氏名、メールアドレス、所属教育機関など、アカウント作成時に提供される情報。
                </li>
                <li>
                  <strong class="text-white font-medium">アップロードされた資料:</strong>{" "}
                  PDF、ドキュメント、画像など、AIによる分析や要約のためにユーザーが意図的にアップロードした学習資料。
                </li>
                <li>
                  <strong class="text-white font-medium">対話履歴:</strong>{" "}
                  AIアシスタントに対する質問、プロンプト、および生成された回答のログ（学習コンテキストの維持のため）。
                </li>
                <li>
                  <strong class="text-white font-medium">利用状況データ:</strong>{" "}
                  アクセスログ、デバイス情報、IPアドレスなど、システムのパフォーマンス監視とセキュリティ維持のための技術情報。
                </li>
              </ul>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                2. AIモデルとデータ使用に関する確約
              </h2>
              <p class="mb-2">
                当社は、教育ツールとしての性質上、ユーザーがアップロードする資料に機密性の高い情報が含まれる可能性を認識しています。当社は以下の原則を厳守します。
              </p>
              <div class="bg-surface-dark border border-border-subtle rounded-lg p-6 mt-4">
                <ul class="space-y-4">
                  <li class="flex items-start gap-4">
                    <span
                      class="material-symbols-outlined text-success mt-0.5"
                      style="font-variation-settings: 'FILL' 1;"
                    >
                      shield
                    </span>
                    <div>
                      <strong class="block text-white font-medium mb-1">
                        基盤モデルの学習への不使用
                      </strong>
                      <p class="text-text-secondary text-sm">
                        ユーザーがアップロードした個人的な学習資料、ノート、PDFを、kiokuの全体的なAI基盤モデルの学習データとして使用することはありません。
                      </p>
                    </div>
                  </li>
                  <li class="flex items-start gap-4">
                    <span
                      class="material-symbols-outlined text-accent-blue mt-0.5"
                      style="font-variation-settings: 'FILL' 1;"
                    >
                      lock
                    </span>
                    <div>
                      <strong class="block text-white font-medium mb-1">
                        テナントの分離
                      </strong>
                      <p class="text-text-secondary text-sm">
                        お客様のデータは論理的に分離され、他のお客様のAIがお客様の文書の内容に基づいて回答を生成することはありません。
                      </p>
                    </div>
                  </li>
                </ul>
              </div>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                3. 情報の共有と第三者提供
              </h2>
              <p class="mb-4 text-text-secondary">
                当社は、以下の場合を除き、ユーザーの個人情報やアップロードされたデータを第三者と共有、販売、または貸与することはありません。
              </p>
              <ul class="list-disc list-outside ml-6 space-y-2 text-text-secondary">
                <li>ユーザーの明示的な同意がある場合。</li>
                <li>
                  本サービスを提供する上で不可欠な外部のインフラストラクチャプロバイダー（クラウドホスティングや特定のAI
                  APIプロバイダー等）を利用する場合。これらのプロバイダーとは厳格なデータ処理契約を結び、データの目的外利用を禁止しています。
                </li>
                <li>法令に基づく要請があり、開示が義務付けられている場合。</li>
              </ul>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                4. データの保持と削除
              </h2>
              <p class="mb-4 text-text-secondary">
                当社は、ユーザーのアカウントが有効である期間、または本サービスを提供する目的において必要な期間に限り、ユーザーのデータを保持します。ユーザーは、アカウント設定画面からいつでもご自身のデータおよびアカウント自体の完全な削除をリクエストすることができます。削除リクエストを受領後、バックアップを含め、システムからデータは安全に消去されます。
              </p>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                5. ユーザーの権利
              </h2>
              <p class="mb-2 text-text-secondary">
                ユーザーは、自身のデータに関して以下の権利を有します。
              </p>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                <div class="border border-border-subtle rounded p-4 bg-background-dark">
                  <span class="material-symbols-outlined text-white mb-2 block">download</span>
                  <strong class="block text-white font-medium text-sm mb-1">
                    データのエクスポート
                  </strong>
                  <p class="text-text-disabled text-sm">
                    保存されたノートや対話履歴を標準的なフォーマットでダウンロードできます。
                  </p>
                </div>
                <div class="border border-border-subtle rounded p-4 bg-background-dark">
                  <span class="material-symbols-outlined text-danger mb-2 block">
                    delete_forever
                  </span>
                  <strong class="block text-white font-medium text-sm mb-1">完全な削除</strong>
                  <p class="text-text-disabled text-sm">
                    アップロードした特定のファイルやアカウント全体を即座に削除する権利。
                  </p>
                </div>
              </div>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                6. お問い合わせ
              </h2>
              <p class="mb-4 text-text-secondary">
                本プライバシーポリシーや当社のデータ取り扱いに関するご質問、懸念事項、または権利の行使については、以下の窓口までご連絡ください。
              </p>
              <div class="flex items-center gap-2 text-white">
                <span class="material-symbols-outlined text-text-secondary text-[20px]">
                  mail
                </span>
                <span class="text-base">privacy@kioku.example.com</span>
              </div>
            </section>
          </div>

          <div class="mt-16 pt-8 border-t border-border-subtle text-center">
            <p class="text-sm text-text-disabled">© 2023 kioku Inc. All rights reserved.</p>
          </div>
        </article>
      </main>
    </div>
  );
}
