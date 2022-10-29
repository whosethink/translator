#### 简介
Little translation tool, support [百度](https://fanyi-api.baidu.com/api/trans/product/desktop), [火山](https://console.volcengine.com/translate), [阿里](https://mt.console.aliyun.com/basic) 和 [腾讯](https://console.cloud.tencent.com/tmt).

#### 使用
```bash
translator --help
translator

USAGE:
    translator [OPTIONS] [WORD]...

ARGS:
    <WORD>...

OPTIONS:
    -c, --config <CONFIG>
    -h, --help               Print help information
    -s, --source <SOURCE>    [default: auto]
    -t, --target <TARGET>    [default: zh]
```

zh -> en
```bash
translator -t en 阳光总在风雨后
```

en -> zh
```bash
translator Empowering everyone to build reliable and efficient software
```

#### config

example:

1. ${project}/translator.toml 

path: 

1. ${config dir}/translator.toml (~/config/translator.toml)
2. ./translator.toml
3. specify by --config

