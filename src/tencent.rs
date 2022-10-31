use chrono::Utc;
use serde::{Serialize, Deserialize};
use reqwest::Version;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, HOST};
use crate::{Context, TransError, TransRes, TransResult};
use crate::common::{hex_byte, sha2_hmac, sha2_str};

#[derive(Serialize, Debug)]
pub struct TencentReq {
  #[serde(rename(serialize = "SourceText"))]
  src: String,

  #[serde(rename(serialize = "ProjectId"))]
  project_id: i32,

  #[serde(rename(serialize = "Target"))]
  target: String,

  #[serde(rename(serialize = "Source"))]
  source: String
}

impl TencentReq {
  fn create(src: &str, source: &str, target: &str) -> Self {
    return TencentReq {
      src: String::from(src),
      project_id: 0,
      target: String::from(target),
      source: String::from(source)
    };
  }
}

#[derive(Deserialize, Debug)]
pub struct TencentRes {
  #[serde(rename(deserialize = "Response"))]
  data: TencentInnerRes
}

impl TencentRes {
  fn to_trans_res(&self) -> TransRes {
    return TransRes::create(self.data.dst.clone());
  }
}

#[derive(Deserialize, Debug)]
struct TencentInnerRes {
  #[serde(rename(deserialize = "TargetText"))]
  dst: String,

  // #[serde(rename(deserialize = "Source"))]
  // source: String,

  // #[serde(rename(deserialize = "Target"))]
  // target: String,

  // #[serde(rename(deserialize = "RequestId"))]
  // request_id: String

}

pub async fn trans(context: &Context) -> TransResult<TransRes> {
  let req = context.req_ref();
  let client = context.client_ref();
  let current_time = Utc::now();
  let current_data = current_time.format("%Y-%m-%d").to_string();
  let current_instant = current_time.timestamp().to_string();
  let body = TencentReq::create(req.src_ref(), req.source_ref(), req.target_ref());
  let body_str = serde_json::to_string(&body).map_err(|_| TransError::SerdeError)?;

  let mut sign_request = Vec::with_capacity(6);
  sign_request.push("POST");
  sign_request.push("/");
  sign_request.push("");
  sign_request.push("content-type:application/json; charset=utf-8\nhost:tmt.tencentcloudapi.com\n");
  sign_request.push("content-type;host");
  let body_hash = hex_byte(sha2_str(body_str.as_ref()).as_slice());
  sign_request.push(body_hash.as_ref());
  let sign_request_str = sign_request.join("\n");

  let mut hmac_data = Vec::with_capacity(4);
  hmac_data.push("TC3-HMAC-SHA256");
  hmac_data.push(current_instant.as_ref());
  let credential_scope = format!("{}/tmt/tc3_request", current_data.as_str());
  hmac_data.push(credential_scope.as_ref());
  let request_hash = hex_byte(sha2_str(sign_request_str.as_str()).as_slice());
  hmac_data.push(request_hash.as_ref());
  let hmac_data_str = hmac_data.join("\n");

  let tencent_secret = context.tencent_secret()?;
  let mut hmac_key = sha2_hmac(format!("TC3{}", tencent_secret).as_bytes(), current_data.as_bytes())?;
  hmac_key = sha2_hmac(hmac_key.as_slice(), "tmt".as_bytes())?;
  hmac_key = sha2_hmac(hmac_key.as_slice(), "tc3_request".as_bytes())?;
  let signature = sha2_hmac(hmac_key.as_slice(), hmac_data_str.as_bytes())?;

  let mut authorization = String::from("TC3-HMAC-SHA256 Credential=");
  authorization.push_str(context.tencent_key()?);
  authorization.push_str("/");
  authorization.push_str(current_data.as_ref());
  authorization.push_str("/tmt/tc3_request");
  authorization.push_str(", SignedHeaders=content-type;host");
  authorization.push_str(", Signature=");
  authorization.push_str(hex_byte(signature.as_slice()).as_ref());

  let mut headers = HeaderMap::with_capacity(7);
  headers.insert(HeaderName::from_static("x-tc-action"), HeaderValue::from_str("TextTranslate").unwrap());
  headers.insert(HeaderName::from_static("x-tc-version"), HeaderValue::from_str("2018-03-21").unwrap());
  headers.insert(HeaderName::from_static("x-tc-region"), HeaderValue::from_str("ap-shanghai").unwrap());
  headers.insert(HeaderName::from_static("x-tc-timestamp"), HeaderValue::from_str(current_instant.as_ref()).unwrap());
  headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json; charset=utf-8").unwrap());
  headers.insert(HOST, HeaderValue::from_str("tmt.tencentcloudapi.com").unwrap());
  headers.insert(AUTHORIZATION, HeaderValue::from_str(authorization.as_ref()).unwrap());

  let mut request_builder = client.post(context.tencent_address());
  request_builder = request_builder.version(Version::HTTP_11);
  request_builder = request_builder.headers(headers);
  request_builder = request_builder.body(body_str);
  let response = request_builder.send()
    .await.map_err(|_| TransError::RequestError)?
    .json::<TencentRes>()
    .await.map_err(|_| TransError::ResponseError)?
    .to_trans_res();
  return Ok(response);
}