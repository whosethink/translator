use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug)]
pub enum TransError {
  RequestError,
  ResponseError,
  ParseConfig,
  BaiduConfig,
  TencentConfig,
  HuoShanConfig,
  AliYunConfig,
  SerdeError,
  HmacError,
  ChannelError
}

pub type TransResult<T> = Result<T, TransError>;

pub struct TransReq {
  src: String,

  source: String,

  target: String
}

impl TransReq {
  pub fn create(src: &str, source: &str, target: &str) -> Self {
    return TransReq {
      src: String::from(src),
      source: String::from(source),
      target: String::from(target),
    };
  }
}

impl<'a> TransReq {
  pub fn target_ref(&'a self) -> &'a str {
    return self.target.as_ref();
  }

  pub fn source_ref(&'a self) -> &'a str {
    return self.source.as_ref();
  }

  pub fn src_ref(&'a self) -> &'a str {
    return self.src.as_ref();
  }
}

#[derive(Debug)]
pub struct TransRes {
  result: String,
}

impl TransRes {
  pub fn create(result: String) -> Self {
    return TransRes {
      result
    };
  }

  pub fn result(&self) -> &str {
    return self.result.as_str();
  }
}

pub struct Context {
  req: TransReq,
  client: Client,
  config: Config,
}

impl Context {
  pub fn create(req: TransReq, client: Client, config: Config) -> Self {
    return Context {
      req,
      client,
      config
    };
  }

  pub fn req_ref(&self) -> &TransReq {
    return &self.req;
  }

  pub fn client_ref(&self) -> &Client {
    return &self.client;
  }
}

const BAIDU_ADDRESS: &'static str = "https://fanyi-api.baidu.com/api/trans/vip/translate";
const ALIYUN_ADDRESS: &'static str = "https://mt.cn-hangzhou.aliyuncs.com/api/translate/web/general";
const HUOSHAN_ADDRESS: &'static str = "https://open.volcengineapi.com";
const TENCENT_ADDRESS: &'static str = "https://tmt.tencentcloudapi.com";

impl<'a> Context {
  pub fn baidu_enabled(&self) -> bool {
    return self.config.baidu.is_some();
  }

  pub fn baidu_address(&'a self) -> &'a str {
    return BAIDU_ADDRESS;
  }

  pub fn baidu_key(&'a self) -> TransResult<&'a str> {
    return self.config.baidu.as_ref().map(|c| c.key.as_str()).ok_or(TransError::BaiduConfig);
  }

  pub fn baidu_secret(&'a self) -> TransResult<&'a str> {
    return self.config.baidu.as_ref().map(|c| c.secret.as_str()).ok_or(TransError::BaiduConfig);
  }

  pub fn aliyun_enabled(&self) -> bool {
    return self.config.aliyun.is_some();
  }
  pub fn aliyun_address(&'a self) -> &'a str {
    return ALIYUN_ADDRESS;
  }

  pub fn aliyun_key(&'a self) -> TransResult<&'a str> {
    return self.config.aliyun.as_ref().map(|c| c.key.as_str()).ok_or(TransError::AliYunConfig);
  }

  pub fn aliyun_secret(&'a self) -> TransResult<&'a str> {
    return self.config.aliyun.as_ref().map(|c| c.secret.as_str()).ok_or(TransError::AliYunConfig);
  }

  pub fn huoshan_enabled(&self) -> bool {
    return self.config.huoshan.is_some();
  }
  pub fn huoshan_address(&'a self) -> &'a str {
    return HUOSHAN_ADDRESS;
  }

  pub fn huoshan_key(&'a self) -> TransResult<&'a str> {
    return self.config.huoshan.as_ref().map(|c| c.key.as_str()).ok_or(TransError::HuoShanConfig);
  }

  pub fn huoshan_secret(&'a self) -> TransResult<&'a str> {
    return self.config.huoshan.as_ref().map(|c| c.secret.as_str()).ok_or(TransError::HuoShanConfig);
  }

  pub fn tencent_enabled(&self) -> bool {
    return self.config.tencent.is_some();
  }
  pub fn tencent_address(&'a self) -> &'a str {
    return TENCENT_ADDRESS;
  }

  pub fn tencent_key(&'a self) -> TransResult<&'a str> {
    return self.config.tencent.as_ref().map(|c| c.key.as_str()).ok_or(TransError::TencentConfig);
  }

  pub fn tencent_secret(&'a self) -> TransResult<&'a str> {
    return self.config.tencent.as_ref().map(|c| c.secret.as_str()).ok_or(TransError::TencentConfig);
  }

}

#[derive(Debug, Deserialize)]
pub struct Config {
  baidu: Option<BaiduConfig>,
  aliyun: Option<AliYunConfig>,
  huoshan: Option<HuoShanConfig>,
  tencent: Option<TencentConfig>
}

#[derive(Debug, Deserialize)]
struct BaiduConfig {
  key: String,

  secret: String,
}

#[derive(Debug, Deserialize)]
struct AliYunConfig {
  key: String,

  secret: String,
}

#[derive(Debug, Deserialize)]
struct HuoShanConfig {
  key: String,

  secret: String,
}

#[derive(Debug, Deserialize)]
struct TencentConfig {
  key: String,

  secret: String,
}

pub fn md5_str(str: &str) -> Vec<u8> {
  use md5::Digest;
  let mut hasher = md5::Md5::new();
  hasher.update(str.as_bytes());
  return hasher.finalize().to_vec();
}

pub fn sha2_str(str: &str) -> Vec<u8> {
  use sha2::Digest;
  let mut hasher = sha2::Sha256::new();
  hasher.update(str.as_bytes());
  return hasher.finalize().to_vec();
}

type HmacSha2 = Hmac<sha2::Sha256>;
pub fn sha2_hmac(key: &[u8], data: &[u8]) -> TransResult<Vec<u8>> {
  let mut hmac = HmacSha2::new_from_slice(key)
    .map_err(|_| TransError::HmacError)?;
  hmac.update(data);
  return Ok(hmac.finalize().into_bytes().to_vec());
}

type HmacSha1 = Hmac<sha1::Sha1>;
pub fn sha1_hmac(key: &[u8], data: &[u8]) -> TransResult<Vec<u8>> {
  let mut hmac = HmacSha1::new_from_slice(key)
    .map_err(|_| TransError::HmacError)?;
  hmac.update(data);
  return Ok(hmac.finalize().into_bytes().to_vec());
}

pub fn hex_byte(data: &[u8]) -> String {
  return hex::encode(data);
}

pub fn base64_byte(data: &[u8]) -> String {
  return base64::encode(data);
}
