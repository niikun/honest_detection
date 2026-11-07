# honest_detection 学習ハンドオフ

作成: 2026-10-28 / 最終更新: 2026-11-07
目的: 検知共有×評判ネットワークのシミュレーションを**一から作り**、Rust（型・所有権・テスト）と設計力を身につける。最終的に Axum API + Lovable(React) まで（期限 2026-12-31）。
関連: 秘書ノート `topics/verifiable-detection-reward-network.md`（構想）/ `topics/honest-detection-rust-api-roadmap.md`（ロードマップ）

> **別のPC・別のAIセッションはこのファイルから読む。** 読んだら「現在地」と「次の一歩」だけ確認して始めてよい。

## 原則（Claudeとの約束）

- **コードも環境設定も本人が書く**（`cargo add`、`Cargo.toml`、`git` も含む）。Claudeは書かない
- Claudeの役割は3つだけ: ①問いを出す ②本人の設計・コードをレビューする ③調べる場所を示す
- **答えより先に問い**。本人が案を出し、Claudeが穴を突く。一度に聞くのは1つ
- 動いただけで次へ進まない。各ステップの終わりに「自分の言葉で説明する」
- 詰まったら `HINTS.md` に質問を書く → ヒントを返す（答えは書かない）
- **やめてよい**。やめるときは理由を `NOTES.md` に1行残す
- 例外: 本人が「やって」と明示した作業（このHANDOFF作成など）はClaudeが行ってよい

## ゴールと完了条件

- ゴール: 3つの報酬条件（none / immediate / retro）で違反率がどう変わるかを、自作のシミュレーションで再現する
- 完了（v0.1）: `STEPS.md` のステップ1〜5。複数シード平均の表が出る
- その先: 台帳（ハッシュチェーン）→ 評判 → Axum API → Lovable（ロードマップのフェーズA〜D）

## 現在地

- 2026-10-28: `cargo new --lib` 済み。`STEPS.md`（6ステップ）と `HINTS.md`（空）あり。GitHub に push 済み（`origin`: niikun/honest_detection）
- 2026-10-29: **ステップ1・2 完了**。`Mode` は `Params` に入れず別 enum。2択 bandit（`Qvalue`・`get_action`・`update_qvalue`）が動き、`cargo test` 2件成功（`test_run` は罰なしで `violated_case >= compliant_case` かつ `< 1.0` を確認）。説明は `Note.md` に記録
- 積み残し: ①`test_run` は Q値の大小しか見ておらず、選んだ行動は未検証 ②`epsilon = 0.0` にしたときの挙動は未確認（探索の役割を理解するため、余裕があれば）
- 2026-10-30〜11-01: **ステップ3 完了**
  - `run(&Params) -> Qvalue` に一本化。`seed` も `Params` に入れ、`Qvalue::new(&mut rng)` も同じシード。`choose_action` を切り出し
  - 境界の検証: 期待利得 `1 - detect_p * p_viol`、境界 P=2。`p_viol` = 0 / 1.5 / 2.5 で Violated / Violated / Compliant（`test_run`）
  - 最終Q値の大小だと 1.5 で約3割のシードが予想と逆（200シード実測）。`run` が「各行動を選んだラウンドのQ値の平均」を返す形に変えたところ、200シードで食い違い 0
- 積み残し:
  1. `Note.md`「Q=1.5,Q=2.5だと不安定な理由」に誤りが残る: 報酬は p_viol=1.5 なら 1.0 か -0.5（-3 ではない）。Q値は `Q += (reward - Q) * 0.1` で報酬の方向に10%しか動かない。Q の上限・下限を書き直す
  2. （対応済み）平均は最初の半分を捨てて後半だけで取る（`epochs` を `Params` に入れ、`i > epochs/2`）。200シード実測で、`p_viol`=1.5/2.5 は食い違い 0、境界に近い 1.8/1.9/2.1/2.2 は 25/73/49/24。期待利得の差が小さいほど報酬のばらつきに埋もれるため。境界 P=2 ちょうどの挙動は未検証（進める判断をした）
  3. `run` が返す `Qvalue` は平均で、もうQ値そのものではない（名前・型の整理はステップ4〜5で）。ある行動が一度も選ばれないと `count == 0` で NaN になる
  4. `test_run` は seed 1つだけで確認。複数シードのテストはステップ5で
  5. ステップ2の積み残し（行動の検証、`epsilon = 0.0` の挙動）は継続
- 2026-11-01: **ステップ4 途中**（型の準備まで。`run` はまだ1体のまま）
  - 済: `Agent`（`qvalue`・`qreport`・通報/偽通報/無視のカウンタ）、`Qreport`（4つの値: report_seen / report_notseen / ignore_seen / ignore_notseen）、`Agents { Vec<Agent> }`、`Observed { Seen, Notseen }`、`Observe { item }`（見る相手の番号。自分は除外、`assert!(n != 1)`）、`ObserveList::new`（全員が相手を1人選ぶ）。`n` は `usize`
  - 決まったこと: 1ラウンドの順番は ①全員が守る/違反を選ぶ → ②他者を見る → ③通報/無視を選ぶ。各エージェントがランダムな1人を見る（全員が全員は N² で重いため）。相手が守っていたら状態は必ず「見ない」＝「見ない×通報」が偽通報
  - 問題: `ObserveList` の `observed_vec` が「見られた側」の添字に印を付けている。通報Qの状態は「見た側」ごとに必要。`detect_p` も、相手が実際に違反したかも、まだ使われていない
  - 小さな修正: `count_ignore` のメソッド名を `ignore` に。`n >= 2` を条件にする書き方もある
- 2026-11-02: **N体化を一度に書き換えて絡まり、コミット時点（`99ee02f`）に戻した**（コードは残っていない）。分かったことは残す:
  - 良かった案（書き直すときに再利用）: `status: Observed` を `Observe` の中に持たせる → 「i番目の Observe ＝ i が見た相手と i の状態」になり `observed_vec` が不要に。`Agent` に `get_action` / `update_qvalue` をメソッドとして持たせる
  - つまずき: ①ラウンドの順番が逆（見る→通報→行動を1人ずつ）②`ObserveList::new` を内側ループで毎回作り直した ③通報の判断に違反Q（`qvalue`）を使った（正しくは `qreport`）④`get_observe` に `&Agents` を渡したが、`Agent` は「このラウンドの行動」を持っていないので相手の行動が分からない ⑤`run` を書き換えて1体の `test_run` が壊れた（ステップ2・3の検証が消える）
  - Rust: E0596 に遭遇。`let mut a = &v[j]`（付け替え可）と `let a = &mut v[j]`（中身を書き換え可）は別物。Book 4.2「Mutable References」
  - 教訓: 整理しきれていないときに全体を書き換えない。小さく区切ってテストを通しながら進む
- 2026-11-03: **ステップ4 の ①② 完了**（小さく区切って、テストを通しながら進めた）。`cargo test` 5件成功（`61535a5`・`76a8196` でコミット済み）
  - 済①: `Agent` に `get_action` / `update_qvalue` をメソッド化（`epsilon` は `Params` へ）。1体用の `run` / `test_run` は残して通っている。`Agents::get_actions -> Actions { Vec<Action> }`（i番目＝i番の今ラウンドの行動）
  - 済②: `Observe { observer, observed, observe_status }`（`ObserveStatus { See, Notsee }`）。`Observe::new` が `&Actions` を受け取り、「確率 `detect_p` で当たり、かつ相手が違反」のときだけ `See`。相手の行動は `Agent` に持たせず、`&Actions` で渡す方式（B案）。見る相手は各自ランダムな1人（自分以外）。`Observes` が全員分を持つ
  - テスト: `test_get_observe_detectp1` / `detectp0`（n=2、守る/違反を固定して結果が確定）。`test_get_actions` の2つ目の assert は常に真で何も確かめていない（任意で直す）
  - 済: `Agent` を `src/agent.rs` に分離（`Qvalue`・`Qreport`・`Agent`・`Agents`）。`lib.rs` は `mod agent;` + `use agent::{...}`。`count_*` は非公開にして、`report` / `false_report` / `ignore`（+1）と読み取り用 `count_report()` などで操作。`ignore` へのメソッド名変更も済み
  - 小さな修正（任意）: `n >= 2` を条件にする書き方、`#[test]` のインデントのずれ、`lib.rs` の未使用 import（`Agents`・`Qreport`。③で使えば消える）
- 2026-11-04: **ステップ4 ③ 着手**（`ac6e6ce`・`e8fdbfe` でコミット済み。`cargo test` 5件成功）
  - 済: `Reaction { Report, Ignore }` / `Reactions { Vec<Reaction> }`（`Action`/`Actions` と同じ形）。`report_p`（通報確率の固定値）を入れかけたが削除する判断 → 通報は `qreport` で学習させる（固定確率だと報酬設計を変えても結果が変わらない）
  - 済: `Qreport` のフィールド名を `report_see` / `report_notsee` / `ignore_see` / `ignore_notsee` に統一。`update_qvalue` は rng ではなく報酬の値（`reward_action`）を受け取る形に（報酬の決め方と更新を分離）。`get_reward` は `(reward_action, reward_reaction)` を返す（`reward_reaction` は未実装で 0.0）
  - 書きかけ: `Agent::get_reaction`（`qreport` を状態で引く。良い）、`Agent::update_qreport`（中身なし）。`Agent` に `id` を追加
  - レビューで指摘した点:
    1. **所有権**: `get_reaction(self, ..., observes: Observes)` は `self` と `Observes` を値で受けるので、1回呼ぶと agent と observes が move する。N体ループで2人目から使えない。答えかけの問い「エージェント0の呼び出しの後、`observes` とエージェント0はどうなっているか？」（Book 4.1「Ownership and Functions」）
    2. **ε-greedy になっていない**: 常に大きい方を選ぶ（`params` 未使用の警告）。探索がないと最初に低かった行動を学び直せない
    3. **状態の渡し方が不揃い**: `get_reaction` は `Observes` 丸ごと＋`id` で探す、`update_qreport` は `&ObserveStatus`。自分の状態1つで足りるなら後者に揃える。そうすると `Agent.id` は Vec の添字と重複
    4. **（③-2 で扱う）違反者の罰**: `get_reward` はまだ1体用（確率 `detect_p` で罰）。N体では P_VIOL は「通報で確定したときの罰」＝誰かがその人を通報したかで決まる。`action` と `reaction` を並べて渡す今の `get_reward` の形で計算できるか？
- 2026-11-04: 最終的な動作環境の候補として `blockchaininnovation/2026gw-amnesia`（Twin Towns・非公開）を確認。Node 22 + pnpm / Foundry(Anvil) / TypeScript モノレポで、Rust は含まれない。honest_detection をどう載せるかは未決
  - 方針（案）: honest_detection は Axum API（通報・評判サービス）として別に立て、Twin Towns から HTTP で呼ぶ。Rust のまま。ステップ4〜5を先に終わらせ、API はその後
  - 呼ぶ場所: `apps/server/src/runner/` か `packages/ledger`。`packages/sim` は純粋関数（I/O禁止）なので呼ばない。街ごとの違いは台帳と Observation（`townNews`）だけに閉じ込める
  - API の最初の口: ①通報を受け取る ②住民ごとの評判を返す。最初はローカル別ポート。AWS（`infra/` の CDK）は最後
  - 未決: 評判を住民の行動に影響させるか、画面表示だけか。どの出来事を API に送るか（違反の目撃・通報など）
  - 2026-11-06 確認: amnesia の場所は PC ごとに違う（メイン: `~/project/2026gw-amnesia`、サブ: `/home/kawata/project/2026gw-amnesia`）。既存サーバーは Fastify で 8787 番、画面は 5173 番 → Axum は別ポートにする
  - 2026-11-06 確認: Twin Towns には「違反」「通報」「評判」にあたるものがまだない（`docs/`・`packages/sim`・`packages/ledger`・`apps` を検索して該当なし）。**API を作る前に「この街で何を違反とするか」を決める必要がある**（お金の移動・土地・投票など、住民の実際の行動のどれに対応させるか）。台帳に足すなら `LedgerEvent`（`tick` と `type` を持つ形）に合わせる。検討は v0.1 の後でよい
- 2026-11-05: **③-1 完了、③-2 ほぼ完了**（未コミット。`cargo test` 6件成功）
  - ③-1: `get_reaction(&self, params, &Observes, rng)` に修正（所有権の問題を解消）。ε-greedy を入れた（探索 → See → Notsee の `if / else if / else`）。`Agents::get_reactions -> Reactions`（`get_actions` と同じ形、`rng` をクロージャに渡す）
  - テスト `test_get_reactions`: n=4、`epsilon = 0.0`、Q値を1か所ずつ 1.0、**状態を4人分手で固定**（最初は `Observes::new` の乱数任せで偶然通っていた → 修正）。任意: 比べる相手も 0.0 と明示すると初期化方法に依存しない
  - ③-2: `update_qreport` を実装。状態で `match` → 反応で4フィールドに振り分け（`&Reaction` と `Reaction` の比較エラーは `&Reaction::Report` で解消）。自分の状態は `observes.iter().find(|o| o.observer == id).unwrap()` で探す形（`get_reaction` は `observes[id]` の添字。どちらかに揃えるとよい）
  - 指摘3（状態の渡し方）は「`Observes` + `id`」で揃える方向で決着
  - 任意: `match (状態, 反応)` のタプル match にすると4通りが表の形で並び、書き忘れをコンパイラが検出（Book 6.2 / 19章）
  - 未確認: テストの `epsilon = 0.0` を外すと失敗するシードがあるか（ε-greedy が効いている確認）
  - 自分の言葉で説明（ステップ4の締めで `Note.md` に）: `self` / `Observes` を値で受けると、なぜ2人目の呼び出しでエラーになるか（所有権・move・借用）
- 2026-11-07: **③-2 `update_qreport` のテスト完了、報酬設計（none / retro）完了**（`cargo test` 8件成功。`src/lib.rs` は未コミット）
  - 済: `test_update_qreport` を4通り（状態2×反応2）に拡張。失敗の原因は `agent` の使い回し → ループ内で毎回作り直して解決
  - 決まったこと: ①none は通報者に `-COST` のみ（嘘への罰 `P_FALSE` は retro だけ。偽通報かどうかで none の報酬は変えない）②違反者の罰 `P_VIOL` は通報者が何人でも1回だけ（「j が通報された」＝ j を `See` した人のうち1人でも `Report`。`any` で bool）③「通報されたか」の bool は関数の**外**で計算して渡す（retro と場合分けしやすい）④1体用 `get_reward` / `run` は触らず、N体用の新しい関数を別に作る（自由関数。`Agent` のメソッドにしない: `self` を使わないため）
  - 済（報酬）: `ReportState { Ignore, ReportTrue, ReportFalse }` / `ReportStates`（i番目＝i番の通報状態）。`Params` に `r_true`（1.0）/`p_false`（1.0）を追加。`get_report_reward(params, &ReportStates, &Observes) -> Vec<(f64, f64)>`（戻りは（通報者の報酬, 違反者の罰）、i番目＝i番）。通報者は none が `-cost`、retro が本物なら `-cost + r_true`、偽物なら `-cost - p_false`。違反者の罰は「j を `See` した人の誰かが `ReportTrue`」のとき `-p_viol` を1回だけ（`observed` に印を付ける形。印は被害者の添字）
  - テスト `test_get_report_reward`: n=3、none と retro の両方（期待値は mode ごとに紙で計算して書く）。`default()` の `p_viol` を 3.0（モデルの値）にしたため、`test_run` 側で `params1.p_viol = 0.0` を明示（「罰なし」の基準）。これで未決の `default` 問題は決着
  - つまずきと解決: ①`observed_rewards` を `states[j]`（j自身の通報状態）と並べていた → 「j が通報されたか」は `observed[j]` で決まるので `match` は不要に ②印を付ける条件に「見た人が通報した」が抜けていた → `observes` と `states` を `zip` ③テストが `p_viol=0` で罰を検出できなかった
  - 未実装・未決: `Mode::Immediate`（R_IMM）の分岐は 0.0 のまま（else と同じ）。`SPAM_P` の定義は未決。浮動小数点の `assert_eq!` は今は通るが、値が増えたら誤差付き比較を調べる
- 次の一歩（ステップ4 ③）:
  1. `Reactions` と `Observes` から `ReportStates` を作る関数（Report × See → `ReportTrue`、Report × Notsee → `ReportFalse`、Ignore → `Ignore`）。i 番目の `Observe` ＝ i 番の見た側、という順序の前提をテストかコメントで残す。小さなテスト（n=3 の例）から
  2. N体のラウンドを別の関数にする（①全員が行動 → ②見る → ③通報/無視 → `ReportStates` → `get_report_reward` → 各エージェントの Q 値を更新）。違反者の報酬 `gain` も忘れずに（違反＝`gain`、通報されたら `-p_viol`）。1体用の `run` は触らない
  3. テスト: none 条件で通報がほぼゼロになること
  4. 余力があれば `Mode::Immediate`（R_IMM）の分岐を足す（ステップ5の3条件比較で必要）
- amnesia との接続: 違反・通報・評判は Twin Towns にまだない。「この街で何を違反とするか」は未決（v0.1 の後で検討）
- 見積もり（11/5 時点）: v0.1（ステップ5完了）まであと5〜8回。11/12〜12/10 は応用編2と並走。年内の範囲（v0.1＋API の最初の口まで、など）はステップ4終了時に見直す

## 参考実装（見るのは各ステップを書き終えてから）

- `~/project/honest_detection_rs`: Claudeが先に書いた移植版（Python版 `~/project/honest_detection_sim/sim.py` の移植）。**この環境は別PCにはない**。完成形の結果は下のとおり

期待される結果（3000ラウンド×20シード、末尾20%平均、N=10）:

| 条件 | 違反率 | 偽通報/R | 総損害/R |
|---|---|---|---|
| none | 0.98 | 0.4 | 26.6 |
| immediate | 0.98 | 22.5 | 26.6 |
| retro | 0.02 | 0.7 | 0.4 |

モデルの要点（自作時に合わせる定数）: GAIN=1.0、HARM=0.3、DETECT_P=0.5、SPAM_P=0.5、COST=0.2、R_IMM=0.3、R_TRUE=1.0、P_FALSE=1.0、P_VIOL=3.0、ALPHA=0.1、EPS_START=0.3（0.02まで線形減衰）。独立Q学習。違反Qは状態なし、通報Qは状態（見た/見ない）×行動（無視/通報）。詳しくは下の「用語集」。

## 用語集

### 設定の全体像

N人（N=10）が毎ラウンド「守る/違反する」を選ぶ。違反は他人に目撃されることがあり、見た人（見ていない人も）は「通報/無視」を選ぶ。各エージェントは試行錯誤で得な行動を学ぶ。**通報者への報われ方（Mode）だけを変えて**、集団の違反率がどう変わるかを見る。

### Mode（3条件）

| Mode | 通報者に起きること |
|---|---|
| none | 何ももらえない。コストだけかかる |
| immediate | 通報した瞬間に R_IMM。真偽は確かめない |
| retro（retroactive＝遡及） | 後で照合する。本物なら R_TRUE、嘘なら P_FALSE の罰 |

### 定数

| 関係する人 | 定数 | 意味 |
|---|---|---|
| 違反する人 | GAIN = 1.0 | 違反で得る得（守る=0） |
| | HARM = 0.3 | 違反が他人に与える損害（結果表の「総損害」） |
| | P_VIOL = 3.0 | 違反が通報で確定したときの罰 |
| 見る人 | DETECT_P = 0.5 | 他人の違反を目撃できる確率 |
| | COST = 0.2 | 通報の手間 |
| | SPAM_P = 0.5 | **要確認**。偽通報（見ていないのに通報）に関わる確率と推測されるが、定義は未確認。元実装のあるPCで確かめるか、自分で決める |
| 通報の報酬 | R_IMM = 0.3 | immediate：通報の瞬間にもらえる額 |
| | R_TRUE = 1.0 | retro：本物の通報への報酬 |
| | P_FALSE = 1.0 | retro：嘘の通報への罰 |
| 学習 | ALPHA = 0.1 | 学習率 |
| | EPS_START = 0.3 | 探索率の初期値（0.02まで線形減衰） |

### 学習の用語

- **Q値**: 行動ごとの「これをやるとどれくらい得か」の見込み。経験で少しずつ直す
- **ALPHA（学習率）**: 見込みと実際のズレのうち、どれだけ見込みを動かすか。0.1 ならズレの10%
- **ε-greedy / EPS（探索率）**: 確率 ε でランダムに試し、それ以外は Q値が最大の行動を選ぶ。0.3 から始めて、ラウンドが進むにつれ一定ペースで 0.02 まで下げる（線形減衰）
- **独立Q学習**: 各エージェントが自分のQ表だけを持ち、他人のことは考えずに学習する
- **違反Q（状態なし）**: 「守る」「違反」の2つのQ値だけ。ステップ2の bandit がこれ
- **通報Q（状態×行動）**: 2×2 の表。「見ない」の行で「通報」を選ぶ＝偽通報

  |  | 無視 | 通報 |
  |---|---|---|
  | 見た | Q | Q |
  | 見ない | Q | Q |

- **シード（seed）**: 乱数の出発点。同じシードなら毎回同じ乱数の並びになり、結果を再現できる。結果表は20シードの平均

## 変数一覧（意味は自分で埋める）

定数（実験のたびに変える/固定するかはステップ1で決める）:

| 名前 | 値 | 出どころ | 意味（自分の言葉で） |
|---|---|---|---|
| GAIN | 1.0 | モデルの要点 | |
| HARM | 0.3 | モデルの要点 | |
| DETECT_P | 0.5 | モデルの要点 | |
| SPAM_P | 0.5 | モデルの要点 | |
| COST | 0.2 | モデルの要点 | |
| R_IMM | 0.3 | モデルの要点 | |
| R_TRUE | 1.0 | モデルの要点 | |
| P_FALSE | 1.0 | モデルの要点 | |
| P_VIOL | 3.0 | モデルの要点（STEPS では `P`） | |
| ALPHA | 0.1 | モデルの要点 | |
| EPS_START | 0.3 | モデルの要点（STEPS では `EPS`） | |
| EPS_END | 0.02 | 「0.02まで線形減衰」（名前は仮） | |
| N | 10 | 結果表の条件 | |
| ラウンド数 | 3000 | 結果表の条件（名前は未定） | |
| シード数 | 20 | 結果表の条件（名前は未定） | |
| 集計範囲 | 末尾20% | 結果表の条件（名前は未定） | |

学習で変わる変数: 違反Q（状態なし×守る/違反）、通報Q（見た/見ない×無視/通報）、現在の ε
結果の指標: 違反率、偽通報/R、総損害/R（R=ラウンド）
未決: `mode`、乱数シード

## 複数PCで作業するときの手順

- 作業の終わり: この「現在地」と「次の一歩」を更新 → `git add` / `commit` / `push`（**本人が行う**）
- 作業の始まり: `git pull` → このファイルを読む。Claudeには「HANDOFF.md を読んで続きから」と言う
- 秘書ノート（`company` リポジトリ）も別PCに同期が必要。ここに書いた要点だけで動けるようにしてある
- `target/` は `.gitignore` 済み。別PCでは初回 `cargo build` が走る

## 最近の判断

- 2026-10-28: ZK/SP1・分散台帳より先に、軽量シミュレーションでメカニズムを検証する
- 2026-10-28: 評判は台帳の検証済みイベントだけから計算する（投票は入れない）
- 2026-10-28: 学習用。11/12〜12/10 の応用編2と並走するため、週2〜3時間が目安

## 未決

- ~~`mode` を `Params` に入れるか~~ → 外に出す（2026-10-28 決定）
- ~~シードを `Params` に入れるか~~ → `Params` に入れた（ステップ3）
- ~~`Params::default()` の `p_viol`~~ → モデルの値（3.0）にした。`test_run` が「罰なし」を使う箇所は `p_viol = 0.0` を明示（2026-11-07）
- 乱数: `rand` 0.10 の API（`RngExt`）。`StdRng` + `SeedableRng::seed_from_u64` で固定済み（2026-10-30）
- SPAM_P の正確な定義（用語集参照。**ステップ4 ③ の「報酬を決める」で決める**）
