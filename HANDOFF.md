# honest_detection 学習ハンドオフ

作成: 2026-10-28 / 最終更新: 2026-11-20
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
  1. （対応済み 11/13）`Note.md`「Q=1.5,Q=2.5だと不安定な理由」に誤りが残る: 報酬は p_viol=1.5 なら 1.0 か -0.5（-3 ではない）。Q値は `Q += (reward - Q) * 0.1` で報酬の方向に10%しか動かない。Q の上限・下限を書き直す
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
- 2026-11-05: **③-1 完了、③-2 ほぼ完了**（コミット済み。`cargo test` 6件成功）
  - ③-1: `get_reaction(&self, params, &Observes, rng)` に修正（所有権の問題を解消）。ε-greedy を入れた（探索 → See → Notsee の `if / else if / else`）。`Agents::get_reactions -> Reactions`（`get_actions` と同じ形、`rng` をクロージャに渡す）
  - テスト `test_get_reactions`: n=4、`epsilon = 0.0`、Q値を1か所ずつ 1.0、**状態を4人分手で固定**（最初は `Observes::new` の乱数任せで偶然通っていた → 修正）。任意: 比べる相手も 0.0 と明示すると初期化方法に依存しない
  - ③-2: `update_qreport` を実装。状態で `match` → 反応で4フィールドに振り分け（`&Reaction` と `Reaction` の比較エラーは `&Reaction::Report` で解消）。自分の状態は `observes.iter().find(|o| o.observer == id).unwrap()` で探す形（`get_reaction` は `observes[id]` の添字。どちらかに揃えるとよい）
  - 指摘3（状態の渡し方）は「`Observes` + `id`」で揃える方向で決着
  - 任意: `match (状態, 反応)` のタプル match にすると4通りが表の形で並び、書き忘れをコンパイラが検出（Book 6.2 / 19章）
  - 未確認: テストの `epsilon = 0.0` を外すと失敗するシードがあるか（ε-greedy が効いている確認）
  - 自分の言葉で説明（ステップ4の締めで `Note.md` に）: `self` / `Observes` を値で受けると、なぜ2人目の呼び出しでエラーになるか（所有権・move・借用）
- 2026-11-07: **③-2 `update_qreport` のテスト完了、報酬設計（none / retro）完了**（`cargo test` 8件成功。コミット済み）
  - 済: `test_update_qreport` を4通り（状態2×反応2）に拡張。失敗の原因は `agent` の使い回し → ループ内で毎回作り直して解決
  - 決まったこと: ①none は通報者に `-COST` のみ（嘘への罰 `P_FALSE` は retro だけ。偽通報かどうかで none の報酬は変えない）②違反者の罰 `P_VIOL` は通報者が何人でも1回だけ（「j が通報された」＝ j を `See` した人のうち1人でも `Report`。`any` で bool）③「通報されたか」の bool は関数の**外**で計算して渡す（retro と場合分けしやすい）④1体用 `get_reward` / `run` は触らず、N体用の新しい関数を別に作る（自由関数。`Agent` のメソッドにしない: `self` を使わないため）
  - 済（報酬）: `ReportState { Ignore, ReportTrue, ReportFalse }` / `ReportStates`（i番目＝i番の通報状態）。`Params` に `r_true`（1.0）/`p_false`（1.0）を追加。`get_report_reward(params, &ReportStates, &Observes) -> Vec<(f64, f64)>`（戻りは（通報者の報酬, 違反者の罰）、i番目＝i番）。通報者は none が `-cost`、retro が本物なら `-cost + r_true`、偽物なら `-cost - p_false`。違反者の罰は「j を `See` した人の誰かが `ReportTrue`」のとき `-p_viol` を1回だけ（`observed` に印を付ける形。印は被害者の添字）
  - テスト `test_get_report_reward`: n=3、none と retro の両方（期待値は mode ごとに紙で計算して書く）。`default()` の `p_viol` を 3.0（モデルの値）にしたため、`test_run` 側で `params1.p_viol = 0.0` を明示（「罰なし」の基準）。これで未決の `default` 問題は決着
  - つまずきと解決: ①`observed_rewards` を `states[j]`（j自身の通報状態）と並べていた → 「j が通報されたか」は `observed[j]` で決まるので `match` は不要に ②印を付ける条件に「見た人が通報した」が抜けていた → `observes` と `states` を `zip` ③テストが `p_viol=0` で罰を検出できなかった
  - 未実装・未決: `Mode::Immediate`（R_IMM）の分岐は 0.0 のまま（else と同じ）。`SPAM_P` の定義は未決。浮動小数点の `assert_eq!` は今は通るが、値が増えたら誤差付き比較を調べる
- 2026-11-08: **`set_report_states` 完了**（`cargo test` 10件成功。コミット済み）
  - 済: `set_report_states(&Observes, &Reactions) -> ReportStates`（Report × See → `ReportTrue`、Report × Notsee → `ReportFalse`、Ignore → `Ignore`。表は自分で書けた）。先頭で `observer == i` を `assert_eq!`（i 番目の `Observe` ＝ i 番の見た側、という前提をコードに残した）。テスト `test_set_report_states`（4通り）と `test_set_report_states_wrong_order`（`#[should_panic]`）
  - 決まったこと: 前提が破れたら `Err` ではなく `panic`（呼ぶ側のバグなので。Rust Book 9.3）。ただし Axum API で外から通報を受けるときは、境界で `Result` / `Err` にする（その時に再検討）
  - つまずき: テストに `panic!` を直書きして常に失敗させていた → `#[should_panic]` を付ければ関数側のパニックで成功になる
  - 任意: `#[should_panic(expected = "...")]` で別のパニックと取り違えない／`observes` と `reactions` の長さの確認／`if` を `match (&状態, 反応)` にする
- 2026-11-09: **`get_rewards` のシグネチャまで**（`cargo test` 10件成功。本体は `todo!()`）
  - 決まったこと: ①N体の報酬は、先に `get_rewards` を作ってから、ラウンド関数につなぐ（小さく区切る）②`get_rewards` の中で1体用の `get_reward` は**使わない**（あれは `detect_p` の確率で罰を決める1体用。N体の罰は「通報されたか」で決まり、`get_report_reward` がすでに計算している）。`rng` も不要 ③損害（`harm`）: エージェント i が被る損害＝`harm × 自分以外の違反者の数`（元モデルの「総損害/R」26.6 ≒ 9.8×9×0.3 と一致）。損害は i の選択に関係なく同じだけ乗るので、`守る` と `違反` の Q値の差は変わらない → **学習の報酬には入れず、集計用の値として別の関数で計算する**
  - 済: `get_rewards(params, &Actions, &ReportStates, &Observes) -> Vec<(f64, f64)>`（第1要素＝違反Q に渡す報酬、第2要素＝通報Q に渡す報酬。`///` に書いた）。**`get_report_reward` はタプルの順序が逆（通報者の報酬, 違反者の罰）なので注意**
  - つまずき: 警告を消そうとして、1体用 `get_reward` の変数名を書き換えて壊した（`reward_reaction` → `_reaction`）→ 元に戻した。1体用は触らない方針
  - （後半で完了）`get_rewards` の中身を実装。下の 11/09 後半を参照
- 2026-11-09 後半: **`get_rewards` の中身を実装**（`cargo test` 10件成功。コミット済み）
  - 決まったこと: 違反Q の報酬は3通り（守った＝0.0／違反して通報されず＝`gain`／違反して通報された＝`gain - p_viol`）。守った人は罰を受けない（`See` は相手が違反したときだけ、罰の印は `ReportTrue` のときだけ → 守った人への通報は必ず `ReportFalse`）。「通報されたか」は `get_report_reward` の i 番目の `.1`（罰。0.0 か `-p_viol`）
  - 済: `get_report_reward` の結果と `actions` を `zip` して、`.1 == 0.0` → 行動で `gain` / `0.0`、それ以外 → `gain - p_viol`。通報Q の報酬は `.0` をそのまま
  - つまずきと解決: ①`match` の腕に `-1.0 * params.p_viol`（式）を書いた → パターンには固定の値しか書けない（Book 19章）②`iter()` の要素が `&f64` で `Vec<(f64, f64)>` にできない → `*report_reward` で値に
  - **（夕方に書き直した）現在の実装は「行動で先に分ける」形**: 違反 → `penalty + gain`、守った → `*penalty`（`.1`）。`== 0.0` の比較と `p_viol` の二重計算は無くなった。`cargo test` 11件成功（`test_get_rewards` の枠だけ追加、中身は空）
  - 残る問い: 守った人の枝が `*penalty` のままだが、有効な入力では罰はいつも 0.0（`See` は相手が違反したときだけ → 守った人への通報は `ReportFalse` で罰なし）。`0.0` と書く方が意図が出るか（ただし不正な入力を黙って通す/通さない、の違いがある）
- 2026-11-10: **`get_rewards` のテスト完了**（`cargo test` 12件成功。`83f4eef` でコミット済み）
  - 済: `test_get_rewards_mode_none` / `test_get_rewards_mode_retro`。n=3 で、守って本物の通報をした人（0）・違反して通報され偽通報もした人（1）・違反して見られなかった人（2）が全部入る例。期待値は `params`（`cost`・`r_true`・`p_false`・`gain`・`p_viol`）から組み立てていて、定数を変えても壊れない。期待値は紙で先に計算した
  - 未確認（任意）: 「1人が2人に通報されても罰は1回」「通報者が0人」の例。守った人の枝が `*penalty` のままでよいか（上の「残る問い」）
- 2026-11-10 後半: **③-3 損害の関数 `get_harms` 完了**（`cargo test` 13件成功。`2d4ac97` でコミット済み）
  - 決まったこと: 戻り値は **i 番目ごとの `Vec<f64>`**（誰がどれだけ被害を受けたかが評判・Twin Towns で使える。合計は `sum()` で出せる）。受け取るのは `&Params` と `&Actions`
  - 済: `get_harms(params, &Actions) -> Vec<f64>`。違反者は `(違反者数 - 1) × harm`、守った人は `違反者数 × harm`。テスト `test_get_harms`（n=3、`[違反, 違反, 守る]` → `[harm, harm, harm×2]`。期待値は `params.harm` から組み立て）
  - つまずきと解決: 最初は違反者の損害を 0.0 にしていた（自分以外の違反者からの損害が抜けていた）→ 式どおりに修正。全ラウンド分の行動を貯める `AllAction` / `get_total_harms` を作りかけた → 集計はラウンド関数で決めるので削除
  - 任意: 先に違反者を数えて（`filter` + `count`）から行動ごとに式を当てる形にすると、仮の `harms` Vec と2周目が不要。`harm_count as f64` の `as` は不要（すでに f64）
- 2026-11-10 夜〜11-11: **③-4 N体のラウンド関数 `round` を実装**（`cargo test` 13件成功。`77519c7` でコミット済み。**`round` 自体のテストはまだ無い**）
  - 決まったこと: ①`Agents` は **`&mut`** で受ける（ラウンドの最後に `update_qvalue` / `update_qreport`＝`&mut self` を呼ぶため。所有権だと呼んだ後に使えず、`&` では更新できない）②戻り値は **`(Actions, ReportStates)`**（生のデータを返し、集計は外で決める。違反率＝`Actions` の `Violated` を数える／偽通報/R＝`ReportStates` の `ReportFalse` を数える／総損害/R＝`get_harms(params, &actions)` の合計）③順序: `get_actions` → `Observes::new` → `get_reactions` → `set_report_states` → `get_rewards` → 全員分の更新
  - 済: `round(agents: &mut Agents, params, rng) -> (Actions, ReportStates)`
  - つまずきと解決: ①`.map(...)` だけでは何も実行されない（イテレータは遅延評価。Book 13.2）→ `for_each`、最終的には `for i in 0..params.n` と添字に（長い `zip` の連鎖より読みやすい）②`iter_mut()` の要素 `&mut Agent` に対してパターン `&mut agent` は move しようとする（`Agent` は `Copy` でない）→ 添字で `agents.agents[i]` を借りる形で回避
  - 注意: `params.n` と `agents.agents.len()` が違うと範囲外でパニック（先頭で `assert_eq!` か、`0..agents.agents.len()` に）。`Agent` のカウンタ（`report` / `false_report` / `ignore`）の更新はまだ無い。ステップ5で `ReportStates` から数えるなら不要、`Agent` のカウンタを使うならここで足す → **どちらにするか決める**
- 2026-11-12: **`round` の最初のテスト完了**（`cargo test` 14件成功。コミット済み）
  - 方針: A（`round` を実際に呼び、返り値と Q値の**つじつま**を確かめる）に決着。乱数に頼らず場面を決めるには、**初期値で固定**する: `epsilon = 0.0`（探索なし）、`detect_p = 0.0`（誰も `See` にならない）、`Qvalue` / `Qreport` の大小で行動・反応を狙う。手で作った `actions` 等は `round` に渡せない（`round` が中で作る）ので、前半の手作り部分は `round` の確認には不要
  - 済: `test_round`（n=2）。`agent0` は Q値が引き分け → `get_action` の `>` が偽で **`Violated`**（引き分けは `Violated` 側）、`Qreport` 引き分けは `>=` で **`Report`** → 偽通報。`agent1` は `compliant_case = 1.0` で守る、`ignore_notsee = 1.0` で**無視**。期待値: `agent0.violated_case` = 0.1（報酬 `gain` ×0.1）、`agent0.report_notsee` = -0.02（`-cost` ×0.1）、`agent1.compliant_case` = 0.9、`agent1.ignore_notsee` = 0.9（**無視は `-cost` がかからない**＝報酬 0.0）。選ばなかった枝は全部動かない。戻り値も `assert_eq!`（`actions == [Violated, Compliant]`、`report_state == [ReportFalse, Ignore]`）
  - つまずきと解決: ①`params` を作る前に `params.seed` を使った ②期待値が前の設定（罰を受ける場面）のまま残っていた → 今の設定で `round` の中を追って計算し直した ③`ignore_see + multiplier` の `+` の打ち間違い（`*` にする） ④`agent1` の期待値 0.88 → 無視の報酬は 0.0 なので 0.9（`get_report_reward` の `Ignore => 0.0`）⑤`Agent` を `Copy` にしない理由: 暗黙の複製で、更新したのに元が変わらないバグが起きやすい。`Clone`（明示的な `.clone()`）で十分
  - 残る（任意）: 前半の手作りの `actions` / `reactions` / `report_states` / `observes` / `get_rewards` と `println!` の整理。`Agent { ... }` の2人分の重複（`Agent::new` + 上書き）。同じ丸めの式を小さな関数に
- 2026-11-13: **`See` を起こす場面の `round` のテスト完了**（`cargo test` 15件成功。コミット済み）
  - 済: `test_round_see`（n=2、`epsilon = 0.0`、`detect_p = 1.0`、retro）。agent0 は守る・`report_see` が大きいので、agent1 の違反を `See` して `ReportTrue`。agent1 は違反（`violated_case` が大きい）・`Notsee` で `Report` → `ReportFalse`。期待値: agent1 の `violated_case` = 0.7（報酬 `gain - p_viol` = -2.0 → `(-2.0 - 1.0) * 0.1 + 1.0`）、agent0 の `report_see` = 0.98（`-cost + r_true`）、agent1 の `report_notsee` = -0.12（`-cost - p_false`）、agent0 の `compliant_case` = 0.45（守った報酬 0.0 に向かって動く）
  - つまずきと解決: 期待値の `0.0` は「Q値が初期値から動かない」前提だった。実際は選んだ行動のQ値は報酬の方向に10%動き、選ばなかった枝は初期値のまま（初期値が 0.5 なら 0.5）。`round` の中を追って計算し直した。`Note.md` に agent1 が 0.7 になる計算を記録
  - 任意: `params.epsilon = 0.0` の重複、`get_rewards` と `println!` の整理（`test_round` と同じく）
- 2026-11-14: **カウンタの決定と none のテスト完了**（`cargo test` 17件成功。コミット済み）
  - 決まったこと: ステップ5の3指標は `round` の戻り値から数える（違反率＝`Actions` の `Violated`、偽通報/R＝`ReportStates` の `ReportFalse`、総損害/R＝`get_harms` の合計）。`Agent` のカウンタは集計には要らないが、**評判の材料（犯罪歴・虚言癖・無関心）として残す**（本人の発案。台帳→評判のフェーズで使う）
  - 済: `Agent` に `count_violate` / `count_report_true` / `count_report_false` / `count_ignore` を追加（`ReportState` の名前に揃えた）。選んだ瞬間に `get_action` / `get_reaction`（`&mut self`）の中で数える。**無関心は `See` × `Ignore` だけ数える**（見ていない無視は当然なので数えない）。ゲッターは消し、フィールドを直接読む
  - テスト: `test_round_see`（`report_true` / `report_false` / `violate`）、`test_round_ignore_observestatus`（`See` × `Ignore` は数え、`Notsee` × `Ignore` は数えない）、`test_none_case_violate_and_report`（n=10、1000ラウンドの後半500。通報の割合 0.13〜0.17、違反率 0.83〜0.87。実測 0.151 / 0.8388）
  - 分かったこと: ε-greedy では、貪欲な選択が `Ignore` でも、探索で `ε × 0.5` だけ通報が残る（ε=0.3 で 0.15）。違反率の理論値は 1 − ε/2 = 0.85（罰で違反Qが一時的に下がるので、実測は少し低い）。retro でも、違反が減ると `See` が減り、通報の割合は 0.15 に近づく → **3条件の違いは通報の割合ではなく、違反率と偽通報/R に出る**
  - 残る（任意）: `get_action` / `get_reaction` が数える副作用を持つ点（同じ `Agent` で2回呼ぶと2回数える）。`test_none_case_violate_and_report` の未使用の `all_actions` / `all_report_states`、`i >= 500` を `epochs / 2` から出す
- 2026-11-14 後半: **`Mode::Immediate` の報酬を実装**（`cargo test` 17件成功。コミット済み）
  - 決まったこと: **`r_imm`（通報の瞬間にもらう額）は immediate と retro だけ**（none は0）。retro は「即時の `r_imm` ＋ 後で照合」（元の構想「検知したら即共有→軽い報酬→あとで照合」に合う）。immediate は真偽を確かめないので、本物も偽通報も `r_imm - cost`（0.1）
  - 済: `get_report_reward` を3つの値に分けた: `reward_imm`（immediate / retro）、`reward_true_bonus`（retro だけ `r_true`）、`penalty_report_false`（retro だけ `p_false`）。本物＝`-cost + reward_imm + reward_true_bonus`、偽通報＝`-cost + reward_imm - penalty_report_false`。retro の期待値: 本物 1.1、偽通報 -0.9（以前は -1.2）
  - つまずきと解決: ①`ReportFalse` に `+ r_imm` をモードに関係なく足し、none でも入った → モードごとの変数に ②テストの `Retro` と `Immediate` を同じ `else` で扱っていた → 枝を分けた ③`round_float(rewards[2].0)` を2つ並べていた（2つ目は `.1`）④浮動小数点の `assert_eq!` は丸め（`round_float`）で対応。`test_round_see` の `agent1.report_notsee` は -0.09（(-0.9)×0.1）に更新
  - 積み残し: `get_rewards` の `Immediate` 用テスト（n=3 で retro と同じ3人の例）は、本人が「合格にして進める」と判断して省いた。`round` を immediate で回すテストも無し
- 2026-11-15: **ステップ4 完了、ステップ5 途中（none の結果が出た）**（`4c3b93b` でコミット済み。`src/experiment.rs` 新規）
  - 済: `SPAM_P` は使わない（偽通報は `qreport` の学習で出る）。`Note.md` に所有権の説明を記入（ステップ4 の締め）
  - 済: ε を引数化。`Params` は `epsilon_start`（0.3）/`epsilon_end`（0.02）、`epochs` の既定値は 3000。`get_action` / `get_reaction` / `get_actions` / `get_reactions` / `round` が `epsilon` を引数で受ける。round のテストは `let epsilon = 0.0` に直した（`params.epsilon = 0.0` のままだと無効なのに、シード固定の偶然で通っていた）
  - 済: `src/experiment.rs`（`run(&Mode, seed)` → `(Agents, Vec<Actions>, Vec<ReportStates>)`、`run_seeds(mode)` → 20シードの3指標。ファイル名は `experiment`、`run_n` と集計を同じファイルに置く。同じ20シードを3条件で使い回す＝差が `mode` の違いから出たと言える）。ε の式は `start - (start - end) / epochs * i`（最初の式は `epochs * i` で割っていて減衰しなかった）
  - none の結果（3000ラウンド×20シード、末尾20%）: 違反率 約0.975（元 0.98）、総損害/R 約26.3（元 26.6）→ 合う。**偽通報は 約0.012/人/R（10人で約0.12/R）で、元の 0.4 の約1/3**。原因は未確認（`SPAM_P` を使わない方針との関係を疑う。ε が 0.02 まで下がると探索の通報が約 ε/2 × 見ない割合 ≈ 0.01/人 になる）
  - つまずきと解決: ①`total_harms:f64 +=`（`:f64` は `let` でしか書けない。`sum::<f64>()`）②末尾の判定 `i >= 2400` が `epochs` 1000 のとき一度も真にならず全部 0.0 → `epochs` を 3000 に ③分母に `epochs`（3000）と `/0.8` を使い、数えたのは末尾600ラウンドだけで、違反率が 0.21 になった（直した）
  - ~~`test_none_case_violate_and_report` が失敗中~~ → 解決（同日。原因: `epochs` を 3000 にしたのに、後半の判定が直書きの `i >= 500` のままだった。`params.epochs / 2` にして `cargo test` 18件成功）。教訓: 既定値を変えるときは、それに依存する直書きの数字を探す（`run` の `i >= 2400` も同じ種類）
  - 済（夕方）: `run` の末尾判定を `epochs * 0.8` から出した。`run_seeds` の分母が `epochs / 5 * 4`（2400）で、保存した600ラウンドと食い違い違反率が 0.244（正しい値の1/4）になっていた → 直して none は 違反率 約0.975・総損害 約26.3 に戻った（`cargo test` 18件成功）。教訓: 集計の分母は「数えた対象の数」。`epochs` から別の式で出すより、保存した `Vec` の長さなど手元の値の方がずれない
  - 未着手: immediate / retro の `run_seeds`（`mode` を変えて回す）、20シードの平均を取る、3条件の表（問い: 20個ずつの3つの `Vec` から表の1行をどう作るか。未回答）。`run_seeds` の `/600.0`（損害）は直書き。`run_seeds` の未使用変数（`seeds`・`agents`・`all_report` の扱い）。`run_seeds` が `Params::default()` を使うので、`run` 内の `params` と食い違う危険（`mode` で `get_harms` は変わらないが、`p_viol` などを変えるときに注意）
- 2026-11-16: **`run_seeds` の分母を手元の値に**（`cargo test` 18件成功。`731bde2` でコミット済み）
  - 済: 分母を `all_report.len()`（保存した `Vec` の長さ）に。`assert_eq!(all_actions.len(), all_report.len())` で前提をコードに残した。`test_run_n` に「20シードすべてで違反率が 0.95〜1.0」の `assert` を追加（表示だけのテストではなくなった）。結果は変わらず（none: 違反率 0.969〜0.979、総損害/R 26.1〜26.4）
  - 任意: 総損害/R にも範囲の `assert`（25〜28 など）。範囲の根拠（元モデルの値と実測）をコメントに
  - **答えかけの問い（偽通報/R の謎）**: 今のモデルは各自が1ラウンドに1人だけ見る（11/01 の決定）。N=10 で1ラウンドに起こりうる偽通報の最大数は？（本人の答えは「9」→ 全員が守ったラウンドで10人全員が `Report` したら？ 偽通報は違反者がいなくても起きる）。それを元モデルの **immediate の偽通報/R = 22.5** と比べると、元モデルの「見る」の仕組みについて何が分かるか？（none の 0.12 と 0.4 の差の原因にもつながる）
- 2026-11-16 後半: **見る仕組みは変えないと決定、`main.rs` で3条件の表が出た**（`dbab043` でコミット済み。**ただし `cargo test` がコンパイルエラー**）
  - 決まったこと: **今のモデル（各自が1ラウンドに1人だけ見る）のまま進める**。理由: 変えると5つの関数とテストに影響し、`update_qreport` が1ラウンドに9回になって学習の速さも変わる。v0.1 の目的（3条件で違反率がどう変わるか）には影響しない。考察も楽（偽通報/人/R ＝「1回見たときに偽通報が起きる割合」と読める）
  - 分かったこと（偽通報の謎）: 今のモデルは1ラウンドの偽通報が最大10（全員が守り全員が通報）。元モデルの immediate 22.5/R は出せない → 元モデルは各自が複数人（おそらく自分以外の9人、最大90）を見ていた（元実装がないので推測）。none の 0.12 と 0.4 の差も見る回数の違いで説明できそう。**偽通報/R の絶対値は元モデルと比べない。条件どうしの違いの向き・倍率で比べる。** 違反率・総損害/R はそのまま比べられる（none が合ったことが裏づけ）
  - 済: `src/main.rs`（`cargo run`）。`run_seeds(&Mode)` を3条件で回し、20シードの平均を `Result { mode, violated_rate, report_false_rate, mean_harms }` にまとめて表示。`run_seeds` の引数を `&Mode` に変更
  - ~~壊れている: `test_run_n` が `run_seeds(mode)` のまま~~ → 直った（`cargo test` 18件成功）
  - 結果（3000ラウンド×20シード、末尾20%。偽通報は /人/R）:

    | 条件 | 違反率 | 偽通報/人/R | 総損害/R |
    |---|---|---|---|
    | none | 0.976 | 0.012 | 26.34 |
    | immediate | **0.033** | 0.960 | 0.89 |
    | retro | **0.033** | 0.023 | 0.89 |
    | （元モデル） | none 0.98 / immediate 0.98 / retro 0.02 | 0.4 / 22.5 / 0.7（/R、見る仕組みが違う） | 26.6 / 26.6 / 0.4 |

  - **気になる点（次回の問い）**:
    1. **immediate の違反率が 0.033 で、元モデルの 0.98 と大きく違う**。immediate でも違反が減ってしまっている。今のモデルでは、本物の通報（`See` × `Report`）があれば mode に関係なく違反者に `-p_viol` が付く。元モデルの immediate で違反が減らなかったのはなぜか？ immediate で「通報されたら違反者が罰を受ける」のは正しいか？（真偽を確かめない条件で、罰は何を根拠に付くのか）
    2. **immediate と retro の違反率・総損害/R が小数点以下まで完全に一致**。偶然か仕組みか要確認（ヒント: 乱数を引く回数は Q値に関係なく同じ／両条件とも `See` なら通報が得になる → 違反者が罰を受ける場面が同じになる？）
    3. 表示が `{:?}` で1行。表の形（`println!` の書式指定）にすると見比べやすい（任意）
- 2026-11-16 夜: **違反者の罰は retro だけに変更、3条件の表が元モデルと同じ向きに**（`cargo test` 18件成功。`baf9a98` でコミット済み）
  - 決まったこと: **違反者の罰 `p_viol` は retro だけ**（後で照合して確定する条件だけ。immediate は真偽を確かめないので通報だけでは罰を付けない。none も付けない）。`get_report_reward` で `p_viol` を mode で分け、none / immediate のテストの期待値を罰 0 に直した。1体用 `get_reward` は触らない（途中で入れた `else { params.p_viol = 0.0 }` は自分で消した）
  - 結果（3000ラウンド×20シード、末尾20%）:

    | 条件 | 違反率 | 偽通報/人/R | 総損害/R |
    |---|---|---|---|
    | none | 0.976 | 0.012 | 26.34 |
    | immediate | **0.976** | **0.498** | 26.34 |
    | retro | 0.033 | 0.023 | 0.89 |
    | （元モデル） | 0.98 / 0.98 / 0.02 | 0.4 / 22.5 / 0.7（/R） | 26.6 / 26.6 / 0.4 |

  - 分かったこと: ①none と immediate の違反率・総損害が**小数点以下まで一致**するのは、罰が 0 なら違反Q の更新が同じで（通報Q は違反Q に何も返さない）、乱数を引く回数も Q値に左右されないから。retro だけは `p_viol` で違反Q が変わり、行動の列が変わる（同じシードでも別の結果）②同じシードは必要条件だが十分条件ではない③immediate の偽通報は none の約40倍（通報が `r_imm - cost` = 0.1 で得になる）。none では通報が `-cost` だけで損なので、探索（ε）の分しか起きない④`Note.md` の考察の材料になる
  - 残る（任意）: `get_reward` の閉じ括弧の後の空白（`} `）。`test_get_rewards_mode_none` の期待値の根拠をコメントに。`Immediate` 用の `get_rewards` テストはまだ無い
- 2026-11-16 夜: **v0.1 完了**（STEPS.md ステップ1〜5。3条件×20シードの平均の表が出て、違反率・総損害は元モデルと同じ向き）。`Note.md` に「immediate で偽通報が増える理由」「retro で違反が減る理由」を記入（本人。retro の1行目は「罰は通報されたときだけ付く → 違反の期待値 = gain − 通報確率 × p_viol、確率 > 1/3 で違反が損」とつなぐと読みやすい。任意）
  - `baf9a98` でコミット済み
  - 次の判断（v0.1 の後）: 年内の範囲の見直し（台帳 → 評判 → Axum API の最初の口 → Lovable）。「理解を優先」（11/15）。Twin Towns の「違反の定義」は未決
- 2026-11-16 終了時: **台帳から考え始める**と決めた（Twin Towns の違反の定義より先）。**次回の最初の問い（未回答）**: 1ラウンドの中で、台帳に1件ずつ記録するべき出来事（イベント）にはどんな種類があるか（行動・見た・通報・罰・報酬など、今のシミュレーションの言葉で挙げる）。材料: 評判は台帳の検証済みイベントだけから計算する（10/28 の決定）、`Agent` のカウンタ（`count_violate` / `count_report_true` / `count_report_false` / `count_ignore`）は評判の材料として残してある（11/14）、Twin Towns の台帳は `LedgerEvent`（`tick` と `type`）の形（11/04）。コードを書く前に、イベントの一覧を紙に書く
- 2026-11-17: **台帳の考え方を整理（コードはまだ）**。`Note.md` の「# Step02 台帳に持つ値」に一覧を書き始めた（未コミット）
  - 台帳とは: 起きた出来事を順に1件ずつ書き足す記録（通帳のイメージ）。書いた行は消さない・書き換えない。カウンタ（状態）は台帳を集計した結果で、台帳から作り直せる（逆はできない）
  - 決まったこと:
    1. **`actions` は台帳に書かない**。シミュレーションは「神さまの視点」で誰が違反したかを知っているが、現実・Twin Towns の台帳は知らない。`actions` を書くと通報の仕組みが不要になってしまう
    2. **主張＝通報（Report）、確定＝照合の結果（本物/偽物）**。確定の根拠は、シミュレーションでは `observe_status`（見た本人しか知らない値）、現実では**証拠と、それを確かめた結果**
    3. **投票（多数決）は入れない**（10/28 の決定を維持）。理由: 1人が1人だけ見るので、違反者 j を見るのは平均1人、`See` は平均0.5人 → 過半数は成り立たない（全員を見る仕組みでも `See` は平均4.5人）。見ていない人の投票は根拠がない
    4. **証拠は「その違反が本当に起きたときにしか手に入らないもの」**。中身は「誰の（違反者＝`Observe.observed`）・いつ（ラウンド番号）・何をした（`Action::Violated`）」＋**通報者の id（`observer`）**。最初は `bool` でもよいが、台帳の1件には誰が・誰を・いつ、が要る
    5. **証拠を出すのは見た側、確かめるのは台帳**。今の `set_report_states` が一瞬でやっている判定を、台帳の仕事として切り出す。シミュレーションの台帳は照合で結局 `actions` を見る（証拠は完璧・照合は必ず正しい前提）が、「確かめる手順」を独立した場所に置けば、後で ZK の検証や Twin Towns の台帳（SQLite / コントラクト）に差し替えられる
  - `Note.md` の一覧（`reported` / `proof` / `is_gilty`、いずれも `bool`）への指摘: 誰が・誰を・いつ、の情報が無い。綴りは `is_guilty`
  - 決まったこと（続き）:
    6. **罰・報酬も台帳に記録する**。理由: 記録しないと過去の罰を**今の `params`** で計算し直すことになり、`p_viol` を 3.0 → 2.0 に変えると過去が実際の 2/3 に書き換わる。記録すれば `params` を変えても台帳は「そのとき実際に科された罰」を残す（台帳が残すのはルールではなく事実）。別解: 罰の額ではなく「そのときのルールの版（`p_viol` = 3.0）」を記録する方法もある（今回は額をそのまま記録）
    7. 照合の結果と罰・報酬は内容が重なるが、**互いを照らし合わせて確かめる手段**になる。食い違い＝改ざん（コモン街の市長が罰を書き換えた等）かバグ。ハッシュチェーン（過去の1件が書き換えられたことを見つける）とは別の守り方（記録どうしの中身の矛盾を見つける）
- 2026-11-19: **台帳のイベントの型を紙で決めた**（`Note.md` の「# Step02」。`9fb0e00` でコミット済み）
  - 決まったこと: ①台帳は**1本の `Vec<Event>`**（種類をまたいだ順序を残す。ハッシュチェーンは1本の列にしか掛けられない）②`Event { round: usize, kind: EventType }`（`type` は予約語なので `kind`）③`EventType` は `Report` / `Judge` / `Reward` の3種類で、種類ごとに持つ項目が違う enum ④`Report { observer, observed }`（主張）／`Judge { observer, observed, is_guilty: bool }`（確定＝通報が本物か）／`Reward { who, reward: f64 }`（結果。罰は負の額）⑤`Judge` を残す理由: none では本物も偽物も報酬が同じで、額から真偽は分からない。`Reward` を残す理由: `params` を変えても過去の額が書き換わらない（11/17 の6）
  - 途中の気づき: 通報者が複数でも、`Report` / `Judge` は通報1件につき1件。罰は通報者が何人でも1回だけなので、`Reward` を別の種類にしたことで1回だけ書ける
  - 任意: `is_guilty` は「observed が本当に違反したか」の意味。名前が `See` / `Notsee` と並ぶと紛らわしいので、後で見直してよい。`Reward` に「何の報酬か」（通報の報酬／罰）を持たせるかは未決
- 2026-11-19 後半: **`Ledger` の設計を言葉で決め、`src/ledger.rs` に型を書いた**（`9fb0e00` でコミット済み。11/20 に `cargo test` 18件成功を確認）
  - 決まったこと: ①公開するのは「読む」と「末尾に足す（`append`）」。公開しないのは書き換え・削除・途中への挿入と、`events` フィールドそのもの ②読む側には `&[Event]` を返す（`&Vec` でも変更はできないが、中身が `Vec` であることを外に見せないため。後で中身を差し替えても読む側が変わらない）③読むメソッドは `&self`、`append` は `&mut self` ④`&mut` が1つある間は他の参照（読む参照も）は持てない。読む参照を持ったまま `append` するとエラー。読む参照は最後に使った行で終わる（NLL）ので、`view → append` の順で、`append` の後に `view` を使わなければ通る
  - 済: `src/ledger.rs` に `Event { round, kind }` と `EventType`（`Report` / `Reward` / `Judge`）。種類ごとに struct を作らず、**`{ }` の形で項目を持たせる enum** にした（`Report { observer, observed }` など）
  - つまずきと解決: enum の種類にデータを持たせるとき `Report:Report` と書いた → `Report(Report)` か `Report { ... }`（Book 6.1。`:` はフィールド名と型の組。enum の種類名は名前が先で、持つ値の型は後）。`float` は `f64`
  - **未了**（11/20 時点）: ①~~`lib.rs` に `mod ledger;` が無い~~ → `pub mod ledger;` あり（済） ②`Event` / `EventType` に `pub` が無い。`Event` のフィールドを `pub` にするか（外から書き換えられてしまう）、非公開にして読む手段を作るか、が未決（最後の問い・未回答）③`Ledger { events }`・`new`・`append`・`events()` は未着手 ④`Note.md` に「公開するもの／しないもの」「`&[Event]` を返す理由」「`&self` と `&mut self`」を自分の言葉で書く（依頼済み・未確認）
- 2026-11-20: **`Events`（台帳）の `new` / `read` / `append` に着手**（`src/ledger.rs` は未コミット。**今はコンパイルが通らない**）
  - 済: `Event` と `EventType` に `#[derive(Debug, Clone, PartialEq)]`。`Event::new(round, event_type)`（作るための関数）。台帳の struct は `Events { events: Vec<Event> }`（フィールドは非公開）。`append(&mut self, event: Event)` は `push` で末尾に足す（通る形）
  - 分かったこと（`Copy`）: `Copy` は「ビットをそのまま複製すれば完全な値になる」型だけに付く。条件は中身が全部 `Copy`。`EventType` / `Event` は `usize`・`f64`・`bool` だけなので付けられるが、**`Events` は `Vec` を持つので付けられない**（`Vec` は本体を heap に持ち、ビットの複製だと住所だけ写って持ち主が2人になる＝二重解放）。`Vec` は `Clone`（`.clone()` で本体ごと複製）。台帳は「こっそり複製されては困る」ので、11/12 の `Agent` と同じく `Copy` にしない
  - **`Vec` と `&[T]` の違いが分からない**（本人。説明を3通り試した: ①図（`Vec` は ptr・len・capacity を持つ持ち主、`&[T]` は ptr・len だけの借りた窓）②ノートとのぞき窓のたとえ ③自分のコードとのつながり＝11/05 の `&Observes` と同じ「貸す」で、貸すものが「Event の並んだ列」になっただけ）。**heap と stack の違いではなく、持ち主か借りているだけか、の違い**。まだ腑に落ちていない
  - 今のコードの問題（`read`）: 中で新しい `Vec<&Event>` を作って clone を集め、その参照を返そうとしている → エラー2つ（`Event` から `Vec<&Event>` は作れない／型の不一致）。さらに、関数の中で作った `Vec` の参照は関数が終わると消えるので返せない。**新しく作らず、台帳がすでに持っている `events` をそのまま貸せばよい**（`String` から `&str` を作るのと同じ書き方。Book 4.3）
  - 残る: `Event` のフィールドが `pub` のまま（台帳に足した後でも外から書き換えられる。`Event::new` を作ったので非公開に戻せる。ただし外から読む手段＝ゲッターが要る）。`fn write()` は中身なし（公開しないと決めたので要らないはず）。`Events::new()`（空の台帳を作る）がまだ無い。`Note.md` の「`&[Event]` を返す理由」が「変更できないようにするため」のまま（`&Vec` でも変更はできない。決めた理由は「中身が `Vec` であることを外に見せないため」）
- 2026-11-21: **`Ledger` の最小版が動いた**（`cargo test` 19件成功。`src/ledger.rs` は未コミット。上の 11/20 の問題は全部解消）
  - 済: `Events` → `Ledger` に改名。`new`（空の台帳）、`read(&self) -> &[Event]`（`&self.events` だけで通る）、`append(&mut self, Event)`。空の `write` は削除。`events` は非公開のまま
  - テスト `test_read`: 3種類のイベントを `round` を 0〜3 に変えて4件足し、足すたびに 0〜i 件目の全部が `read()` 経由で変わっていないことと、件数を確かめる（最初の版は直前の1件だけ・`round` が全部 0・件数なし・`ledger.events` 直読みだった → 直した）
  - 腑に落ちたこと（`Vec` と `&[T]`）: 窓（slice）の持ち主は `self.events` で、`read` が終わっても生きている。前の版は `read` の中で作った `Vec` の窓を返したので、持ち主が先に消えた（E0515）。`&` は読むだけ・slice には `push` が無い（本人が「参照で所有権がないから書き換えられない」と説明できた）
  - 分かったこと（非公開）: Rust の非公開は**モジュール単位**。子モジュール（`tests`）は親の非公開フィールドに触れる（`use super::*` は名前の近道で、理由ではない）。外のモジュール（`experiment.rs` など）からは `ledger.events` はコンパイルエラー。「書き換えない」を守る本体は、テストよりも、非公開フィールド＋`&mut` を返さない設計（Book 7.3）
  - 任意: `test_read` の `println!` と最後の4行（ループと重複）の整理。`Ledger::default()`（clippy の `new_without_default`）。空の台帳を読むテスト。`Event` のフィールドの `pub`（台帳に足した後でも外から書き換えられる。`Copy` なので `read()` の `&Event` からは直接書き換えられないが、`Event` を取り出して作り直すことはできる。非公開にして読むメソッドを作るかは未決）
  - 未確認: `Note.md` の「公開するもの／しないもの」「`&[Event]` を返す理由」「`&self` と `&mut self`」（11/19 に依頼）。「変更できないようにするため」のまま残っていたら、「中身が `Vec` であることを外に見せないため」に直す
- 2026-11-22: **照合の切り出し（`Verify` トレイトと `SimVerifier`）が動いた**（`cargo test` 21件のはず。`src/ledger.rs`・`src/lib.rs` は未コミット。**未コミットのまま休憩**）
  - 決まったこと（紙の設計）: ①照合の材料に神さまの値（`actions`・`ObserveStatus`）を使うのは、台帳（`Ledger`）の外に出した受け口だけ。台帳は `Verify` トレイトだけを知る ②**後で証拠（ZK の検証結果・Twin Towns の台帳）に差し替えたいので、手順を `trait` にする（enum ではなく）**。`Report` は今のまま（`observer`・`observed`）③`verify(&self, event: &Event) -> Option<bool>`（`round` が `Event` にあるので `Report` 単体ではなく `Event` を受け取る）④戻り値の意味: `Report` 以外の `Event` → `None`／`Report` → `Some(本物か)`／材料が壊れている（そのラウンドに合う `Observe` が無い）→ `panic`（呼ぶ側のバグ。11/08 の `set_report_states` と同じ種類。`Option` は「種類が違う」だけを表す）⑤見ていない通報は、たまたま当たっても偽物（今の `set_report_states` の挙動と同じ。v0.1 の結果とも一貫）
  - 腑に落ちたこと: 台帳は見た本人の頭の中（`ObserveStatus`）を知らない。`ObserveStatus` はシミュレーションが `detect_p` と `actions` から決めた値で、台帳から見れば神さまの値。だから受け口を台帳の外に出して狭く保つ。`actions` で照合すると「見ていなくてもたまたま当たった通報」が本物になり、retro の「嘘を言うと損」が弱まる（モデルの意味が変わる）
  - 済: `Verify`（トレイト）、`SimVerifier { all_observes: Vec<Observes> }`（i 番目＝i ラウンド）、`SimVerifier::new`、`verify` の中身（`is_found` で「合う Observe があったか」を持つ）。テスト `test_verifier`（`Some(true)`・`Some(false)`・`None`×2）と `test_verifier_panic_case`（`#[should_panic(expected = "...")]`）
  - つまずきと解決: ①`match` のパターンで `Report{target_observer:usize, ...}` と型を書いた → パターンの `:` の右は新しい変数名（フィールド名は enum の実際の名前。Book 19.3）②`for_each` の中で、他人の `Observe` に当たるたびに `panic!` していた（n≥2 でほぼ必ず止まる）→ ループの外に出して `is_found` で判断 ③`panic!` が `for` の外で必ず実行される位置にあった → `is_found` で分岐
  - 任意: `test_verifier` の未使用 `event3`・`ledger`、`Observes` の組み立ての重複（小さな関数に）。`SimVerifier::new` に `pub` が無い（`lib.rs` / `experiment.rs` から使うとき）。`round` が範囲外のときの添字パニックと、`expected` の文言が別なのでテストで区別できる
  - 見積もりの見直し（11/21）: Lovable の前まで 7〜11回。残り: 照合を台帳につなぐ・`round` から `Report` / `Judge` / `Reward` を書く 2〜3回、ハッシュチェーン 1〜2回、評判 2〜3回、Axum の最初の口 2〜3回。12/31 まで約5週間で取れるのは 11〜17回。Axum の最初の口までは届きそう、Lovable は「届けば」。遅れたら評判を最小（カウンタの集計だけ）にして API を先に通す手もある
- 2026-11-23: **`Report` と `Judge` の `Event` を作る関数ができた**（`cargo test` 22件成功。`src/ledger.rs` は未コミット。`README.md` は `382e1d5` でコミット済み）
  - 決まったこと（紙の設計）: ①**`Ledger` は足すだけ（B 案）**。照合は台帳の外で、呼ぶ側が `verify` の結果を受け取り、`Ledger::append` を1件ずつ呼ぶ（`Vec<Event>` は `for` で回す。複数用のメソッドは作らない。理由: ハッシュチェーンは1件ごとに「1件前のハッシュ」を使うので、足す口は1か所がよい）②台帳の並びは **`Report` → その `Judge` → 次の `Report` → その `Judge`**（通報1件ごとに組）。ただし結びつけは隣り合うことではなく `round`・`observer`・`observed` で行う③`Report` は 0〜n 件（全員が `Report` を選ぶと n 件。n−1 ではない）④`Judge` は、通報の `Event` が持つ `observer` / `observed` をそのまま使う（`verify` は `bool` を返すだけで、同じ値を返し直さない。差し替える実装の責任を増やさない）⑤**`Reward` の設計（次に書く）**: 違反者 j を a・b の2人が本物と通報したら、通報者2人分の報酬と違反者の罰1件の計3件。`none` / `immediate` は罰なし、`retro` だけ罰。**書くかどうかは値（`== 0.0`）ではなく何が起きたかで決める**: 通報者は `ReportState`（`ReportTrue` / `ReportFalse` なら書く、`Ignore` は書かない）、違反者は `Judge` の `is_guilty = true`（罰は j につき1回だけ。`observed` に印を付けて覚える）。起きなかったことは台帳に書かない（11/19）
  - 済: `make_report_event(round, &Observes, &ReportStates) -> Vec<Event>`（`ReportState` が `ReportTrue` / `ReportFalse` の人だけ。`zip` の前に長さを `assert_eq!`）、`make_judge_event(&Vec<Event>, &Vec<Observes>) -> Vec<Event>`（`Report` の `Event` ごとに `verify`。`Some(is_guilty)` を1つの枝にまとめた）。テスト `test_make_report_event`（`make_judge_event` のテストはまだ無い）
  - つまずきと解決: ①`.map` の中で `if` に `else` が無い（`E0317`）→ `map` は全員から必ず1件作る。0件か1件は `filter` + `map`（または `filter_map`）②`SimVerifier` を `&'a Vec<Observes>` を借りる形に変えた → テストの補助関数が、中で作った `Vec` を借りる `SimVerifier` を返して `E0106` / `E0308`。補助関数は `Vec<Observes>` を返し、`SimVerifier` はテストの中で `&sim_vec` を借りて作る形に直した ③`cargo test -p ledger` はできない（`-p` はパッケージ。モジュールは `cargo test ledger` の名前の絞り込み。Book 11.2）
  - 腑に落ちたこと（lifetime）: `'a` は新しい値を作らず、借りた相手と借りている間の長さをコンパイラに伝える注釈。`read` の E0515 と同じ種類の安全の話（ローカルの `Vec` を借りた物を返すとダメ）。自分で書くのは①`struct` が参照を持つ ②参照を返す関数で参照の引数が2つ以上 ③`'static`（スレッド・Axum の共有状態。所有または `Arc` にする）。`struct` には参照ではなく所有する値を入れれば避けられる（Book 10.3）。**借りる形のままにする**: 呼ぶ側が `all_observes` を後の `Reward` の計算でまた使うため（所有にすると `clone` が要る）。`SimVerifier` を `Ledger` や Axum の状態に長く持たせるときは見直す
  - 任意: `make_judge_event` の `judege_event`（綴り）、`for` の外側の `match` を `if let` に。`make_report_event` の長さの食い違いを `#[should_panic]` で確かめるテスト。`test_read` の `println!` と最後の `assert_eq!` の整理
  - 見積もり（11/23 時点）: Lovable の前まで 6〜10回（照合をつなぐ残り 1〜2回、ハッシュチェーン 1〜2回、評判 2〜3回、Axum の最初の口 2〜3回）。12/31 まで約5週間で取れるのは 10〜16回
- 次の一歩（フェーズA 台帳）:
  0. （11/23 済）`Report` と `Judge` の `Event` を作る関数。**次は `Reward` の `Event` を作る関数**。最初の問い: `reward`（額）は何から取るか（通報者の報酬と違反者の罰は `get_report_reward` の `(通報者の報酬, 違反者の罰)` から取れるか）。罰を書くのは `retro` のときだけだが、その判断を何で決めるか（`Params.mode`。値の `== 0.0` ではなく）。そのあと ①`make_judge_event` のテスト ②ラウンドの中で、`get_reactions` の後に `Report` → `Judge` → `Reward` を `Ledger::append` で足す ③`Note.md` の説明（公開するものと、`&[Event]` を返す理由。未確認）
  （以下は 11/20 までの項目。0〜3 は済）
  4. その後: 照合を台帳の仕事として切り出す（今の `set_report_states` の判定）→ `round` から台帳に `Report` / `Judge` / `Reward` を書く
- （以下は v0.1 完了前の「次の一歩」。参考として残す）次の一歩（ステップ5 の続き）:
  1. `git add` / `commit` / `push`（本人。`src/lib.rs`）
  2. （済）「気になる点」1・2 は上の 11/16 夜で決着。3（表の書式）は任意
  3. 結果表を元モデルと見比べ、`Note.md` に考察を書く（immediate で偽通報が増える理由、retro で違反が減る理由。偽通報は倍率で比べる）
  - 残る（任意）: `main.rs` の未使用の `let mut results`、`Result` という名前は標準の `Result` 型と紛らわしい（`ResultRow` など）。`run_seeds` の未使用変数（`seeds`・`agents`）、`run_seeds` が `Params::default()` を使うので `run` 内の `params` と食い違う危険、警告の整理、総損害/R の範囲の `assert`
  - 決定済みの記録: `SPAM_P` は使わない（11/15）／ステップ4 完了（11/15）／`run` の戻り値 `(Agents, Vec<Actions>, Vec<ReportStates>)` と ε を引数で渡す設計（11/15）／見る仕組みは変えない（11/16）
- amnesia との接続: 違反・通報・評判は Twin Towns にまだない。「この街で何を違反とするか」は未決（v0.1 の後で検討）
- 見積もり: **v0.1 は 11/16 に完了**。次はフェーズA（台帳）。台帳の範囲（イベントの種類・照合の置き場所）を決めてから見積もる。11/12〜12/10 は応用編2と並走。年内の範囲（v0.1＋API の最初の口まで、など）は v0.1 の後に見直す
  - これまでの推移: 11/05「あと5〜8回」→ 11/08「3〜5回」→ 11/10「3〜4回」→ 11/11「2〜4回」→ 11/14「2回弱」

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

- 2026-11-15: **期限より理解を優先する**。見積もりは v0.1 まであと1〜1.5回、その先（台帳・評判・Axum API・Lovable）は10〜16回、補正して15〜25回。12/31 に Lovable まで届くかは怪しいので、Lovable は「届けば」の扱い。年内の範囲は、v0.1 → 台帳 → 評判 → Axum API の最初の口、の順に決めていく（範囲の見直しは v0.1 が済んだとき）

- 2026-10-28: ZK/SP1・分散台帳より先に、軽量シミュレーションでメカニズムを検証する
- 2026-10-28: 評判は台帳の検証済みイベントだけから計算する（投票は入れない）
- 2026-10-28: 学習用。11/12〜12/10 の応用編2と並走するため、週2〜3時間が目安

## 未決

- ~~`mode` を `Params` に入れるか~~ → 10/28 に「外に出す」と決めたが、**`Params` に入れる形に変更**（理由: 実験の設定を `Params` 1つで全部管理できる。シードも入れたので「`Params`＝1回の実験の設定一式」。3条件の比較は `p_viol` と同じく clone して `mode` を書き換える）（2026-11-09 記録）
- ~~シードを `Params` に入れるか~~ → `Params` に入れた（ステップ3）
- ~~`Params::default()` の `p_viol`~~ → モデルの値（3.0）にした。`test_run` が「罰なし」を使う箇所は `p_viol = 0.0` を明示（2026-11-07）
- 乱数: `rand` 0.10 の API（`RngExt`）。`StdRng` + `SeedableRng::seed_from_u64` で固定済み（2026-10-30）
- ~~SPAM_P の正確な定義~~ → 使わない（2026-11-15。偽通報は学習で出る）。用語集・変数一覧の SPAM_P は参考値として残す
