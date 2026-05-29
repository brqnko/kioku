# MioTTS Docker

[Aratako/MioTTS-GGUF](https://huggingface.co/Aratako/MioTTS-GGUF) 0.6B を Docker で動かすための構成。
llama.cpp サーバー + MioTTS-Inference サーバーを1コンテナで起動する。

## クイックスタート

```bash
# ビルド
docker build -t miotts .

# 起動 (CPU)
docker run -p 8001:8001 miotts

# 起動 (GPU)
docker run --gpus all -p 8001:8001 -e DEVICE=cuda miotts
```

初回起動時に GGUF モデル (~650MB Q8_0) と MioCodec が HuggingFace から自動ダウンロードされる。

```bash
# キャッシュを永続化して再ダウンロードを防ぐ
docker run -p 8001:8001 -v miotts-cache:/root/.cache miotts
```

## 環境変数

| 変数 | デフォルト | 説明 |
|------|-----------|------|
| `MODEL_REPO` | `Aratako/MioTTS-GGUF` | HuggingFace リポジトリ |
| `MODEL_FILE` | `MioTTS-0.6B-Q8_0.gguf` | GGUF ファイル名 |
| `LLM_PORT` | `8000` | llama.cpp サーバーのポート (内部) |
| `TTS_PORT` | `8001` | MioTTS API のポート |
| `DEVICE` | `cpu` | MioCodec のデバイス (`cpu` / `cuda`) |

量子化バリエーション:

| ファイル | サイズ |
|---------|-------|
| `MioTTS-0.6B-BF16.gguf` | 1.22 GB |
| `MioTTS-0.6B-Q8_0.gguf` | 653 MB |
| `MioTTS-0.6B-Q6_K.gguf` | 506 MB |
| `MioTTS-0.6B-Q4_K_M.gguf` | 408 MB |

## API

### ヘルスチェック

```
GET /health
```

```json
{"status": "ok"}
```

### プリセット一覧

```
GET /v1/presets
```

```json
{"presets": ["en_female", "en_male", "jp_female", "jp_male"]}
```

### 音声合成 (JSON)

```
POST /v1/tts
Content-Type: application/json
```

リクエスト:

```json
{
  "text": "こんにちは、世界。",
  "reference": {
    "type": "preset",
    "preset_id": "jp_female"
  }
}
```

全パラメータ:

```json
{
  "text": "合成するテキスト",
  "reference": {
    "type": "preset",
    "preset_id": "jp_female"
  },
  "llm": {
    "temperature": 0.8,
    "top_p": 1.0,
    "max_tokens": 700,
    "repetition_penalty": 1.0,
    "presence_penalty": 0.0,
    "frequency_penalty": 0.0
  },
  "output": {
    "format": "base64"
  },
  "best_of_n": {
    "enabled": false,
    "n": 1,
    "language": "auto"
  }
}
```

レスポンス:

```json
{
  "audio": "<base64エンコードされたWAVデータ>",
  "format": "base64",
  "sample_rate": 24000,
  "token_count": 123,
  "timings": {
    "llm_sec": 0.5,
    "parse_sec": 0.01,
    "codec_sec": 0.2,
    "total_sec": 0.71,
    "best_of_n_sec": null,
    "asr_sec": null
  },
  "normalized_text": "前処理後のテキスト"
}
```

### 音声合成 (ファイルアップロード)

```
POST /v1/tts/file
Content-Type: multipart/form-data
```

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| `text` | string | Yes | 合成テキスト |
| `reference_audio` | file | ※ | リファレンス音声 (.wav/.flac/.ogg, 最大20MB/20秒) |
| `reference_preset_id` | string | ※ | プリセットID |
| `temperature` | float | No | LLM temperature (デフォルト: 0.8) |
| `top_p` | float | No | Top-P サンプリング (デフォルト: 1.0) |
| `max_tokens` | int | No | 最大トークン数 (デフォルト: 700) |
| `repetition_penalty` | float | No | 繰り返しペナルティ (デフォルト: 1.0) |
| `output_format` | string | No | `"wav"` または `"base64"` |
| `best_of_n_enabled` | bool | No | Best-of-N 有効化 |
| `best_of_n_n` | int | No | 生成候補数 (最大8) |

※ `reference_audio` と `reference_preset_id` のどちらか一方が必須。

### curl 例

```bash
# プリセットで合成
curl -s http://localhost:8001/v1/tts \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "こんにちは",
    "reference": {"type": "preset", "preset_id": "jp_female"}
  }' | jq -r .audio | base64 -d > output.wav

# WAV形式で直接取得
curl -s http://localhost:8001/v1/tts \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "Hello, world.",
    "reference": {"type": "preset", "preset_id": "en_female"},
    "output": {"format": "wav"}
  }' -o output.wav

# リファレンス音声ファイルで合成
curl -s http://localhost:8001/v1/tts/file \
  -F 'text=こんにちは' \
  -F 'reference_audio=@reference.wav' \
  -F 'output_format=wav' \
  -o output.wav
```

## MioTTS-Inference サーバー設定

`run_server.py` に渡せるオプション。環境変数でも設定可。

### サーバー

| パラメータ | 環境変数 | デフォルト |
|-----------|---------|-----------|
| `--host` | `MIOTTS_HOST` | `0.0.0.0` |
| `--port` | `MIOTTS_PORT` | `8001` |
| `--log-level` | `MIOTTS_LOG_LEVEL` | `info` |

### LLM

| パラメータ | 環境変数 | デフォルト |
|-----------|---------|-----------|
| `--llm-base-url` | `MIOTTS_LLM_BASE_URL` | `http://localhost:8000/v1` |
| `--llm-api-key` | `MIOTTS_LLM_API_KEY` | - |
| `--llm-model` | `MIOTTS_LLM_MODEL` | 自動検出 |
| `--llm-timeout` | `MIOTTS_LLM_TIMEOUT` | `120` |

### サンプリング (環境変数のみ)

| 環境変数 | デフォルト |
|---------|-----------|
| `MIOTTS_LLM_TEMPERATURE` | `0.8` |
| `MIOTTS_LLM_TOP_P` | `1.0` |
| `MIOTTS_LLM_MAX_TOKENS` | `700` |
| `MIOTTS_LLM_REPETITION_PENALTY` | `1.0` |
| `MIOTTS_LLM_PRESENCE_PENALTY` | `0.0` |
| `MIOTTS_LLM_FREQUENCY_PENALTY` | `0.0` |

### コーデック

| パラメータ | 環境変数 | デフォルト |
|-----------|---------|-----------|
| `--codec-model` | `MIOTTS_CODEC_MODEL` | `Aratako/MioCodec-25Hz-44.1kHz-v2` |
| `--device` | `MIOTTS_DEVICE` | `cuda` |
| `--presets-dir` | `MIOTTS_PRESETS_DIR` | `presets` |

### Best-of-N

| パラメータ | 環境変数 | デフォルト |
|-----------|---------|-----------|
| `--best-of-n-enabled` | `MIOTTS_BEST_OF_N_ENABLED` | `false` |
| `--best-of-n-default` | `MIOTTS_BEST_OF_N_DEFAULT` | `1` |
| `--best-of-n-max` | `MIOTTS_BEST_OF_N_MAX` | `8` |
| `--best-of-n-language` | `MIOTTS_BEST_OF_N_LANGUAGE` | `auto` |

### 制約

| パラメータ | 環境変数 | デフォルト |
|-----------|---------|-----------|
| `--max-text-length` | `MIOTTS_MAX_TEXT_LENGTH` | `300` |
| `--max-reference-mb` | `MIOTTS_MAX_REFERENCE_MB` | `20` |
| `--allowed-audio-exts` | `MIOTTS_ALLOWED_AUDIO_EXTS` | `.wav,.flac,.ogg` |

## プリセット作成

カスタム音声プリセットの作成:

```bash
docker exec -it <container> uv run python scripts/generate_preset.py \
  --audio /path/to/reference.wav \
  --preset-id my_voice \
  --device cpu
```

## ライセンス

- MioTTS-0.6B: Apache 2.0
- デフォルトプリセット (jp_female, jp_male, en_female, en_male): 商用利用不可 (T5Gemma-TTS / Gemini 由来)
- MioTTS-Inference コード: MIT
