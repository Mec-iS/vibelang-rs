
```
$ cargo build
```
Run an example:
```
$ cargo run -- examples/knowledge_retrieval.vibe --output-dir ./generated
```


### install ollama

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