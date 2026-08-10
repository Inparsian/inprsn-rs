#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShellOp {
    AndIf,
    OrIf,
    Seq,
    Pipe,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
    RedirectErrOut,
    RedirectErrAppend,
}

#[derive(Clone, Debug)]
pub(super) enum ShellToken {
    Word(String),
    Op(ShellOp),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ChainCondition {
    Always,
    AndIf,
    OrIf,
}

#[derive(Clone, Debug)]
pub(super) struct SimpleCommand {
    pub(super) argv: Vec<String>,
    pub(super) stdin: Option<String>,
    pub(super) stdout: Option<(String, bool)>, // (path, append)
    pub(super) stderr: Option<(String, bool)>, // (path, append)
}

#[derive(Clone, Debug)]
pub(super) struct Pipeline {
    pub(super) commands: Vec<SimpleCommand>,
}

pub(super) fn completion_segment(before_cursor: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut last_start = 0_usize;
    let chars: Vec<char> = before_cursor.chars().collect();
    let mut i = 0_usize;

    while i < chars.len() {
        let ch = chars[i];

        if escape {
            escape = false;
            i += 1;
            continue;
        }

        if in_single {
            if ch == '\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }

        if in_double {
            match ch {
                '\\' => escape = true,
                '"' => in_double = false,
                _ => {}
            }
            i += 1;
            continue;
        }

        match ch {
            '\\' => escape = true,
            '\'' => in_single = true,
            '"' => in_double = true,
            '|' if i + 1 < chars.len() && chars[i + 1] == '|' => {
                last_start = i + 2;
                i += 1;
            }
            '|' | ';' => last_start = i + 1,
            '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                last_start = i + 2;
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    before_cursor[last_start..].to_owned()
}

pub(super) fn tokenize_shell(input: &str) -> Result<Vec<ShellToken>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0_usize;

    while i < chars.len() {
        let ch = chars[i];

        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        if i + 2 < chars.len() && chars[i] == '2' && chars[i + 1] == '>' && chars[i + 2] == '>' {
            tokens.push(ShellToken::Op(ShellOp::RedirectErrAppend));
            i += 3;
            continue;
        }

        if i + 1 < chars.len() {
            match (chars[i], chars[i + 1]) {
                ('&', '&') => {
                    tokens.push(ShellToken::Op(ShellOp::AndIf));
                    i += 2;
                    continue;
                }
                ('|', '|') => {
                    tokens.push(ShellToken::Op(ShellOp::OrIf));
                    i += 2;
                    continue;
                }
                ('>', '>') => {
                    tokens.push(ShellToken::Op(ShellOp::RedirectAppend));
                    i += 2;
                    continue;
                }
                ('2', '>') => {
                    tokens.push(ShellToken::Op(ShellOp::RedirectErrOut));
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        match ch {
            ';' => {
                tokens.push(ShellToken::Op(ShellOp::Seq));
                i += 1;
            }
            '|' => {
                tokens.push(ShellToken::Op(ShellOp::Pipe));
                i += 1;
            }
            '<' => {
                tokens.push(ShellToken::Op(ShellOp::RedirectIn));
                i += 1;
            }
            '>' => {
                tokens.push(ShellToken::Op(ShellOp::RedirectOut));
                i += 1;
            }
            '&' => return Err("syntax error near unexpected token '&'".to_owned()),
            _ => {
                let mut word = String::new();
                let mut in_single = false;
                let mut in_double = false;
                let mut escape = false;

                while i < chars.len() {
                    let c = chars[i];

                    if escape {
                        word.push(c);
                        escape = false;
                        i += 1;
                        continue;
                    }

                    if in_single {
                        if c == '\'' {
                            in_single = false;
                        } else {
                            word.push(c);
                        }
                        i += 1;
                        continue;
                    }

                    if in_double {
                        match c {
                            '\\' => escape = true,
                            '"' => in_double = false,
                            _ => word.push(c),
                        }
                        i += 1;
                        continue;
                    }

                    match c {
                        '\\' => {
                            escape = true;
                            i += 1;
                        }
                        '\'' => {
                            in_single = true;
                            i += 1;
                        }
                        '"' => {
                            in_double = true;
                            i += 1;
                        }
                        delim
                            if delim.is_whitespace()
                                || matches!(delim, ';' | '|' | '<' | '>' | '&') =>
                        {
                            break;
                        }
                        _ => {
                            word.push(c);
                            i += 1;
                        }
                    }
                }

                if escape || in_single || in_double {
                    return Err("Invalid quotes or escape sequence".to_owned());
                }

                if !word.is_empty() {
                    tokens.push(ShellToken::Word(word));
                }
            }
        }
    }

    Ok(tokens)
}

pub(super) fn parse_command_chains(
    tokens: &[ShellToken],
) -> Result<Vec<(Option<ChainCondition>, Pipeline)>, String> {
    let mut i = 0_usize;
    let mut out = Vec::new();
    let mut pending_cond: Option<ChainCondition> = None;

    while i < tokens.len() {
        let (pipeline, next_i) = parse_pipeline(tokens, i)?;
        out.push((pending_cond, pipeline));
        pending_cond = None;
        i = next_i;

        if i >= tokens.len() {
            break;
        }

        match tokens.get(i) {
            Some(ShellToken::Op(ShellOp::Seq)) => {
                pending_cond = Some(ChainCondition::Always);
                i += 1;
            }
            Some(ShellToken::Op(ShellOp::AndIf)) => {
                pending_cond = Some(ChainCondition::AndIf);
                i += 1;
            }
            Some(ShellToken::Op(ShellOp::OrIf)) => {
                pending_cond = Some(ChainCondition::OrIf);
                i += 1;
            }
            Some(ShellToken::Op(ShellOp::Pipe)) => {
                return Err("syntax error near unexpected token '|'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectIn)) => {
                return Err("syntax error near unexpected token '<'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectOut)) => {
                return Err("syntax error near unexpected token '>'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectAppend)) => {
                return Err("syntax error near unexpected token '>>'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectErrOut)) => {
                return Err("syntax error near unexpected token '2>'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectErrAppend)) => {
                return Err("syntax error near unexpected token '2>>'".to_owned());
            }
            Some(ShellToken::Word(w)) => {
                return Err(format!("syntax error near unexpected token '{}'", w));
            }
            None => break,
        }
    }

    if pending_cond.is_some() {
        return Err("syntax error near unexpected token 'newline'".to_owned());
    }

    Ok(out)
}

fn parse_pipeline(tokens: &[ShellToken], mut i: usize) -> Result<(Pipeline, usize), String> {
    let mut commands = Vec::new();

    loop {
        let (cmd, next_i) = parse_simple_command(tokens, i)?;
        commands.push(cmd);
        i = next_i;

        match tokens.get(i) {
            Some(ShellToken::Op(ShellOp::Pipe)) => {
                i += 1;
            }
            _ => break,
        }
    }

    if commands.is_empty() {
        return Err("syntax error near unexpected token '|'".to_owned());
    }

    Ok((Pipeline { commands }, i))
}

fn parse_simple_command(tokens: &[ShellToken], mut i: usize) -> Result<(SimpleCommand, usize), String> {
    let mut argv = Vec::new();
    let mut stdin = None;
    let mut stdout = None;
    let mut stderr = None;

    while let Some(token) = tokens.get(i) {
        match token {
            ShellToken::Word(w) => {
                argv.push(w.clone());
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectIn) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '<'".to_owned());
                };
                stdin = Some(path.clone());
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectOut) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '>'".to_owned());
                };
                stdout = Some((path.clone(), false));
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectAppend) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '>>'".to_owned());
                };
                stdout = Some((path.clone(), true));
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectErrOut) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '2>'".to_owned());
                };
                stderr = Some((path.clone(), false));
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectErrAppend) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '2>>'".to_owned());
                };
                stderr = Some((path.clone(), true));
                i += 1;
            }
            ShellToken::Op(ShellOp::Pipe | ShellOp::AndIf | ShellOp::OrIf | ShellOp::Seq) => {
                break;
            }
        }
    }

    if argv.is_empty() {
        return Err("syntax error near unexpected token".to_owned());
    }

    Ok((
        SimpleCommand {
            argv,
            stdin,
            stdout,
            stderr,
        },
        i,
    ))
}
