# Loom

**Layer-Oriented Orchestration Machine**

A framework/runtime and set of binaries for building layered pipelines with
intelligence models.

<pre>
├── <a href="./README.md"><b>&lt;&lt;root&gt;&gt;</b></a> 👈
├── <a href="./libs/README.md">▸ 0. Libraries</a>
├── <a href="./docs/1.memory.md">▸ 1. Memory</a>
├── <a href="./docs/2.ingestion.md">▸ 2. Ingestion</a>
├── <a href="./docs/3.guards.md">▸ 3. Guards</a>
├── <a href="./docs/4.recall.md">▸ 4. Recall</a>
├── <a href="./docs/5.classification.md">▸ 5. Classification</a>
└── <a href="./docs/README.md">▾ 6. Research/</a>
    ├── <a href="./docs/loom/README.md">▸ 6.1 Loom/</a>
    ├── <a href="./docs/reference/README.md">▸ 6.2 Reference/</a>
    └── <a href="./docs/analysis/README.md">▸ 6.3 Analysis/</a>
</pre>

## Datasets

The following conversation datasets are used for training and evaluation:

| Dataset | Samples | Size | Description |
|---------|---------|------|-------------|
| [DailyDialog](http://yanran.li/dailydialog.html) | 102,979 | 72 MB | Multi-turn dialogues with emotion and dialog act labels, covering various topics about daily life |
| [Multi-Session Chat](https://huggingface.co/datasets/nayohan/multi_session_chat) | 3,372 | 3.6 MB | Human-human conversations across multiple sessions with persona information |
| [MSC-Self-Instruct](https://huggingface.co/datasets/MemGPT/MSC-Self-Instruct) | 5,964 | 14 MB | Multi-session conversations with persona-grounded dialogue for memory-augmented agents |
| [LongMemEval](https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned) | 500 | 239 MB | Memory evaluation benchmark with cleaned history sessions for long-term memory testing |
| [LoCoMo](https://huggingface.co/datasets/Percena/locomo-mc10) | 1,986 | 200 MB | Long conversation memory multiple-choice benchmark with 5 reasoning types |

To download and convert datasets to the samples format, run:

```bash
cargo scripts datasets fetch
```
