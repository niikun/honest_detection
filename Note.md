## Params Modeについて
- この実験は多くの試行回数を繰り返すことになる。ただし、条件を変えるときのために、Modeはenumで3つの値が取れるようにして、外に切り分ける。

## 罰がないとき、1000ラウンド後に違反を選ぶようになるのはなぜか。Q値と ε-greedy の両方の言葉を使ってください。
行動がviolatedのときしか、報酬が出ない。そのため、violatedを選んだ場合が、compliantを選ぶより期待値が高くなる。また、ε-greedyで一定の確率（0.3）でrandomに選ぶため、qvalue.compliantも0に近づいていく。
## test_run が成功することは、その説明のうち何を裏づけているか。裏づけていない部分があれば、それも教えてください。
1000回施行を繰り返した結果、qvalue.violated>qvalue.compliantであることとqvalue.violatedがgainの1.0を超えないこと
しかし、行動については検証していない。

## 見つかったときのペナルティPがある場合
rewardの期待値 = 1 - detect_p * p_viol
detect_p = 0.5, p_viol = 2のとき、violatedのrewardの期待値0はcompliantのreward期待値と同じになる

detect_p = 0.5, p_viol = 3のとき、期待値は-0.5
qvalue.violated_caseも-0.5になる。
