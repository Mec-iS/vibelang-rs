
```
$ RUST_BACKTRACE=1 cargo test tests::integration --
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