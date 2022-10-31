use reqwest::Version;
use serde::Deserialize;
use crate::common::{Context, TransRes, TransError, TransResult, hex_byte, md5_str};

#[derive(Deserialize, Debug)]
pub struct BaiduRes {
  // #[serde(rename(deserialize = "from"))]
  // source: String,

  // #[serde(rename(deserialize = "to"))]
  // target: String,

  #[serde(rename(deserialize = "trans_result"))]
  data: Vec<BaiduInnerRes>,
}

impl BaiduRes {
  fn to_trans_res(&self) -> TransRes {
    return TransRes::create(self.data.first().map(|res| res.dst.clone()).unwrap());
  }
}

#[derive(Deserialize, Debug)]
struct BaiduInnerRes {
  // src: String,

  dst: String,
}

pub async fn trans(context: & Context) -> TransResult<TransRes> {
  let req = context.req_ref();
  let client = context.client_ref();
  let mut sign_content = String::from(context.baidu_key()?);
  sign_content.push_str(req.src_ref());
  sign_content.push_str("9527");
  sign_content.push_str(context.baidu_secret()?);
  let sign = hex_byte(md5_str(sign_content.as_str()).as_slice());
  let query = [
    ("q", req.src_ref()),
    ("from", "auto"),
    ("to", req.target_ref()),
    ("salt", "9527"),
    ("appid", context.baidu_key()?),
    ("sign", sign.as_ref())
  ];
  let mut request_builder = client.get(context.baidu_address());
  request_builder = request_builder.version(Version::HTTP_11);
  request_builder = request_builder.query(&query);
  let response = request_builder.send()
    .await.map_err(|_| TransError::RequestError)?
    .json::<BaiduRes>()
    .await.map_err(|_| TransError::ResponseError)?
    .to_trans_res();
  return Ok(response);
}