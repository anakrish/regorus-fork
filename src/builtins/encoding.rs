// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::ast::{Expr, Ref};
use crate::builtins;
use crate::builtins::utils::{ensure_args_count, ensure_string};
use crate::lexer::Span;
use crate::value::Value;

use std::collections::{BTreeMap, HashMap};

use anyhow::{bail, Context, Result};

pub fn register(m: &mut HashMap<&'static str, builtins::BuiltinFcn>) {
    #[cfg(feature = "base64")]
    {
        m.insert("base64.decode", (base64_decode, 1));
        m.insert("base64.encode", (base64_encode, 1));
        m.insert("base64.is_valid", (base64_is_valid, 1));
    }
    #[cfg(feature = "base64url")]
    {
        m.insert("base64url.decode", (base64url_decode, 1));
        m.insert("base64url.encode", (base64url_encode, 1));
        m.insert("base64url.encode_no_pad", (base64url_encode_no_pad, 1));
    }
    #[cfg(feature = "hex")]
    {
        m.insert("hex.decode", (hex_decode, 1));
        m.insert("hex.encode", (hex_encode, 1));
    }
    #[cfg(feature = "urlquery")]
    {
        m.insert("urlquery.decode_object", (urlquery_decode_object, 1));
    }
    m.insert("json.is_valid", (json_is_valid, 1));
    m.insert("json.marshal", (json_marshal, 1));
    m.insert("json.unmarshal", (json_unmarshal, 1));
    #[cfg(feature = "jsonschema")]
    {
        m.insert("json.match_schema", (json_match_schema, 2));
        m.insert("json.verify_schema", (json_verify_schema, 1));
    }

    #[cfg(feature = "yaml")]
    {
        m.insert("yaml.is_valid", (yaml_is_valid, 1));
        m.insert("yaml.marshal", (yaml_marshal, 1));
        m.insert("yaml.unmarshal", (yaml_unmarshal, 1));
    }
}

#[cfg(feature = "base64")]
fn base64_decode(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "base64.decode";
    ensure_args_count(span, name, params, args, 1)?;

    let encoded_str = ensure_string(name, &params[0], &args[0])?;
    let decoded_bytes = data_encoding::BASE64.decode(encoded_str.as_bytes())?;
    Ok(Value::String(
        String::from_utf8_lossy(&decoded_bytes).into(),
    ))
}

#[cfg(feature = "base64")]
fn base64_encode(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "base64.encode";
    ensure_args_count(span, name, params, args, 1)?;

    let string = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::String(
        data_encoding::BASE64.encode(string.as_bytes()).into(),
    ))
}

#[cfg(feature = "base64")]
fn base64_is_valid(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "base64.is_valid";
    ensure_args_count(span, name, params, args, 1)?;

    let encoded_str = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::Bool(
        data_encoding::BASE64.decode(encoded_str.as_bytes()).is_ok(),
    ))
}

#[cfg(feature = "base64url")]
fn base64url_decode(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "base64url.decode";
    ensure_args_count(span, name, params, args, 1)?;

    let encoded_str = ensure_string(name, &params[0], &args[0])?;
    let decoded_bytes = match data_encoding::BASE64URL.decode(encoded_str.as_bytes()) {
        Ok(b) => b,
        Err(_) => {
            #[cfg(feature = "base64url")]
            {
                data_encoding::BASE64URL_NOPAD.decode(encoded_str.as_bytes())?
            }
            #[cfg(not(feature = "base64url"))]
            {
                bail!(params[0].span().error("nor a valid url"));
            }
        }
    };

    Ok(Value::String(
        String::from_utf8_lossy(&decoded_bytes).into(),
    ))
}

#[cfg(feature = "base64url")]
fn base64url_encode(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "base64url.encode";
    ensure_args_count(span, name, params, args, 1)?;

    let string = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::String(
        data_encoding::BASE64URL.encode(string.as_bytes()).into(),
    ))
}

#[cfg(feature = "base64url")]
fn base64url_encode_no_pad(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "base64url.encode_no_pad";
    ensure_args_count(span, name, params, args, 1)?;

    let string = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::String(
        data_encoding::BASE64URL_NOPAD
            .encode(string.as_bytes())
            .into(),
    ))
}

#[cfg(feature = "hex")]
fn hex_decode(span: &Span, params: &[Ref<Expr>], args: &[Value], _strict: bool) -> Result<Value> {
    let name = "hex.decode";
    ensure_args_count(span, name, params, args, 1)?;

    let encoded_str = ensure_string(name, &params[0], &args[0])?;
    let decoded_bytes = data_encoding::HEXLOWER_PERMISSIVE.decode(encoded_str.as_bytes())?;
    Ok(Value::String(
        String::from_utf8_lossy(&decoded_bytes).into(),
    ))
}

#[cfg(feature = "hex")]
fn hex_encode(span: &Span, params: &[Ref<Expr>], args: &[Value], _strict: bool) -> Result<Value> {
    let name = "hex.encode";
    ensure_args_count(span, name, params, args, 1)?;

    let string = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::String(
        data_encoding::HEXLOWER_PERMISSIVE
            .encode(string.as_bytes())
            .into(),
    ))
}

#[cfg(feature = "urlquery")]
fn urlquery_decode_object(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "urlquery.encode";
    ensure_args_count(span, name, params, args, 1)?;

    let string = ensure_string(name, &params[0], &args[0])?;
    let url_string = "https://non-existent?".to_owned() + string.as_ref();
    let url = match url::Url::parse(&url_string) {
        Ok(v) => v,
        Err(_) => bail!(params[0].span().error("not a valid url query")),
    };

    let mut map = BTreeMap::new();
    for (k, v) in url.query_pairs() {
        let key = Value::String(k.clone().into());
        let value = Value::String(v.clone().into());
        if let Ok(a) = map.entry(key).or_insert(Value::new_array()).as_array_mut() {
            a.push(value)
        }
    }
    Ok(Value::from_map(map))
}
/*
#[cfg(feature = "urlquery")]
fn urlquery_encode(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "urlquery.encode";
    ensure_args_count(span, name, params, args, 1)?;

    let string = ensure_string(name, &params[0], &args[0])?;
    let url_string = "https://non-existent?" + string;
    let url = url::Url::parse(&url_string)
        .map_err(|_| bail!(params[0].span().error("not a valid url query")))?;

    Ok(Value::from_object(
        url.query_pairs()
            .map(|(k, v)| (Value::from(k.clone()), Value::from(v.clone())))
            .collect(),
    ))
}*/

#[cfg(feature = "yaml")]
fn yaml_is_valid(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "yaml.is_valid";
    ensure_args_count(span, name, params, args, 1)?;

    let yaml_str = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::Bool(Value::from_yaml_str(&yaml_str).is_ok()))
}

#[cfg(feature = "yaml")]
fn yaml_marshal(span: &Span, params: &[Ref<Expr>], args: &[Value], _strict: bool) -> Result<Value> {
    let name = "yaml.marshal";
    ensure_args_count(span, name, params, args, 1)?;
    Ok(Value::String(
        serde_yaml::to_string(&args[0])
            .with_context(|| span.error("could not serialize to yaml"))?
            .into(),
    ))
}

#[cfg(feature = "yaml")]
fn yaml_unmarshal(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "yaml.unmarshal";
    ensure_args_count(span, name, params, args, 1)?;
    let yaml_str = ensure_string(name, &params[0], &args[0])?;
    Value::from_yaml_str(&yaml_str).with_context(|| span.error("could not deserialize yaml."))
}

fn json_is_valid(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "json.is_valid";
    ensure_args_count(span, name, params, args, 1)?;

    let json_str = ensure_string(name, &params[0], &args[0])?;
    Ok(Value::Bool(Value::from_json_str(&json_str).is_ok()))
}

fn json_marshal(span: &Span, params: &[Ref<Expr>], args: &[Value], _strict: bool) -> Result<Value> {
    let name = "json.marshal";
    ensure_args_count(span, name, params, args, 1)?;
    Ok(Value::String(
        serde_json::to_string(&args[0])
            .with_context(|| span.error("could not serialize to json"))?
            .into(),
    ))
}

fn json_unmarshal(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    _strict: bool,
) -> Result<Value> {
    let name = "json.unmarshal";
    ensure_args_count(span, name, params, args, 1)?;
    let json_str = ensure_string(name, &params[0], &args[0])?;
    Value::from_json_str(&json_str).with_context(|| span.error("could not deserialize json."))
}

#[cfg(feature = "jsonschema")]
fn compile_json_schema(param: &Ref<Expr>, arg: &Value) -> Result<jsonschema::JSONSchema> {
    let schema_str = match arg {
        Value::String(schema_str) => schema_str.as_ref().to_string(),
        _ => arg.to_json_str()?,
    };

    if let Ok(schema) = serde_json::from_str(&schema_str) {
        match jsonschema::JSONSchema::compile(&schema) {
            Ok(schema) => return Ok(schema),
            Err(e) => bail!(e.to_string()),
        }
    }
    bail!(param.span().error("not a valid json schema"))
}

#[cfg(feature = "jsonschema")]
fn json_verify_schema(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    strict: bool,
) -> Result<Value> {
    let name = "json.verify_schema";
    ensure_args_count(span, name, params, args, 1)?;

    Ok(Value::from_array(
        match compile_json_schema(&params[0], &args[0]) {
            Ok(_) => [Value::Bool(true), Value::Null],
            Err(e) if strict => bail!(params[0]
                .span()
                .error(format!("invalid schema: {e}").as_str())),
            Err(e) => [Value::Bool(false), Value::String(e.to_string().into())],
        }
        .to_vec(),
    ))
}

#[cfg(feature = "jsonschema")]
fn json_match_schema(
    span: &Span,
    params: &[Ref<Expr>],
    args: &[Value],
    strict: bool,
) -> Result<Value> {
    let name = "json.match_schema";
    ensure_args_count(span, name, params, args, 2)?;

    // The following is expected to succeed.
    let document: serde_json::Value = serde_json::from_str(&args[0].to_json_str()?)?;

    Ok(Value::from_array(
        match compile_json_schema(&params[1], &args[1]) {
            Ok(schema) => match schema.validate(&document) {
                Ok(_) => [Value::Bool(true), Value::Null],
                Err(e) => [
                    Value::Bool(false),
                    Value::from_array(e.map(|e| Value::String(e.to_string().into())).collect()),
                ],
            },
            Err(e) if strict => bail!(params[1]
                .span()
                .error(format!("invalid schema: {e}").as_str())),
            Err(e) => [Value::Bool(false), Value::String(e.to_string().into())],
        }
        .to_vec(),
    ))
}
