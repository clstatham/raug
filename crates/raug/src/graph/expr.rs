use nom::{
    Parser,
    branch::alt,
    bytes::complete::{take_while_m_n, take_while1},
    character::complete::{char, multispace0},
    combinator::{all_consuming, map, map_res, opt, recognize},
    sequence::{delimited, pair, preceded, terminated},
};
use nom_language::error::{VerboseError, convert_error};
use raug_graph::node::{AsNodeInputIndex, AsNodeOutputIndex};

use crate::{
    graph::{Node, builder::GraphBuilder, node::RaugNodeIndexExt},
    prelude::ProcessorNode,
};

type IResult<'a, O> = nom::IResult<&'a str, O, VerboseError<&'a str>>;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum NodePortIndex<'a> {
    U32(u32),
    Str(&'a str),
}

impl<'a> std::fmt::Display for NodePortIndex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodePortIndex::U32(i) => write!(f, "{}", i),
            NodePortIndex::Str(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> AsNodeInputIndex<ProcessorNode> for NodePortIndex<'a> {
    fn as_node_input_index(
        &self,
        graph: &raug_graph::graph::Graph<ProcessorNode>,
        node: raug_graph::graph::NodeIndex,
    ) -> Option<u32> {
        match self {
            NodePortIndex::U32(i) => i.as_node_input_index(graph, node),
            NodePortIndex::Str(s) => s.as_node_input_index(graph, node),
        }
    }
}

impl<'a> AsNodeOutputIndex<ProcessorNode> for NodePortIndex<'a> {
    fn as_node_output_index(
        &self,
        graph: &raug_graph::graph::Graph<ProcessorNode>,
        node: raug_graph::graph::NodeIndex,
    ) -> Option<u32> {
        match self {
            NodePortIndex::U32(i) => i.as_node_output_index(graph, node),
            NodePortIndex::Str(s) => s.as_node_output_index(graph, node),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ast<'a> {
    NodePort(&'a str, NodePortIndex<'a>),
    NodeInputAssignment {
        name: &'a str,
        node_input: NodePortIndex<'a>,
        value: Box<Ast<'a>>,
    },
    Number(f32),
    InfixOp {
        op: char,
        left: Box<Ast<'a>>,
        right: Box<Ast<'a>>,
    },
    PrefixOp {
        op: char,
        expr: Box<Ast<'a>>,
    },
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Val<'a> {
    Void,
    NodePort(Node, NodePortIndex<'a>),
    Number(f32),
}

impl Val<'_> {
    pub fn as_number(&self) -> Option<f32> {
        if let Val::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }
}

impl<'a> Ast<'a> {
    pub fn parse(s: &'a str) -> Result<Self, String> {
        let (_, expr) = expr(s).map_err(|e| match e {
            nom::Err::Incomplete(_) => "incomplete input".to_string(),
            nom::Err::Error(ve) | nom::Err::Failure(ve) => convert_error(s, ve),
        })?;
        Ok(expr)
    }

    pub fn eval(&self, builder: &mut GraphBuilder) -> Result<Val<'_>, String> {
        match self {
            Ast::Number(n) => Ok(Val::Number(*n)),
            Ast::PrefixOp { op, expr } => {
                let val = expr.eval(builder)?;
                match (*op, val) {
                    ('+', Val::Number(n)) => Ok(Val::Number(n)),
                    ('-', Val::Number(n)) => Ok(Val::Number(-n)),
                    _ => Err(format!("invalid prefix op: {} {:?}", op, val)),
                }
            }
            Ast::InfixOp { op, left, right } => {
                let lval = left.eval(builder)?;
                let rval = right.eval(builder)?;
                match (op, lval, rval) {
                    // number-number ops
                    ('+', Val::Number(l), Val::Number(r)) => Ok(Val::Number(l + r)),
                    ('-', Val::Number(l), Val::Number(r)) => Ok(Val::Number(l - r)),
                    ('*', Val::Number(l), Val::Number(r)) => Ok(Val::Number(l * r)),
                    ('/', Val::Number(l), Val::Number(r)) => Ok(Val::Number(l / r)),
                    ('%', Val::Number(l), Val::Number(r)) => Ok(Val::Number(l % r)),
                    // number-port ops
                    ('+', Val::Number(l), Val::NodePort(node, port)) => {
                        let new_node = builder.graph.add(l, node.output(port));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('-', Val::Number(l), Val::NodePort(node, port)) => {
                        let new_node = builder.graph.sub(l, node.output(port));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('*', Val::Number(l), Val::NodePort(node, port)) => {
                        let new_node = builder.graph.mul(l, node.output(port));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('/', Val::Number(l), Val::NodePort(node, port)) => {
                        let new_node = builder.graph.div(l, node.output(port));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('%', Val::Number(l), Val::NodePort(node, port)) => {
                        let new_node = builder.graph.rem(l, node.output(port));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    // port-number ops
                    ('+', Val::NodePort(node, port), Val::Number(r)) => {
                        let new_node = builder.graph.add(node.output(port), r);
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('-', Val::NodePort(node, port), Val::Number(r)) => {
                        let new_node = builder.graph.sub(node.output(port), r);
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('*', Val::NodePort(node, port), Val::Number(r)) => {
                        let new_node = builder.graph.mul(node.output(port), r);
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('/', Val::NodePort(node, port), Val::Number(r)) => {
                        let new_node = builder.graph.div(node.output(port), r);
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('%', Val::NodePort(node, port), Val::Number(r)) => {
                        let new_node = builder.graph.rem(node.output(port), r);
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    // port-port ops
                    ('+', Val::NodePort(lnode, lport), Val::NodePort(rnode, rport)) => {
                        let new_node = builder.graph.add(lnode.output(lport), rnode.output(rport));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('-', Val::NodePort(lnode, lport), Val::NodePort(rnode, rport)) => {
                        let new_node = builder.graph.sub(lnode.output(lport), rnode.output(rport));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('*', Val::NodePort(lnode, lport), Val::NodePort(rnode, rport)) => {
                        let new_node = builder.graph.mul(lnode.output(lport), rnode.output(rport));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('/', Val::NodePort(lnode, lport), Val::NodePort(rnode, rport)) => {
                        let new_node = builder.graph.div(lnode.output(lport), rnode.output(rport));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    ('%', Val::NodePort(lnode, lport), Val::NodePort(rnode, rport)) => {
                        let new_node = builder.graph.rem(lnode.output(lport), rnode.output(rport));
                        Ok(Val::NodePort(new_node, NodePortIndex::U32(0)))
                    }
                    _ => Err(format!("invalid infix op: {} {:?} {:?}", op, lval, rval)),
                }
            }
            Ast::NodePort(name, port) => {
                let node = builder.get(name);
                Ok(Val::NodePort(node, *port))
            }
            Ast::NodeInputAssignment {
                name,
                node_input,
                value,
            } => {
                let node = builder.get(name);
                let val = value.eval(builder)?;
                let input = node.input(*node_input);
                match val {
                    Val::Number(n) => {
                        builder.graph.connect_constant(n, input);
                        Ok(Val::Void)
                    }
                    Val::NodePort(n, port) => {
                        builder.graph.connect(n.output(port), input);
                        Ok(Val::Void)
                    }
                    Val::Void => Err("cannot assign void to node input".to_string()),
                }
            }
        }
    }
}

pub fn expr(input: &'_ str) -> IResult<'_, Ast<'_>> {
    all_consuming(delimited(multispace0, statement, multispace0)).parse(input)
}

pub fn statement(input: &'_ str) -> IResult<'_, Ast<'_>> {
    alt((expr_assignment, expr_infix)).parse(input)
}

fn expr_assignment(input: &'_ str) -> IResult<'_, Ast<'_>> {
    let (input, name) = terminated(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        multispace0,
    )
    .parse(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, node_input) = alt((
        map_res(
            recognize(take_while_m_n(1, 10, |c: char| c.is_ascii_digit())),
            |s: &str| s.parse::<u32>().map(NodePortIndex::U32),
        ),
        map(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |s: &str| NodePortIndex::Str(s),
        ),
    ))
    .parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, value) = expr_infix(input)?;
    Ok((
        input,
        Ast::NodeInputAssignment {
            name,
            node_input,
            value: Box::new(value),
        },
    ))
}

fn expr_infix(input: &'_ str) -> IResult<'_, Ast<'_>> {
    expr_pratt(input, 0)
}

fn expr_pratt(input: &'_ str, min_precedence: u8) -> IResult<'_, Ast<'_>> {
    let (mut input, mut left) = expr_prefix(input)?;

    loop {
        let (rest, _) = multispace0(input)?;

        if let Ok((rest, op)) = infix_op(rest) {
            let (precedence, associativity) = op_precedence(op);
            if precedence < min_precedence {
                break;
            }

            let next_min_prec = match associativity {
                Associativity::Left => precedence + 1,
                Associativity::Right => precedence,
            };

            let (rest, _) = multispace0(rest)?;
            let (rest, right) = expr_pratt(rest, next_min_prec)?;

            left = Ast::InfixOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
            input = rest;
        } else {
            break;
        }
    }

    Ok((input, left))
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Associativity {
    Left,
    Right,
}

fn infix_op(input: &'_ str) -> IResult<'_, char> {
    alt((char('+'), char('-'), char('*'), char('/'), char('%'))).parse(input)
}

fn op_precedence(op: char) -> (u8, Associativity) {
    match op {
        '+' | '-' => (1, Associativity::Left),
        '*' | '/' | '%' => (2, Associativity::Left),
        _ => (0, Associativity::Left),
    }
}

fn expr_prefix(input: &'_ str) -> IResult<'_, Ast<'_>> {
    alt((
        map(
            pair(
                alt((char('+'), char('-'))),
                preceded(multispace0, expr_prefix),
            ),
            |(op, expr)| Ast::PrefixOp {
                op,
                expr: Box::new(expr),
            },
        ),
        expr_primary,
    ))
    .parse(input)
}

fn parse_sci_float(input: &'_ str) -> IResult<'_, f32> {
    map_res(
        recognize(pair(
            alt((
                recognize(pair(
                    take_while1(|c: char| c.is_ascii_digit()),
                    opt(pair(char('.'), take_while1(|c: char| c.is_ascii_digit()))),
                )),
                recognize(pair(char('.'), take_while1(|c: char| c.is_ascii_digit()))),
            )),
            opt(pair(
                alt((char('e'), char('E'))),
                pair(
                    opt(alt((char('+'), char('-')))),
                    take_while1(|c: char| c.is_ascii_digit()),
                ),
            )),
        )),
        |s: &str| s.parse::<f32>(),
    )
    .parse(input)
}

fn parse_integer(input: &'_ str) -> IResult<'_, f32> {
    map_res(
        recognize(take_while1(|c: char| c.is_ascii_digit())),
        |s: &str| s.parse::<f32>(),
    )
    .parse(input)
}

fn parse_float(input: &'_ str) -> IResult<'_, f32> {
    alt((parse_sci_float, parse_integer)).parse(input)
}

fn expr_primary(input: &'_ str) -> IResult<'_, Ast<'_>> {
    alt((
        map(parse_float, Ast::Number),
        map(
            pair(
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                opt(preceded(
                    char('.'),
                    alt((
                        map(
                            recognize(take_while1(|c: char| c.is_ascii_digit())),
                            |s: &str| NodePortIndex::U32(s.parse::<u32>().unwrap()),
                        ),
                        map(
                            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                            |s: &str| NodePortIndex::Str(s),
                        ),
                    )),
                )),
            ),
            |(name, port)| {
                let port = port.unwrap_or(NodePortIndex::U32(0));
                Ast::NodePort(name, port)
            },
        ),
        delimited(
            delimited(multispace0, char('('), multispace0),
            expr_infix,
            delimited(multispace0, char(')'), multispace0),
        ),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::approx_constant)]
    #[test]
    fn test_parse_number() {
        assert_eq!(Ast::parse("42").unwrap(), Ast::Number(42.0));
        assert_eq!(Ast::parse("3.14").unwrap(), Ast::Number(3.14));
        assert_eq!(
            Ast::parse("-5").unwrap(),
            Ast::PrefixOp {
                op: '-',
                expr: Box::new(Ast::Number(5.0))
            }
        );
        assert_eq!(Ast::parse("1e2").unwrap(), Ast::Number(100.0));
        assert_eq!(Ast::parse("2.5e-1").unwrap(), Ast::Number(0.25));
    }

    #[test]
    fn test_parse_node() {
        assert_eq!(
            Ast::parse("oscillator").unwrap(),
            Ast::NodePort("oscillator", NodePortIndex::U32(0))
        );
        assert_eq!(
            Ast::parse("filter_1").unwrap(),
            Ast::NodePort("filter_1", NodePortIndex::U32(0))
        );
        assert_eq!(
            Ast::parse("_private").unwrap(),
            Ast::NodePort("_private", NodePortIndex::U32(0))
        );
    }

    #[test]
    fn test_parse_node_output() {
        assert_eq!(
            Ast::parse("osc.0").unwrap(),
            Ast::NodePort("osc", NodePortIndex::U32(0))
        );
        assert_eq!(
            Ast::parse("filter.2").unwrap(),
            Ast::NodePort("filter", NodePortIndex::U32(2))
        );
        assert_eq!(
            Ast::parse("synthesizer.10").unwrap(),
            Ast::NodePort("synthesizer", NodePortIndex::U32(10))
        );
    }

    #[test]
    fn test_parse_node_output_named() {
        assert_eq!(
            Ast::parse("osc.out").unwrap(),
            Ast::NodePort("osc", NodePortIndex::Str("out"))
        );
        assert_eq!(
            Ast::parse("filter.cutoff").unwrap(),
            Ast::NodePort("filter", NodePortIndex::Str("cutoff"))
        );
        assert_eq!(
            Ast::parse("synthesizer.main").unwrap(),
            Ast::NodePort("synthesizer", NodePortIndex::Str("main"))
        );
    }

    #[test]
    fn test_parse_prefix_ops() {
        assert_eq!(
            Ast::parse("+42").unwrap(),
            Ast::PrefixOp {
                op: '+',
                expr: Box::new(Ast::Number(42.0))
            }
        );
        assert_eq!(
            Ast::parse("-osc").unwrap(),
            Ast::PrefixOp {
                op: '-',
                expr: Box::new(Ast::NodePort("osc", NodePortIndex::U32(0)))
            }
        );
    }

    #[test]
    fn test_parse_infix_ops() {
        assert_eq!(
            Ast::parse("2 + 3").unwrap(),
            Ast::InfixOp {
                op: '+',
                left: Box::new(Ast::Number(2.0)),
                right: Box::new(Ast::Number(3.0))
            }
        );
        assert_eq!(
            Ast::parse("a * b").unwrap(),
            Ast::InfixOp {
                op: '*',
                left: Box::new(Ast::NodePort("a", NodePortIndex::U32(0))),
                right: Box::new(Ast::NodePort("b", NodePortIndex::U32(0)))
            }
        );
    }

    #[test]
    fn test_parse_operator_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        assert_eq!(
            Ast::parse("2 + 3 * 4").unwrap(),
            Ast::InfixOp {
                op: '+',
                left: Box::new(Ast::Number(2.0)),
                right: Box::new(Ast::InfixOp {
                    op: '*',
                    left: Box::new(Ast::Number(3.0)),
                    right: Box::new(Ast::Number(4.0))
                })
            }
        );
    }

    #[test]
    fn test_parse_parentheses() {
        // (2 + 3) * 4 should parse as (2 + 3) * 4
        assert_eq!(
            Ast::parse("(2 + 3) * 4").unwrap(),
            Ast::InfixOp {
                op: '*',
                left: Box::new(Ast::InfixOp {
                    op: '+',
                    left: Box::new(Ast::Number(2.0)),
                    right: Box::new(Ast::Number(3.0))
                }),
                right: Box::new(Ast::Number(4.0))
            }
        );
    }

    #[test]
    fn test_parse_complex_expression() {
        let result = Ast::parse("osc.0 + filter.1 * 2.5").unwrap();
        assert_eq!(
            result,
            Ast::InfixOp {
                op: '+',
                left: Box::new(Ast::NodePort("osc", NodePortIndex::U32(0))),
                right: Box::new(Ast::InfixOp {
                    op: '*',
                    left: Box::new(Ast::NodePort("filter", NodePortIndex::U32(1))),
                    right: Box::new(Ast::Number(2.5))
                })
            }
        );
    }

    #[test]
    fn test_parse_assignment() {
        let result = Ast::parse("filter[cutoff] = osc.0 * 0.5 + 200.0").unwrap();
        assert_eq!(
            result,
            Ast::NodeInputAssignment {
                name: "filter",
                node_input: NodePortIndex::Str("cutoff"),
                value: Box::new(Ast::InfixOp {
                    op: '+',
                    left: Box::new(Ast::InfixOp {
                        op: '*',
                        left: Box::new(Ast::NodePort("osc", NodePortIndex::U32(0))),
                        right: Box::new(Ast::Number(0.5))
                    }),
                    right: Box::new(Ast::Number(200.0))
                })
            }
        );
    }

    #[test]
    fn test_parse_whitespace() {
        assert_eq!(Ast::parse("  42  ").unwrap(), Ast::Number(42.0));
        assert_eq!(
            Ast::parse(" 2 + 3 ").unwrap(),
            Ast::InfixOp {
                op: '+',
                left: Box::new(Ast::Number(2.0)),
                right: Box::new(Ast::Number(3.0))
            }
        );
    }

    #[test]
    fn test_val_as_number() {
        assert_eq!(Val::Number(42.0).as_number(), Some(42.0));
        assert_eq!(Val::Void.as_number(), None);
    }

    #[test]
    fn test_parse_invalid_syntax() {
        assert!(Ast::parse("2 +").is_err());
        assert!(Ast::parse("* 3").is_err());
        assert!(Ast::parse("node.").is_err());
        assert!(Ast::parse("(2 + 3").is_err());
    }
}
