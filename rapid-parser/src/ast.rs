/// From: https://fs.gongkong.com/files/technicalData/201309/2013090913353800001.pdf

#[derive(PartialEq, Debug)]
pub enum Module {
    Module(ModuleInfo),
    Error,
}

#[derive(PartialEq, Debug)]
pub struct ModuleInfo {
    pub name: String,
    pub attributes: Vec<ModuleAttribute>,
    pub statements: Vec<Statement>,
}

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Scope {
    LOCAL,
    GLOBAL,
    TASK,
}

#[derive(PartialEq, Debug)]
pub enum Declaration {
    TypeDefinition(TypeDefinition),
    DataDeclaration(DataDeclaration),
    RoutineDeclaration(RoutineDeclaration),
}

pub fn tokenize_string(
    _l: usize,
    input: &str,
    r: usize,
) -> Result<String, lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'_>, &'static str>> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    // Skip the opening quote
    chars.next();

    loop {
        match chars.next() {
            Some('"') => {
                if chars.peek() == Some(&'"') {
                    // Escaped quote
                    chars.next();
                    result.push('"');
                } else {
                    // End of string
                    return Ok(result);
                }
            }
            Some('\\') => {
                if chars.peek() == Some(&'\\') {
                    // Escaped backslash
                    chars.next();
                    result.push('\\');
                } else {
                    // Hex sequence
                    let hex1 = chars.next().ok_or("Unexpected end of string after '\'")?;
                    let hex2 = chars
                        .next()
                        .ok_or("Unexpected end of string in hex sequence")?;
                    let hex_str = format!("{}{}", hex1, hex2);

                    if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                        result.push(byte as char);

                        // Check for optional backslash after hex sequence
                        if chars.peek() == Some(&'\\') {
                            chars.next();
                        }
                    } else {
                        return Err(lalrpop_util::ParseError::User {
                            error: "Invalid hex sequence",
                        });
                    }
                }
            }
            Some(ch) => {
                // Regular character
                result.push(ch);
            }
            None => {
                // Reached end of input without closing quote
                return Err(lalrpop_util::ParseError::UnrecognizedEof {
                    location: r,
                    expected: vec!["\"".to_string()],
                });
            }
        }
    }
}

// TODO: Move validate_module_attributes and validate_module_declarations to the sematic analysis

fn is_sorted<T>(data: &[T]) -> bool
where
    T: Ord,
{
    data.windows(2).all(|w| w[0] <= w[1])
}

pub fn validate_module_attributes<'input>(
    attrs: &Vec<ModuleAttribute>,
) -> Result<(), lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>> {
    if !is_sorted(attrs) {
        return Err(lalrpop_util::ParseError::User {
            error: "Module attributes are not in the correct order",
        });
    }
    for attr in attrs {
        match attr {
            ModuleAttribute::NOVIEW => {
                if attrs.contains(&ModuleAttribute::NOSTEPIN)
                    || attrs.contains(&ModuleAttribute::VIEWONLY)
                    || attrs.contains(&ModuleAttribute::READONLY)
                {
                    return Err(lalrpop_util::ParseError::User {
                        error: "NOVIEW attribute is mutually exclusive with NOSTEPIN, VIEWONLY, and READONLY",
                    });
                }
            }
            ModuleAttribute::VIEWONLY => {
                if attrs.contains(&ModuleAttribute::READONLY) {
                    return Err(lalrpop_util::ParseError::User {
                        error: "VIEWONLY attribute is mutually exclusive with READONLY",
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn validate_module_declarations<'input>(
    items: &Vec<Statement>,
) -> Result<(), lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>> {
    let mut seen_data_declaration = false;
    let mut seen_routine_declaration = false;
    for item in items {
        match item {
            Statement::TypeDefinition(_) => {
                if seen_data_declaration || seen_routine_declaration {
                    return Err(lalrpop_util::ParseError::User{ error: "Type definitions must come before data declarations and routine declarations" });
                }
            }
            Statement::DataDeclaration(_) => {
                if seen_routine_declaration {
                    return Err(lalrpop_util::ParseError::User {
                        error: "Data declarations must come before routine declarations",
                    });
                }
                seen_data_declaration = true;
            }
            Statement::RoutineDeclaration(_) => {
                seen_routine_declaration = true;
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn validate_routine_declarations<'input>(
    items: &Vec<Statement>,
) -> Result<(), lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>> {
    for item in items {
        match item {
            Statement::TypeDefinition(_) => {
                return Err(lalrpop_util::ParseError::User {
                    error: "Type definitions are not allowed inside routines",
                });
            }
            Statement::RoutineDeclaration(_) => {
                return Err(lalrpop_util::ParseError::User {
                    error: "Routine declarations are not allowed inside other routines",
                });
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Term(Term),
    Op(Box<Expr>, OpCode, Box<Expr>),
    FuncCall(String, Vec<Argument>),
    EXP,
    UnaryOp(OpCode, Box<Expr>),
}

#[derive(PartialEq, Debug)]
pub enum Term {
    String(String),
    Bool(bool),
    Num(f64),
    Array(Vec<Expr>),
    Var(Variable),
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum OpCode {
    Add,
    Sub,
    Mul,
    Div,
    DivInt,
    Mod,
    Lt,
    Lte,
    Eq,
    Gt,
    Gte,
    Ne,
    And,
    Or,
    Xor,
    Not,
}

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum ModuleAttribute {
    SYSMODULE,
    NOSTEPIN,
    VIEWONLY,
    READONLY,
    NOVIEW,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum TypeDefinition {
    RecordDefinition(Scope, RecordDefinition),
    AliasDefinition(Scope, AliasDefinition),
    TDN,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct RecordDefinition {
    pub name: String,
    pub components: Vec<RecordComponent>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct RecordComponent {
    pub data_type: String,
    pub name: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct AliasDefinition {
    pub name: String,
    pub data_type: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum AccessMode {
    IN,
    VAR,
    PERS,
    INOUT,
    REF,
}

#[derive(PartialEq, Debug)]
pub enum DataDeclaration {
    VarDeclaration(Scope, VarDeclaration),
    DDN,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum VarDeclarationType {
    VarDeclaration,
    PersDeclaration,
    ConstDeclaration,
}

#[derive(PartialEq, Debug)]
pub struct VarDeclaration {
    pub declaration_type: VarDeclarationType,
    pub data_type: String,
    pub definition: Definition,
}

#[derive(PartialEq, Debug)]
pub struct Definition {
    pub identifier: String,
    pub expression: Option<Expr>,
    pub dim: Option<Dimension>,
}

#[derive(PartialEq, Debug)]
pub enum RoutineDeclaration {
    ProcDeclaration(ProcDeclaration),
    FuncDeclaration(FuncDeclaration),
    TrapDeclaration(TrapDeclaration),
    RDN,
}

#[derive(PartialEq, Debug)]
pub struct ProcDeclaration {
    pub name: String,
    pub parameters: Vec<ParameterDeclarationType>,
    pub statements: Vec<Statement>,
    pub backward_handler: Option<Vec<Statement>>,
    pub error_handler: Option<ErrorHandler>,
    pub undo_handler: Option<Vec<Statement>>,
}

#[derive(PartialEq, Debug)]
pub struct FuncDeclaration {
    pub data_type: String,
    pub name: String,
    pub parameters: Vec<ParameterDeclarationType>,
    pub statements: Vec<Statement>,
    pub error_handler: Option<ErrorHandler>,
    pub undo_handler: Option<Vec<Statement>>,
}

#[derive(PartialEq, Debug)]
pub struct TrapDeclaration {
    pub name: String,
    pub statements: Vec<Statement>,
    pub error_handler: Option<ErrorHandler>,
    pub undo_handler: Option<Vec<Statement>>,
}

#[derive(PartialEq, Debug)]
pub enum ParameterDeclarationType {
    ParameterDeclaration(ParameterDeclaration),
    OptionalParameterDeclaration(Vec<OptionalParameterDeclarationType>),
    PAR,
}

#[derive(PartialEq, Debug)]
pub enum OptionalParameterDeclarationType {
    OptionalParameterDeclaration(ParameterDeclaration),
    Switch(String),
    ALT,
}

#[derive(PartialEq, Debug)]
pub struct ParameterDeclaration {
    pub access_mode: AccessMode,
    pub data_type: String,
    pub name: String,
    pub dim: Option<Dimension>,
}

#[derive(PartialEq, Debug)]
pub enum Statement {
    TypeDefinition(TypeDefinition),
    DataDeclaration(DataDeclaration),
    RoutineDeclaration(RoutineDeclaration),
    Label(String),
    Assignment(AssignmentTarget, Expr),
    // TODO: Make sure that dynamic binding is parsed and works!
    ProcCall(Expr, Vec<Argument>),
    Goto(String),
    Return(Option<Expr>),
    Raise(Option<Expr>),
    Exit,
    Retry,
    TryNext,
    Connect(String, String),
    If(
        Expr,
        Vec<Statement>,              // Statements
        Vec<(Expr, Vec<Statement>)>, // ElseIf Statements
        Vec<Statement>,              // Else Statements
    ),
    For(
        String,         // Loop Variable
        Expr,           // From
        Expr,           // To
        Option<Expr>,   // Step
        Vec<Statement>, // Statements
    ),
    While(
        Expr,           // Conditional Expression
        Vec<Statement>, // Statements
    ),
    Test(
        Expr,                   // Conditional Expression
        Vec<TestCase>,          // Cases
        Option<Vec<Statement>>, // Default
    ),
    Comment(String),
    SMT,
}

#[derive(PartialEq, Debug)]
pub enum AssignmentTarget {
    Variable(Variable),
    VAR, // TODO: Also remove because we shouldn't parse it
}

#[derive(PartialEq, Debug)]
pub enum TestCase {
    Case(
        Vec<Expr>,      // Conditional Expression
        Vec<Statement>, // Statements
    ),
    CSE,
}

#[derive(PartialEq, Debug)]
pub enum Variable {
    Variable(String),
    VariableElement(String, Dimension),
    VariableComponent(Box<Variable>, String),
}

#[derive(PartialEq, Debug)]
pub enum Parameter {
    Parameter(String),
    ParameterElement(String, Dimension),
    ParameterComponent(Box<Parameter>, String),
}

#[derive(PartialEq, Debug)]
pub enum Argument {
    Required(Option<String>, Expr),
    Optional(String, Option<Expr>),
    Conditional(String, Parameter, Parameter),
}

#[derive(PartialEq, Debug)]
pub enum Dimension {
    Dimension(Vec<Expr>),
    DIM,
}

#[derive(PartialEq, Debug)]
pub struct ErrorHandler {
    pub numbers: Vec<Expr>,
    pub statements: Vec<Statement>,
}
