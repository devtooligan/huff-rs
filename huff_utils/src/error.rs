use crate::{
    files::{Span, Spanned},
    io::UnpackError,
    prelude::{parse_extension, AstSpan},
    report::{Report, Reporter},
    token::TokenKind,
};
use std::{ffi::OsString, fmt, io::Write};

/// A Parser Error
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ParserError {
    /// The type of Parser Error
    pub kind: ParserErrorKind,
    /// A collection of spans the Parser Error crosses
    pub spans: AstSpan,
}

/// A Type of Parser Error
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ParserErrorKind {
    /// A general syntax error that accepts a message
    SyntaxError(String),
    /// Unexpected type
    UnexpectedType(TokenKind),
    /// Invalid definition
    InvalidDefinition,
    /// Invalid constant value
    InvalidConstantValue(TokenKind),
    /// Unexpected token in macro body
    InvalidTokenInMacroBody(TokenKind),
    /// Unexpected token in label definition
    InvalidTokenInLabelDefinition(TokenKind),
    /// Unexpected Single Arg
    InvalidSingleArg(TokenKind),
    /// Unexpected Table Body Token
    InvalidTableBodyToken(TokenKind),
    /// Invalid constant
    InvalidConstant(TokenKind),
    /// Unexpected Arg Call Token
    InvalidArgCallIdent(TokenKind),
    /// Invalid name (macro, event, function, constant)
    InvalidName(TokenKind),
    /// Invalid arguments
    InvalidArgs(TokenKind),
    /// Invalid Uint256 Size
    InvalidUint256(usize),
    /// Invalid Bytes
    InvalidBytes(usize),
    /// Invalid Int
    InvalidInt(usize),
    /// Invalid macro call arguments
    InvalidMacroArgs(TokenKind),
    /// Invalid return arguments
    InvalidReturnArgs,
    /// Invalid import path
    InvalidImportPath(String),
}

/// A Lexing Error
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LexicalError<'a> {
    /// The kind of error
    pub kind: LexicalErrorKind<'a>,
    /// The span where the error occurred
    pub span: Span,
}

impl<'a> LexicalError<'a> {
    /// Public associated function to instatiate a new LexicalError.
    pub fn new(kind: LexicalErrorKind<'a>, span: Span) -> Self {
        Self { kind, span }
    }
}

/// A Lexical Error Kind
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexicalErrorKind<'a> {
    /// Unexpected end of file
    UnexpectedEof,
    /// Invalid character
    InvalidCharacter(char),
    /// Invalid Array Size
    /// String param expected to be usize parsable
    InvalidArraySize(&'a str),
    /// Invalid Primitive EVM Type
    InvalidPrimitiveType(&'a str),
}

impl<'a> Spanned for LexicalError<'a> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'a, W: Write> Report<W> for LexicalError<'a> {
    fn report(&self, f: &mut Reporter<'_, W>) -> std::io::Result<()> {
        match self.kind {
            LexicalErrorKind::InvalidCharacter(ch) => write!(f.out, "Invalid character '{}'", ch),
            LexicalErrorKind::UnexpectedEof => write!(f.out, "Found unexpected EOF"),
            LexicalErrorKind::InvalidArraySize(str) => {
                write!(f.out, "Invalid array size: '{}'", str)
            }
            LexicalErrorKind::InvalidPrimitiveType(str) => {
                write!(f.out, "Invalid Primitive EVM Type '{}'", str)
            }
        }
    }
}

/// A Code Generation Error
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CodegenError {
    /// The kind of code generation error
    pub kind: CodegenErrorKind,
    /// An Optional Span where the error occured
    pub span: AstSpan,
    /// An Optional Token Kind
    pub token: Option<TokenKind>,
}

impl CodegenError {
    /// Public associated function to instatiate a new CodegenError.
    pub fn new(kind: CodegenErrorKind, spans: AstSpan, token: Option<TokenKind>) -> Self {
        Self { kind, span: spans, token }
    }
}

/// The Code Generation Error Kind
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CodegenErrorKind {
    /// Storage Pointers Not Derived
    StoragePointersNotDerived,
    /// Invalid Macro Body Statement
    InvalidMacroStatement,
    /// The Macro Definition is Missing
    MissingMacroDefinition(String),
    /// Missing Constant Definition
    MissingConstantDefinition(String),
    /// Abi Generation Failure
    AbiGenerationFailure,
    /// Unmatched Jump
    UnmatchedJumpLabel,
    /// An IO Error
    IOError(String),
    /// ArgCall has an unknown type
    UnkownArgcallType,
    /// Missing Macro Invocation
    MissingMacroInvocation(String),
    /// Missing Macro Definition for Invocation
    InvalidMacroInvocation(String),
    /// Conversion Error for usize
    UsizeConversion(String),
}

impl Spanned for CodegenError {
    fn span(&self) -> Span {
        self.span.0[0].clone()
    }
}

impl<W: Write> Report<W> for CodegenError {
    fn report(&self, f: &mut Reporter<'_, W>) -> std::io::Result<()> {
        match &self.kind {
            CodegenErrorKind::StoragePointersNotDerived => {
                write!(f.out, "Storage pointers not derived for AST!")
            }
            CodegenErrorKind::InvalidMacroStatement => write!(f.out, "Invalid Macro Statement!"),
            CodegenErrorKind::InvalidMacroInvocation(str) => {
                write!(f.out, "Missing Macro Definition for Invocation: \"{}\"!", str)
            }
            CodegenErrorKind::MissingMacroDefinition(str) => {
                write!(f.out, "Missing Macro \"{}\" Definition!", str)
            }
            CodegenErrorKind::MissingConstantDefinition(cd) => {
                write!(f.out, "Missing Constant Definition for \"{}\"!", cd)
            }
            CodegenErrorKind::AbiGenerationFailure => write!(f.out, "Abi generation failure!"),
            CodegenErrorKind::UnmatchedJumpLabel => write!(f.out, "Unmatched jump label!"),
            CodegenErrorKind::IOError(ioe) => write!(f.out, "IO ERROR: {:?}", ioe),
            CodegenErrorKind::UnkownArgcallType => write!(f.out, "Unknown Argcall Type!"),
            CodegenErrorKind::MissingMacroInvocation(str) => {
                write!(f.out, "Missing Macro \"{}\" Invocation!", str)
            }
            CodegenErrorKind::UsizeConversion(input) => {
                write!(f.out, "Usize Conversion Failed for \"{}\"", input)
            }
        }
    }
}

/// CompilerError
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerError<'a> {
    /// Failed to Lex Source
    LexicalError(LexicalError<'a>),
    /// File unpacking error
    FileUnpackError(UnpackError),
    /// Parsing Error
    ParserError(ParserError),
    /// Reading PathBuf Failed
    PathBufRead(OsString),
    /// Bytecode Generation Error
    CodegenError(CodegenError),
    /// Multiple Failed Compiles
    FailedCompiles(Vec<CompilerError<'a>>),
}

impl<'a> fmt::Display for CompilerError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::LexicalError(le) => match le.kind {
                LexicalErrorKind::UnexpectedEof => {
                    write!(
                        f,
                        "\nError: Unexpected End Of File {}{}\n",
                        le.span.identifier(),
                        le.span.source_seg()
                    )
                }
                LexicalErrorKind::InvalidCharacter(c) => {
                    write!(
                        f,
                        "\nError: Invalid Character: \"{}\" {}{}\n",
                        c,
                        le.span.identifier(),
                        le.span.source_seg()
                    )
                }
                LexicalErrorKind::InvalidArraySize(a) => {
                    write!(
                        f,
                        "\nError: Invalid Array Size: \"{}\" {}{}\n",
                        a,
                        le.span.identifier(),
                        le.span.source_seg()
                    )
                }
                LexicalErrorKind::InvalidPrimitiveType(ty) => {
                    write!(
                        f,
                        "\nError: Invalid Primitive Type: \"{}\" {}{}\n",
                        ty,
                        le.span.identifier(),
                        le.span.source_seg()
                    )
                }
            },
            CompilerError::FileUnpackError(ue) => match ue {
                UnpackError::InvalidDirectory(id) => {
                    write!(f, "\nError: Invalid File Directory {}\n", id)
                }
                UnpackError::UnsupportedExtension(unsupported) => {
                    write!(
                        f,
                        "\nError: Unsupported File Extension \"{}\"\n--> {}\n",
                        parse_extension(unsupported).unwrap_or(""),
                        unsupported
                    )
                }
                UnpackError::MissingFile(file) => {
                    write!(f, "\nError: File Not Found \"{}\"\n", file)
                }
            },
            CompilerError::ParserError(pe) => match &pe.kind {
                ParserErrorKind::SyntaxError(se) => {
                    write!(f, "\nError: Syntax Error: \"{}\" \n{}\n", se, pe.spans.error())
                }
                ParserErrorKind::UnexpectedType(ut) => {
                    write!(f, "\nError: Unexpected Type: \"{}\" \n{}\n", ut, pe.spans.error())
                }
                ParserErrorKind::InvalidDefinition => {
                    write!(f, "\nError: Invalid Defintiion\n{}\n", pe.spans.error())
                }
                ParserErrorKind::InvalidConstantValue(cv) => {
                    write!(
                        f,
                        "\nError: Invalid Constant Value: \"{}\" \n{}\n",
                        cv,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidTokenInMacroBody(tmb) => {
                    write!(
                        f,
                        "\nError: Invalid Token In Macro Body: \"{}\" \n{}\n",
                        tmb,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidTokenInLabelDefinition(tlb) => {
                    write!(
                        f,
                        "\nError: Invalid Token In Label Defintiion: \"{}\" \n{}\n",
                        tlb,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidSingleArg(sa) => {
                    write!(f, "\nError: Invalid Argument: \"{}\" \n{}\n", sa, pe.spans.error())
                }
                ParserErrorKind::InvalidTableBodyToken(tbt) => {
                    write!(
                        f,
                        "\nError: Invalid Token In Table Body: \"{}\" \n{}\n",
                        tbt,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidConstant(constant) => {
                    write!(
                        f,
                        "\nError: Invalid Constant: \"{}\" \n{}\n",
                        constant,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidArgCallIdent(aci) => {
                    write!(
                        f,
                        "\nError: Invalid Argument Call Identifier: \"{}\" \n{}\n",
                        aci,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidName(name) => {
                    write!(f, "\nError: Invalid Name: \"{}\" \n{}\n", name, pe.spans.error())
                }
                ParserErrorKind::InvalidArgs(args) => {
                    write!(f, "\nError: Invalid Arguments: \"{}\" \n{}\n", args, pe.spans.error())
                }
                ParserErrorKind::InvalidUint256(v) => {
                    write!(f, "\nError: Invalid Uint256 Value: \"{}\" \n{}\n", v, pe.spans.error())
                }
                ParserErrorKind::InvalidBytes(b) => {
                    write!(f, "\nError: Invalid Bytes Value: \"{}\" \n{}\n", b, pe.spans.error())
                }
                ParserErrorKind::InvalidInt(i) => {
                    write!(f, "\nError: Invalid Int Value: \"{}\" \n{}\n", i, pe.spans.error())
                }
                ParserErrorKind::InvalidMacroArgs(ma) => {
                    write!(
                        f,
                        "\nError: Invalid Macro Arguments: \"{}\" \n{}\n",
                        ma,
                        pe.spans.error()
                    )
                }
                ParserErrorKind::InvalidReturnArgs => {
                    write!(f, "\nError: Invalid Return Arguments\n{}\n", pe.spans.error())
                }
                ParserErrorKind::InvalidImportPath(ip) => {
                    write!(f, "\nError: Invalid Import Path: \"{}\" \n{}\n", ip, pe.spans.error())
                }
            },
            CompilerError::PathBufRead(os_str) => {
                write!(
                    f,
                    "\nError: Invalid Import Path: \"{}\"",
                    os_str.as_os_str().to_str().unwrap_or("<unknown import>")
                )
            }
            CompilerError::CodegenError(ce) => match &ce.kind {
                CodegenErrorKind::StoragePointersNotDerived => {
                    write!(f, "\nError: Storage Pointers Not Derived\n{}\n", ce.span.error())
                }
                CodegenErrorKind::InvalidMacroStatement => {
                    write!(f, "\nError: Invalid Macro Statement\n{}\n", ce.span.error())
                }
                CodegenErrorKind::MissingMacroDefinition(md) => {
                    write!(
                        f,
                        "\nError: Missing Macro Definition For \"{}\"\n{}",
                        md,
                        ce.span.file()
                    )
                }
                CodegenErrorKind::InvalidMacroInvocation(mmi) => {
                    write!(
                        f,
                        "\nError: Missing Macro Definition For Invocation: \"{}\"\n{}\n",
                        mmi,
                        ce.span.error()
                    )
                }
                CodegenErrorKind::MissingConstantDefinition(_) => {
                    write!(f, "\nError: Missing Constant Definition\n{}\n", ce.span.error())
                }
                CodegenErrorKind::AbiGenerationFailure => {
                    write!(f, "\nError: ABI Generation Failed\n{}\n", ce.span.error())
                }
                CodegenErrorKind::IOError(ioe) => {
                    write!(f, "\nError: IO Error: {}\n{}", ioe, ce.span.file())
                }
                CodegenErrorKind::UnkownArgcallType => {
                    write!(f, "\nError: Unknown Arg Call Type\n{}\n", ce.span.error())
                }
                CodegenErrorKind::MissingMacroInvocation(mmi) => {
                    write!(
                        f,
                        "\nError: Missing Macro Invocation: \"{}\"\n{}\n",
                        mmi,
                        ce.span.error()
                    )
                }
                CodegenErrorKind::UnmatchedJumpLabel => {
                    write!(f, "\nError: Unmatched Jump Label\n{}\n", ce.span.error())
                }
                CodegenErrorKind::UsizeConversion(_) => {
                    write!(f, "\nError: Usize Conversion\n{}\n", ce.span.error())
                }
            },
            CompilerError::FailedCompiles(v) => {
                v.iter().for_each(|ce| {
                    let _ = write!(f, "{}", ce);
                });
                Ok(())
            }
        }
    }
}
