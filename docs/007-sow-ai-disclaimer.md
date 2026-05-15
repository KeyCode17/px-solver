# 007 — LLM / AI Accuracy Disclaimer

`px-solver` does not use LLMs at runtime. Where AI coding assistants are used during development, generated code is reviewed and tested by the maintainer before merge. The behavior of the solver itself is deterministic with respect to its inputs (modulo the inherent variability of network and PX challenge responses), and the success metric is empirical (canary solve rate). No claims of correctness are made about LLM-generated planning or code in this repository beyond what passes CI.
