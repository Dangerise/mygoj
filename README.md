# It's MYGOJ

An online judge that focus on fast, easy and reliable .

## Development Guide 

### Compilation

由于使用了`sqlx`， 首先应初始化数据库

```bash
python init.py
```

这会生成一个空的数据库，使得 `sqlx` 的查询语句可以编译。

项目的前端使用了 `dioxus` 作为框架，因此你应先安装 `dioxus-cli`

在 arch linux 中，你可以方便地安装

```bash
sudo pacman -S dioxus-cli
```

或者通过 `cargo` 安装

```bash
cargo install dioxus-cli
```

由于后端会直接把前端打包进可执行文件，所以你要先打包前端

```bash
dx bundle -p front
```

或加上 `--release` 

再运行以下命令以初始化存储目录

```bash
cargo r -p server -- init
```

