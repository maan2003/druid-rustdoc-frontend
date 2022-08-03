## Druid Rustdoc Frontend

A native rustdoc frontend using [druid] and [rustdoc-json].

![screenshot](https://user-images.githubusercontent.com/49202620/182473115-99a9bfad-4977-49dc-8e4d-2d99c8da6621.png)

## Run

A sample json is included in the repo `piet.json`.

```shell
cargo run -- piet.json
```

Note: you need "Source Code Pro" Font installed or change it in src/theme.rs:37

[druid]: https://github.com/linebender/druid
[rustdoc-json]: https://rust-lang.github.io/rfcs/2963-rustdoc-json.html
