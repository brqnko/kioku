export default function TOSPage() {
  return (
    <div class="bg-background-dark text-text-primary min-h-screen flex flex-col">
      <header class="sticky top-0 z-50 bg-background-dark/80 backdrop-blur-md border-b border-border-subtle w-full h-14 flex items-center px-6">
        <div class="flex items-center gap-2">
          <span class="text-[22px] font-bold text-text-primary tracking-tight">kioku</span>
          <span class="text-text-disabled mx-2">/</span>
          <span class="text-[14px] text-text-secondary">法務・ポリシー</span>
        </div>
        <div class="ml-auto">
          <a
            href="/"
            class="flex items-center gap-1 text-[14px] text-text-secondary hover:text-text-primary transition-colors px-4 py-2 rounded hover:bg-surface-dark"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
            <span>閉じる</span>
          </a>
        </div>
      </header>

      <div class="flex-1 flex w-full">
        <main class="flex-1 w-full max-w-3xl px-6 py-16 mx-auto">
          <div class="mb-16">
            <div class="flex items-center gap-2 mb-4">
              <span class="material-symbols-outlined text-text-disabled text-[32px]">gavel</span>
            </div>
            <h1 class="text-[54px] leading-[1.04] font-bold text-text-primary mb-2">利用規約</h1>
            <p class="text-[14px] text-text-disabled">最終更新日：2023年10月27日</p>
          </div>

          <div class="space-y-8 text-text-secondary text-base">
            <section id="section-1" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                1. はじめに
              </h2>
              <p class="mb-4">
                kioku（以下「本サービス」）へようこそ。この利用規約（以下「本規約」）は、ユーザーの皆様が本サービスを利用する際の条件を定めるものです。本サービスにアクセスまたは利用することにより、ユーザーは本規約のすべての条項に同意したものとみなされます。
              </p>
              <p>
                本規約に同意いただけない場合は、本サービスの利用を直ちに中止してください。当社は、事前の通知なく本規約を変更する権利を留保します。
              </p>
            </section>

            <section id="section-2" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                2. 定義
              </h2>
              <p class="mb-4">本規約において使用する用語の定義は、以下のとおりとします。</p>
              <ul class="list-disc pl-6 space-y-2">
                <li>
                  <strong>「当社」</strong>とは、kiokuを運営する事業者を指します。
                </li>
                <li>
                  <strong>「ユーザー」</strong>
                  とは、本サービスに登録し、または本サービスを利用するすべての個人および法人を指します。
                </li>
                <li>
                  <strong>「コンテンツ」</strong>
                  とは、テキスト、音声、画像、データなど、本サービスを通じて提供されるすべての情報を指します。
                </li>
                <li>
                  <strong>「AIモデル」</strong>
                  とは、当社が本サービス内で提供する人工知能技術およびアルゴリズムを指します。
                </li>
              </ul>
            </section>

            <section id="section-3" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                3. サービスの利用
              </h2>
              <p class="mb-4">
                ユーザーは、本規約および関連する法令を遵守することを条件として、本サービスを利用するための非独占的、譲渡不可、取消可能なライセンスを付与されます。
              </p>
              <p class="mb-2 text-text-primary font-medium">禁止事項</p>
              <p class="mb-4">ユーザーは、以下の行為を行ってはなりません。</p>
              <ul class="list-disc pl-6 space-y-2">
                <li>法令、裁判所の判決、決定もしくは命令、または法令上拘束力のある行政措置に違反する行為。</li>
                <li>公の秩序または善良の風俗を害するおそれのある行為。</li>
                <li>
                  当社または第三者の著作権、商標権、特許権等の知的財産権、名誉権、プライバシー権、その他法令上または契約上の権利を侵害する行為。
                </li>
                <li>
                  本サービスのサーバーやネットワークシステムに支障を与える行為、BOT、チートツール、その他の技術的手段を利用してサービスを不正に操作する行為。
                </li>
                <li>AIモデルを利用して、有害、差別的、または違法なコンテンツを生成する行為。</li>
              </ul>
            </section>

            <section id="section-4" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                4. ユーザーアカウント
              </h2>
              <p class="mb-4">
                本サービスの一部機能を利用するためには、アカウントの登録が必要となる場合があります。ユーザーは、真実かつ正確な情報を提供し、常に最新の状態に保つ責任を負います。
              </p>
              <div class="bg-surface-dark border border-border-subtle rounded-lg p-6 mt-4">
                <span class="material-symbols-outlined text-accent-blue mb-2 block">security</span>
                <p class="text-[14px] text-text-secondary">
                  アカウントのパスワード等の管理不十分、使用上の過誤、第三者の使用等による損害の責任はユーザーが負うものとし、当社は一切責任を負いません。アカウントの不正利用が発覚した場合は、直ちに当社にご連絡ください。
                </p>
              </div>
            </section>

            <section id="section-5" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                5. プライバシーとデータ
              </h2>
              <p class="mb-4">
                当社は、ユーザーのプライバシーを尊重し、個人情報を適切に取り扱います。個人情報の収集、利用、共有に関する詳細は、別途定める「プライバシーポリシー」をご参照ください。
              </p>
              <p>
                ユーザーが本サービスに入力したデータ（プロンプトやアップロードしたファイル等）は、サービスの提供およびAIモデルの品質向上のために利用される場合があります。機密情報や個人を特定できる機微な情報を入力することは推奨されません。
              </p>
            </section>

            <section id="section-6" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                6. 知的財産
              </h2>
              <p class="mb-4">
                本サービスおよび本サービスに関連する一切のプログラム、ソフトウェア、商標、ロゴマーク等に関する知的財産権は、当社または正当な権利を有する第三者に帰属します。
              </p>
              <p>
                ユーザーが本サービスを利用して生成したコンテンツの著作権等の取り扱いについては、生成に利用された元データの権利関係や該当する国の法令に準拠するものとします。当社は、ユーザーが生成したコンテンツに対して所有権を主張しませんが、サービスの運営に必要な範囲でこれを利用する権利を付与されたものとします。
              </p>
            </section>
          </div>

          <div class="mt-16 pt-6 border-t border-border-subtle text-center">
            <p class="text-[12px] text-text-disabled">© 2023 kioku. All rights reserved.</p>
          </div>
        </main>
      </div>
    </div>
  );
}
