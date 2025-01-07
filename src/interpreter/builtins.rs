use std::time::{SystemTime, UNIX_EPOCH};

use super::MutInterpreter;
use crate::interpreter::Result;
use crate::{Token, TokenType, Value};

pub fn clock(_interpreter: &MutInterpreter, _args: &[Value]) -> Result<Value> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // Возвращает время в секундах
    Ok(Value::Number(since_the_epoch.as_secs_f64()))
}

pub fn sum(_interpreter: &MutInterpreter, args: &[Value]) -> Result<Value> {
    let a = &args[0];
    let b = &args[1];

    let res = a.calculate(Some(&b), Token::new(TokenType::PLUS, "+", None, 1));

    Ok(res?)
}
