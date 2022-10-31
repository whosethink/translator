extern crate core;

mod baidu;
mod common;
mod aliyun;
mod huoshan;
mod tencent;

use std::future::Future;
use std::path::{Path, PathBuf};
use clap::Parser;
use futures::FutureExt;
use reqwest::Client;
use reqwest::header::HeaderName;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;
use crate::common::{Config, Context, TransError, TransReq, TransRes, TransResult};

#[tokio::main]
async fn main() {
  let command: TransCommand = TransCommand::parse();
  if let Err(error) = inner_main(&command).await {
    println!("{:?}", error);
  }
  std::process::exit(0);
}

async fn inner_main(command: &TransCommand) -> TransResult<()> {
  let (tx, mut rx) = tokio::sync::mpsc::channel(4);
  let context = prepare_trans(&command).await?;
  let mut trans_tasks = Vec::with_capacity(4);
  if context.baidu_enabled() {
    trans_tasks.push(trans(" BaiDu ", tx.clone(), baidu::trans(&context)).boxed());
  }
  if context.huoshan_enabled() {
    trans_tasks.push(trans("HuoShan", tx.clone(), huoshan::trans(&context)).boxed());
  }
  if context.aliyun_enabled() {
    trans_tasks.push(trans(" AliYun", tx.clone(), aliyun::trans(&context)).boxed());
  }
  if context.tencent_enabled() {
    trans_tasks.push(trans("Tencent", tx.clone(), tencent::trans(&context)).boxed());
  }
  { tx; }
  futures::future::join_all(trans_tasks).await;
  while let Some((name, res)) = rx.recv().await {
    print_trans_res(name.as_str(), res).await;
  }
  Ok(())
}

async fn trans<F>(name: &str, sender: Sender<(String, TransResult<TransRes>)>, future: F) -> TransResult<()>
  where
    F: Future<Output=TransResult<TransRes>>,
{
  let res = future.await;
  return sender.send((name.to_string(), res)).await.map_err(|_| TransError::ChannelError);
}

async fn print_trans_res(name: &str, res: TransResult<TransRes>) {
  let mut writer = tokio::io::BufWriter::new(tokio::io::stdout());
  match res {
    Ok(answer) => {
      let _ = writer.write_all(
        format!("{}: {}\n", console::style(name).yellow(), console::style(answer.result()).white()).as_bytes()
      ).await;
    }
    Err(error) => {
      let _ = writer.write_all(
        format!("{}: {:?}\n", console::style(name).yellow(), console::style(error).red()).as_bytes()
      ).await;
    }
  }
  let _ = writer.flush().await;
}

async fn prepare_trans(command: &TransCommand) -> TransResult<Context> {
  let config = parse_config(command).await.map_err(|_| TransError::ParseConfig)?;
  return Ok(Context::create(command.trans_req(), Client::new(), config));
}

async fn parse_config(command: &TransCommand) -> TransResult<Config> {
  let config = if command.config_path().is_some() {
    let config_path = command.config_path().unwrap();
    config_from_command_path(config_path).await
  } else {
    config_from_default_path().await
  };
  return config;
}

async fn config_from_command_path<P: AsRef<Path>>(path: P) -> TransResult<Config> {
  let meta = tokio::fs::metadata(path.as_ref())
    .await.map_err(|_| TransError::ParseConfig)?;
  if meta.is_dir() {
    return Err(TransError::ParseConfig);
  }
  return parse_config_from_file(path).await;
}

async fn config_from_default_path() -> TransResult<Config> {
  let config_path = dirs::config_dir()
    .map(|path| path.join("translator.toml"))
    .unwrap_or(PathBuf::from("./translator.toml"));
  if tokio::fs::metadata(config_path.as_path()).await.is_ok() {
    return parse_config_from_file(config_path.as_path()).await;
  }
  return Err(TransError::ParseConfig);
}

async fn parse_config_from_file<P: AsRef<Path>>(path: P) -> TransResult<Config> {
  let content = tokio::fs::read(path)
    .await.map_err(|_| TransError::ParseConfig)?;
  return toml::from_slice(content.as_slice()).map_err(|_| TransError::ParseConfig);
}

#[derive(Debug, Parser)]
struct TransCommand {
  word: Vec<String>,

  #[clap(long = "source", short = 's', default_value = "auto")]
  source: String,

  #[clap(long = "target", short = 't', default_value = "zh")]
  target: String,

  #[clap(long = "config", short = 'c')]
  config: Option<PathBuf>,

  // #[clap(long = "log-level", default_value = "warn")]
  // log_level: Level,
}

impl TransCommand {
  fn trans_req(&self) -> TransReq {
    let src = self.word.join(" ");
    return TransReq::create(src.as_str(), self.source.as_str(), self.target.as_str());
  }

  fn config_path(&self) -> Option<PathBuf> {
    return self.config.clone();
  }
}