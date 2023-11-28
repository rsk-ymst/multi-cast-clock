### 全順序マルチキャストシュミレータ―
<img src="https://github.com/rsk-ymst/multi-cast-clock/assets/81950820/6ff5c263-dd55-4719-936d-f1c917f74408.jpg" width="500">


### 開発環境
- rustc : 1.71.0 (8ede3aae2 2023-07-12)
- cargo : 1.71.0 (cfd3bbd8f 2023-06-08)
- make  : GNU Make 3.81


## 概説
- 当プロジェクトの構成要素は３つ
    - 論理クロックを持つレシーバ・プロセスA, B
    - オペレータ・プロセス X 1


### レシーバ・プロセスについて
- 各プロセスは、メインスレッドと論理クロック用スレッドを保持する。
    - メインレスレッド      --> メッセージの送受信を担う。
    - 論理クロック用スレッド --> 時刻を一定間隔で進める。
    - 時刻はスレッド間で共有される。
- メインスレッドは任意の処理に伴い、時刻を取得する。
    - スレッド間で共有する論理クロック値はデータレースが起こらないよう、Arc<Mutex<T>>型でラップする。
- メッセージの送受信はソケットを用い、マルチキャストで行う。
    - 模擬的な形ではなく、正式なマルチキャスト・アドレス登録の手順を踏む
- メッセージの送信はスレッドを用いて非同期に行う。
    - メインスレッドが送信時に子スレッドを生成し、それに処理を任せることで、同期的に待機せずメインループの受信元に戻る処理を実現した


### オペレータ・プロセスについて
- 各プロセスに対してリクエスト送る役割を持つ
- オペレータの送信処理はテスト関数として定義されており、簡易的に実行可能


### src構成
- clock.rs       : 論理クロックに関連する構造体・関数群が定義されている
- ipc.rs         : プロセス間通信に関連する構造体・関数群が定義されている
- operator.rs    : 各プロセスに対する操作リクエストを担う
- utils.rs       : 主にIO関連の関数が定義されている
- main.rs        : レシーバのエントリポイント


### ビルド＆実行
一連の動作を確認するためには、合計３つのターミナルウィンドウが必要である。

最初に、独立したプロセスを二つ起動させるため、
二つターミナルウィンドウを開き、以下のコマンドをそれぞれ実行

```
$ make A
```

```
$ make B
```

各プロセスが待機状態に入ったことを確認できたら、もう一つターミナルウィンドウを開き
以下のコマンドでオペレータ関数を実行させる。このコマンドによって、ほぼ同時刻にプロセスA, Bに対するリクエストが送られ、一連の処理が行われる。

```
$ make ope
```
