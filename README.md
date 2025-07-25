# VibeLang in Rust

Create programmatically any agent you need from an annotated payload, using VibeLang.

VibeLang is a format to describe LLM interactions to generic clients, base on **Meaning Typed Prompting*. As presented in [this paper](https://arxiv.org/pdf/2410.18146).

## Usage

It works for now it only works with Ollama but simple clients to any OpenAI-style API could be implemented.

1. Create your resource description file or string (see `examples/`)
2. generate automatically Rust code running `cargo run -- your-file.vibe --output-dir ./generated`
3. modify the `generated/main.rs` to make the desired calls to the LLM using the pregenerated code
4. `cd generate && cargo run`. Enjoy 

## Build
```
$ cargo build
```
Run an example:
```
$ cargo run -- examples/knowledge_retrieval.vibe --output-dir ./generated
```
Run tests:
```
$ cargo test
# OR
$ cargo test --test test_unit_extra
```

### install ollama

For now this only support Ollama.

```
$ curl -fsSL https://ollama.com/install.sh | sh
```
Check `localhost:11434` in your browser.

### models selection

```
$ ollama pull <MODEL>
$ ollama serve
$ ollama run <MODEL>
```
Models available: [link](https://ollama.com/library).

Set:
```
export OLLAMA_MODEL=llama3.1
```
or any other model you have downloaded to change model.
