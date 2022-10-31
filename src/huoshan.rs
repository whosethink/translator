use chrono::Utc;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, HOST};
use serde::{Serialize, Deserialize};
use crate::{Context, HeaderName, TransError, TransRes, TransResult};
use crate::common::{hex_byte, sha2_hmac, sha2_str};

#[derive(Serialize, Debug)]
pub struct HuoShanReq {
  #[serde(rename(serialize = "TargetLanguage"))]
  target: String,

  #[serde(rename(serialize = "TextList"))]
  data: Vec<String>,
}

impl HuoShanReq {
  fn create(src: &str, target: &str) -> Self {
    return HuoShanReq {
      target: String::from(target),
      data: vec![String::from(src)],
    };
  }
}

#[derive(Deserialize, Debug)]
pub struct HuoShanRes {
  #[serde(rename(deserialize = "TranslationList"))]
  data: Vec<HuoShanInnerRes>,
}

impl HuoShanRes {
  fn to_trans_res(&self) -> TransRes {
    return TransRes::create(self.data.first().map(|res| res.dst.clone()).unwrap());
  }
}

#[derive(Deserialize, Debug)]
struct HuoShanInnerRes {
  // #[serde(rename(deserialize = "DetectedSourceLanguage"))]
  // source: String,

  #[serde(rename(deserialize = "Translation"))]
  dst: String,
}

pub async fn trans(context: &Context) -> TransResult<TransRes> {
  let req = context.req_ref();
  let client = context.client_ref();
  let current_time = Utc::now();
  let current_data1 = current_time.format("%Y%m%d").to_string();
  let current_data2 = current_time.format("%Y%m%dT%H%M%SZ").to_string();
  let body = HuoShanReq::create(req.src_ref(), req.target_ref());
  let body_str = serde_json::to_string(&body).map_err(|_| TransError::SerdeError)?;

  let mut sign_request = Vec::with_capacity(6);
  sign_request.push("POST");
  sign_request.push("/");
  sign_request.push("Action=TranslateText&Version=2020-06-01");
  let canonical_headers = format!("content-type:application/json; charset=utf-8\nhost:open.volcengineapi.com\nx-date:{}\n", current_data2.as_str());
  sign_request.push(canonical_headers.as_str());
  sign_request.push("content-type;host;x-date");
  let body_hash = hex_byte(sha2_str(body_str.as_str()).as_slice());
  sign_request.push(body_hash.as_ref());
  let sign_request_str = sign_request.join("\n");

  let mut hmac_data = Vec::with_capacity(4);
  hmac_data.push("HMAC-SHA256");
  hmac_data.push(current_data2.as_str());
  let credential_scope = format!("{}/cn-north-1/translate/request", current_data1.as_str());
  hmac_data.push(credential_scope.as_ref());
  let request_hash = hex_byte(sha2_str(sign_request_str.as_str()).as_slice());
  hmac_data.push(request_hash.as_ref());
  let hmac_data_str = hmac_data.join("\n");

  let mut hmac_key = sha2_hmac(context.huoshan_secret()?.as_bytes(), current_data1.as_bytes())?;
  hmac_key = sha2_hmac(hmac_key.as_slice(), "cn-north-1".as_bytes())?;
  hmac_key = sha2_hmac(hmac_key.as_slice(), "translate".as_bytes())?;
  hmac_key = sha2_hmac(hmac_key.as_slice(), "request".as_bytes())?;
  let signature = sha2_hmac(hmac_key.as_slice(), hmac_data_str.as_bytes())?;

  let mut authorization = String::from("HMAC-SHA256 Credential=");
  authorization.push_str(context.huoshan_key()?);
  authorization.push_str("/");
  authorization.push_str(credential_scope.as_str());
  authorization.push_str(", SignedHeaders=content-type;host;x-date");
  authorization.push_str(", Signature=");
  authorization.push_str(hex_byte(signature.as_slice()).as_ref());

  let mut headers = HeaderMap::with_capacity(4);
  headers.insert(HeaderName::from_static("x-date"), HeaderValue::from_str(current_data2.as_str()).unwrap());
  headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json; charset=utf-8").unwrap());
  headers.insert(HOST, HeaderValue::from_str("open.volcengineapi.com").unwrap());
  headers.insert(AUTHORIZATION, HeaderValue::from_str(authorization.as_str()).unwrap());

  let query = [
    ("Action", "TranslateText"),
    ("Version", "2020-06-01")
  ];
  let mut request_builder = client.post(context.huoshan_address());
  request_builder = request_builder.query(&query);
  request_builder = request_builder.body(body_str);
  request_builder = request_builder.headers(headers);
  let response = request_builder.send()
    .await.map_err(|_| TransError::RequestError)?
    .json::<HuoShanRes>()
    .await.map_err(|_| TransError::ResponseError)?
    .to_trans_res();
  return Ok(response);
}