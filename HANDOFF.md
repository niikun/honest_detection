# honest_detection 学習ハンドオフ

作成: 2026-10-28 / 最終更新: 2026-11-12
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
- 2026-11-12: **`round` の最初のテスト完了**（`cargo test` 14件成功。`5f2a304` まではコミット済み、最後の整理分は未コミット）
  - 方針: A（`round` を実際に呼び、返り値と Q値の**つじつま**を確かめる）に決着。乱数に頼らず場面を決めるには、**初期値で固定**する: `epsilon = 0.0`（探索なし）、`detect_p = 0.0`（誰も `See` にならない）、`Qvalue` / `Qreport` の大小で行動・反応を狙う。手で作った `actions` 等は `round` に渡せない（`round` が中で作る）ので、前半の手作り部分は `round` の確認には不要
  - 済: `test_round`（n=2）。`agent0` は Q値が引き分け → `get_action` の `>` が偽で **`Violated`**（引き分けは `Violated` 側）、`Qreport` 引き分けは `>=` で **`Report`** → 偽通報。`agent1` は `compliant_case = 1.0` で守る、`ignore_notsee = 1.0` で**無視**。期待値: `agent0.violated_case` = 0.1（報酬 `gain` ×0.1）、`agent0.report_notsee` = -0.02（`-cost` ×0.1）、`agent1.compliant_case` = 0.9、`agent1.ignore_notsee` = 0.9（**無視は `-cost` がかからない**＝報酬 0.0）。選ばなかった枝は全部動かない。戻り値も `assert_eq!`（`actions == [Violated, Compliant]`、`report_state == [ReportFalse, Ignore]`）
  - つまずきと解決: ①`params` を作る前に `params.seed` を使った ②期待値が前の設定（罰を受ける場面）のまま残っていた → 今の設定で `round` の中を追って計算し直した ③`ignore_see + multiplier` の `+` の打ち間違い（`*` にする） ④`agent1` の期待値 0.88 → 無視の報酬は 0.0 なので 0.9（`get_report_reward` の `Ignore => 0.0`）⑤`Agent` を `Copy` にしない理由: 暗黙の複製で、更新したのに元が変わらないバグが起きやすい。`Clone`（明示的な `.clone()`）で十分
  - 残る（任意）: 前半の手作りの `actions` / `reactions` / `report_states` / `observes` / `get_rewards` と `println!` の整理。`Agent { ... }` の2人分の重複（`Agent::new` + 上書き）。同じ丸めの式を小さな関数に
- 次の一歩（ステップ4 ③-4 の続き）:
  1. `See` を起こす場面（`detect_p = 1.0`、本物の通報、罰 `-p_viol`）での `round` のテスト（retro も入れると `r_true` / `p_false` まで通せる）
  2. カウンタをどうするか決める（上の注意）。`params.n` と `agents.len()` の前提の確認も
  3. テスト: none 条件で通報がほぼゼロになること（`round` を何千回か回す。ステップ5の `run_n` の前段）
  4. 余力があれば `Mode::Immediate`（R_IMM）の分岐を足す（ステップ5の3条件比較で必要）
- amnesia との接続: 違反・通報・評判は Twin Towns にまだない。「この街で何を違反とするか」は未決（v0.1 の後で検討）
- 見積もり（11/10 時点）: 部品は揃った（v0.1 の7割程度）。v0.1 まであと3〜4回（ラウンド関数1〜2回＋none のテストと `Immediate` 0.5〜1回＋ステップ5 1.5〜2回）
- 見積もり（11/12 に見直し）: `round` とその最初のテストが済んだ。ステップ4 の残りは `See` の場面のテストと none の通報ゼロのテスト（0.5〜1回）。v0.1 まであと2〜3回（＋`Immediate` 0.5回＋ステップ5 1.5〜2回）。進み具合は v0.1 の7割強〜8割弱。（11/11 は「あと2〜4回」）結果表に合わせる調整と `SPAM_P` は外れやすい。（11/10 は「あと3〜4回」、11/5 は「あと5〜8回」）11/12〜12/10 は応用編2と並走。年内の範囲（v0.1＋API の最初の口まで、など）はステップ4終了時に見直す

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

- ~~`mode` を `Params` に入れるか~~ → 10/28 に「外に出す」と決めたが、**`Params` に入れる形に変更**（理由: 実験の設定を `Params` 1つで全部管理できる。シードも入れたので「`Params`＝1回の実験の設定一式」。3条件の比較は `p_viol` と同じく clone して `mode` を書き換える）（2026-11-09 記録）
- ~~シードを `Params` に入れるか~~ → `Params` に入れた（ステップ3）
- ~~`Params::default()` の `p_viol`~~ → モデルの値（3.0）にした。`test_run` が「罰なし」を使う箇所は `p_viol = 0.0` を明示（2026-11-07）
- 乱数: `rand` 0.10 の API（`RngExt`）。`StdRng` + `SeedableRng::seed_from_u64` で固定済み（2026-10-30）
- SPAM_P の正確な定義（用語集参照。**ステップ4 ③ の「報酬を決める」で決める**）
