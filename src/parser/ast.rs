use crate::{
    ast,
    parser::{Constant, ZeroOrMore, comma_t},
};

use super::{Parser, Result};

fn expression(source: &str) -> Option<Result<'_, ast::Node>> {
    todo!()
}

fn arguments(source: &str) -> Option<Result<'_, Vec<ast::Node>>> {
    let parser = expression
        .bind(|arg| {
            ZeroOrMore::new(comma_t.and(expression)).bind(move |mut args| {
                args.insert(0, arg.clone());
                Constant::new(args)
            })
        })
        .or(Constant::new(vec![]));

    parser.parse(source)
}
