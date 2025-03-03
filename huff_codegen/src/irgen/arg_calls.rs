use huff_utils::prelude::*;
use std::str::FromStr;

// Arguments can be literals, labels, opcodes, or constants
// !! IF THERE IS AMBIGUOUS NOMENCLATURE
// !! (E.G. BOTH OPCODE AND LABEL ARE THE SAME STRING)
// !! COMPILATION _WILL_ ERROR

/// Arg Call Bubbling
#[allow(clippy::too_many_arguments)]
pub fn bubble_arg_call(
    arg_name: &str,
    bytes: &mut Vec<(usize, Bytes)>,
    macro_def: &MacroDefinition,
    contract: &Contract,
    scope: &mut Vec<MacroDefinition>,
    offset: &mut usize,
    // mis: Parent macro invocations and their indices
    mis: &mut Vec<(usize, MacroInvocation)>,
    jump_table: &mut JumpTable,
) -> Result<(), CodegenError> {
    let starting_offset = *offset;

    // Check Constant Definitions
    if let Some(constant) = contract.constants.iter().find(|const_def| const_def.name.eq(arg_name))
    {
        tracing::info!(target: "codegen", "ARGCALL IS CONSTANT: {:?}", constant);
        let push_bytes = match &constant.value {
            ConstVal::Literal(l) => {
                let hex_literal: String = bytes32_to_string(l, false);
                format!("{:02x}{}", 95 + hex_literal.len() / 2, hex_literal)
            }
            ConstVal::FreeStoragePointer(fsp) => {
                // If this is reached in codegen stage,
                // `derive_storage_pointers`
                // method was not called on the AST.
                tracing::error!(target: "codegen", "STORAGE POINTERS INCORRECTLY DERIVED FOR \"{:?}\"", fsp);
                return Err(CodegenError {
                    kind: CodegenErrorKind::StoragePointersNotDerived,
                    span: AstSpan(vec![]),
                    token: None,
                })
            }
        };
        *offset += push_bytes.len() / 2;
        tracing::info!(target: "codegen", "OFFSET: {}, PUSH BYTES: {:?}", offset, push_bytes);
        bytes.push((starting_offset, Bytes(push_bytes)));
    } else if let Ok(o) = Opcode::from_str(arg_name) {
        // Check Opcode Definition
        let b = Bytes(o.to_string());
        *offset += b.0.len() / 2;
        tracing::info!(target: "codegen", "RECURSE_BYTECODE ARG CALL FOUND OPCODE: {:?}", b);
        bytes.push((starting_offset, b));
    } else if let Some(macro_invoc) = mis.last() {
        // Literal & Arg Call Check
        // First get this arg_nam position in the macro definition params
        if let Some(pos) = macro_def
            .parameters
            .iter()
            .position(|r| r.name.as_ref().map_or(false, |s| s.eq(arg_name)))
        {
            tracing::info!(target: "codegen", "GOT \"{}\" POS IN ARG LIST: {}", arg_name, pos);

            if let Some(arg) = macro_invoc.1.args.get(pos) {
                tracing::info!(target: "codegen", "GOT \"{:?}\" ARG FROM MACRO INVOCATION", arg);
                match arg {
                    MacroArg::Literal(l) => {
                        tracing::info!(target: "codegen", "GOT LITERAL {} ARG FROM MACRO INVOCATION", bytes32_to_string(l, false));

                        let hex_literal: String = bytes32_to_string(l, false);
                        let push_bytes =
                            format!("{:02x}{}", 95 + hex_literal.len() / 2, hex_literal);
                        let b = Bytes(push_bytes);
                        *offset += b.0.len() / 2;
                        bytes.push((starting_offset, b));
                    }
                    MacroArg::ArgCall(ac) => {
                        tracing::info!(target: "codegen", "GOT ARG CALL \"{}\" ARG FROM MACRO INVOCATION", ac);
                        tracing::debug!(target: "codegen", "~~~ BUBBLING UP ARG CALL");
                        let mut new_scope = Vec::from(&scope[..scope.len().saturating_sub(1)]);
                        let bubbled_macro_invocation = new_scope.last().unwrap().clone();
                        tracing::debug!(target: "codegen", "BUBBLING UP WITH MACRO DEF: {}", bubbled_macro_invocation.name);
                        tracing::debug!(target: "codegen", "CURRENT MACRO DEF: {}", macro_def.name);

                        // Only remove an invocation if not at bottom level, otherwise we'll
                        // remove one too many
                        let last_mi = match mis.last() {
                            Some(mi) => mi,
                            None => {
                                return Err(CodegenError {
                                    kind: CodegenErrorKind::MissingMacroInvocation(
                                        macro_def.name.clone(),
                                    ),
                                    span: bubbled_macro_invocation.span,
                                    token: None,
                                })
                            }
                        };
                        return if last_mi.1.macro_name.eq(&macro_def.name) {
                            bubble_arg_call(
                                arg_name,
                                bytes,
                                &bubbled_macro_invocation,
                                contract,
                                &mut new_scope,
                                offset,
                                &mut Vec::from(&mis[..mis.len().saturating_sub(1)]),
                                jump_table,
                            )
                        } else {
                            bubble_arg_call(
                                arg_name,
                                bytes,
                                &bubbled_macro_invocation,
                                contract,
                                &mut new_scope,
                                offset,
                                mis,
                                jump_table,
                            )
                        }
                    }
                    MacroArg::Ident(iden) => {
                        tracing::debug!(target: "codegen", "FOUND IDENT ARG IN \"{}\" MACRO INVOCATION: \"{}\"!", macro_invoc.1.macro_name, iden);
                        tracing::debug!(target: "codegen", "Macro invocation index: {}", macro_invoc.0);
                        tracing::debug!(target: "codegen", "At offset: {}", *offset);

                        // This should be equivalent to a label call.
                        bytes.push((*offset, Bytes(format!("{}xxxx", Opcode::Push2))));
                        jump_table.insert(
                            *offset,
                            vec![Jump {
                                label: iden.to_owned(),
                                bytecode_index: 0,
                                span: macro_invoc.1.span.clone(),
                            }],
                        );
                        *offset += 3;
                    }
                }
            } else {
                tracing::warn!(target: "codegen", "\"{}\" FOUND IN MACRO DEF BUT NOT IN MACRO INVOCATION!", arg_name);
            }
        } else {
            tracing::warn!(target: "codegen", "\"{}\" NOT IN ARG LIST", arg_name);
        }
    } else {
        // This is a label call
        tracing::info!(target: "codegen", "RECURSE_BYTECODE ARG CALL DEFAULTING TO LABEL CALL: \"{}\"", arg_name);
        let new_span = match mis.last() {
            Some(mi) => mi.1.span.clone(),
            None => AstSpan(vec![]),
        };
        jump_table.insert(
            mis.last().map(|mi| mi.0).unwrap_or_else(|| 0),
            vec![Jump { label: arg_name.to_owned(), bytecode_index: 0, span: new_span }],
        );
        bytes.push((*offset, Bytes(format!("{}xxxx", Opcode::Push2))));
        *offset += 3;
    }

    Ok(())
}
