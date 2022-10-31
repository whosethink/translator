use chrono::Utc;
use reqwest::Version;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, DATE, HeaderMap, HeaderValue, HOST};
use serde::{Serialize, Deserialize};
use crate::{Context, HeaderName, TransError, TransRes, TransResult};
use crate::common::{base64_byte, md5_str, sha1_hmac};

#[derive(Serialize, Debug)]
pub struct AliYunReq {
  #[serde(rename(serialize = "FormatType"))]
  format: String,

  #[serde(rename(serialize = "SourceLanguage"))]
  source: String,

  #[serde(rename(serialize = "TargetLanguage"))]
  target: String,

  #[serde(rename(serialize = "SourceText"))]
  src: String,

  #[serde(rename(serialize = "Scene"))]
  scene: String,
}

impl AliYunReq {
  fn create(src: &str, source: &str, target: &str) -> Self {
    return AliYunReq {
      format: String::from("text"),
      source: String::from(source),
      target: String::from(target),
      src: String::from(src),
      scene: String::from("general")
    }
  }
}

#[derive(Deserialize, Debug)]
pub struct AliYunRes {

  // #[serde(rename(deserialize = "RequestId"))]
  // request_id: String,

  // #[serde(rename(deserialize = "Code"))]
  // code: String,

  #[serde(rename(deserialize = "Data"))]
  data: AliYunInnerRes

}

impl AliYunRes {
  fn to_trans_res(&self) -> TransRes {
    return TransRes::create(self.data.dst.clone());
  }
}

#[derive(Deserialize, Debug)]
struct AliYunInnerRes {

  #[serde(rename(deserialize = "Translated"))]
  dst: String,
}

pub async fn trans(context: &Context) -> TransResult<TransRes> {
  let req = context.req_ref();
  let client = context.client_ref();

  let current_time = Utc::now();
  let current_time_str: String = current_time.to_rfc2822();
  let nonce = current_time.timestamp().to_string();

  let body = AliYunReq::create(req.src_ref(), req.source_ref(), req.target_ref());
  let body_str = serde_json::to_string(&body).unwrap();
  let body_hash = base64_byte(md5_str(body_str.as_str()).as_slice());
  let mut sign_request = Vec::with_capacity(8);
  sign_request.push("POST");
  sign_request.push("application/json");
  sign_request.push(body_hash.as_str());
  sign_request.push("application/json; charset=utf-8");
  sign_request.push(current_time_str.as_str());
  sign_request.push("x-acs-signature-method:HMAC-SHA1");
  let signature_nonce = format!("x-acs-signature-nonce:{}", nonce);
  sign_request.push(signature_nonce.as_str());
  sign_request.push("/api/translate/web/general");
  let sign_request_str = sign_request.join("\n");

  let signature = base64_byte(sha1_hmac(context.aliyun_secret()?.as_bytes(), sign_request_str.as_bytes())?.as_slice());
  let authorization = format!("acs {}:{}", context.aliyun_key()?, signature);
  let mut headers = HeaderMap::with_capacity(5);
  headers.insert(HeaderName::from_static("content-md5"), HeaderValue::from_str(body_hash.as_str()).unwrap());
  headers.insert(HeaderName::from_static("x-acs-signature-nonce"), HeaderValue::from_str(nonce.as_str()).unwrap());
  headers.insert(HeaderName::from_static("x-acs-signature-method"), HeaderValue::from_str("HMAC-SHA1").unwrap());
  headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json; charset=utf-8").unwrap());
  headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());
  headers.insert(HOST, HeaderValue::from_str("mt.cn-hangzhou.aliyuncs.com").unwrap());
  headers.insert(DATE, HeaderValue::from_str(current_time_str.as_str()).unwrap());
  headers.insert(AUTHORIZATION, HeaderValue::from_str(authorization.as_str()).unwrap());

  let mut request_builder = client.post(context.aliyun_address());
  request_builder = request_builder.version(Version::HTTP_11);
  request_builder = request_builder.headers(headers);
  request_builder = request_builder.body(body_str);
  let response = request_builder.send()
    .await.map_err(|_| TransError::RequestError)?
    .json::<AliYunRes>()
    .await.map_err(|_| TransError::ResponseError)?
    .to_trans_res();
  return Ok(response);
}