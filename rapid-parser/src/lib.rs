pub mod ast;
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub rapid);

pub use lalrpop_util::ParseError;

/// Parses a RAPID module from the given input string.
///
/// # Arguments
///
/// * `input` - A string slice containing the RAPID module code to parse.
///
/// # Returns
///
/// A `Result` containing either the parsed `ast::Module` or a `ParseError`.
pub fn parse_module(
    input: &str,
) -> Result<ast::Module, ParseError<usize, lalrpop_util::lexer::Token<'_>, &'static str>> {
    rapid::ModuleParser::new().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_module_attributes() {
        let input = r#"MODULE test (SYSMODULE, VIEWONLY)
ENDMODULE"#;
        parse_module(&input).unwrap();
    }

    #[test]
    fn parse_module_without_attributes() {
        let input = r#"MODULE test
ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_alias() {
        let input = r#"
            MODULE test
                ALIAS num level;
                CONST level low := 2.5;
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_var_declarations() {
        let input = r#"
            MODULE test
                VAR num someVar;
                PERS num somePers;
                CONST num someConst;
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_data_declarations() {
        let input = r#"
            MODULE test
                LOCAL VAR num someVar;
                LOCAL PERS num somePers;
                LOCAL CONST num someConst;
                TASK VAR num someTaskVar;
                TASK PERS num somePers;
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_multi_dim_array() {
        let input = "VAR num someArray{2,2} := [[1,2],[1,2]];";
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_multi_dim_persistent_array() {
        let input = "PERS num someArray{2,2} := [[2,2],[3,3]];";
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_comments() {
        let input = r#"
            MODULE test
                ! Comment
                ! Another comment
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_tdn_placeholder() {
        let input = r#"
            MODULE test
                <TDN>
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_td_record() {
        let input = r#"
            MODULE test
                RECORD testRecord
                    num someNumber;
                    bool anotherRecord;
                ENDRECORD
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_td_decl_record() {
        let input = r#"
            MODULE test
                RECORD testRecord
                    num someNumber;
                    bool anotherRecord;
                ENDRECORD
                VAR testRecord someRecord;
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_dd_record() {
        let input = r#"
            MODULE test
                RECORD testRecord
                    num someNumber;
                    bool anotherRecord;
                ENDRECORD
                PERS num multiDim{2,2};
                VAR num multiDim2{3,3};
                PERS num anotherNumber := 1;
                VAR num someNumber := 5;
                VAR num assignedMultiDim{2,2} := [[3,3],[4,5]];
                PERS num assignedMultiDim2{2,2} := [[3,3],[4,5]];
                CONST num someConst := 3.14;
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_simple_proc_routine() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter, num anotherParameter)
                    VAR num someVariable;
                    someVariable := 5;
                ENDPROC
            ENDMODULE"#;
        assert!(parse_module(input).is_ok());
    }

    #[test]
    fn parse_error_missing_semicolon() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter, num anotherParameter)
                    VAR num someVariable
                    someVariable := 5;
                ENDPROC
            ENDMODULE"#;
        assert!(parse_module(input).is_err());
    }

    #[test]
    fn parse_par_parameter_declaration() {
        let input = r#"
            MODULE test
                PROC Test(<PAR>)
                    VAR num someVariable;
                    someVariable := 5;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_optional_parameter_declaration() {
        let input = r#"
            MODULE test
                PROC Test(num somParameter, \num optionalParameter)
                    VAR num someVariable;
                    someVariable := 5;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_multiple_optional_parameter_declaration() {
        let input = r#"
            MODULE test
                PROC Test(num somParameter, \num optionalParameter | num another | num andAnother)
                    VAR num someVariable;
                    someVariable := 5;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_switch_parameter_declaration() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter{2,2}, \switch on | switch off)
                    <STM>;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_parameter_declaration_par_ph() {
        let input = r#"
            MODULE test
                PROC Test(<PAR>)
                    <STM>;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_parameter_declaration_alt_ph() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter, \<ALT> | <ALT>)
                    <STM>;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_parameter_declaration_dim_ph() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter{2,2}, num someOther{<DIM>})
                    <STM>;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_parameter_declaration_dim_invalid() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter{3*radius,2})
                    <STM>;
                ENDPROC
            ENDMODULE"#;
        // NOTE: This should actually fail, but the RAPID parser in RobotStudio is also not complaining
        // parse_module(input).unwrap_err();
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_full_proce_routine() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter, num anotherParameter)
                    VAR num someVariable;
                    someVariable := 5;
                BACKWARD
                    someVariable := 0;
                ERROR
                    someVariable := 1;
                UNDO
                    someVariable := 2;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_full_proce_routine_with_custom_error_handler() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter, num anotherParameter)
                    VAR num someVariable;
                    someVariable := 5;
                BACKWARD
                    someVariable := 0;
                ERROR (56, ERR_DIVZERO)
                    someVariable := 1;
                UNDO
                    someVariable := 2;
                ENDPROC
            ENDMODULE"#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_trap_routine() {
        let input = r#"
            MODULE test
                PROC main()
                    VAR intnum hp;
                    CONNECT hp WITH high_pressure;
                ENDPROC
                TRAP high_pressure
                    close_valve\fast;
                    RETURN;
                ENDTRAP
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_var_assignment() {
        let input = r#"
            MODULE test
                PROC Test()
                    count := count + 1;
                    matrix{i,j} := temp;
                    posarr{i}.y := x;
                    a := "test";
                ENDPROC
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_procedure_call() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter)
                    <STM>;
                ENDPROC
                PROC AnotherTest()
                    VAR num someVar:=2;
                    Test someVar;
                ENDPROC
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_procedure_call_ph() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter)
                    <STM>;
                ENDPROC
                PROC AnotherTest()
                    VAR num someVar:=2;
                    Test <ARG>;
                ENDPROC
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_optional_and_conditional_procedure_call() {
        let input = r#"
            MODULE test
                PROC Test(num someParameter, \num optionalParam, \num anotherOptional)
                    <STM>;
                ENDPROC
                PROC AnotherTest()
                    VAR num someVar:=2;
                    VAR num anotherVar:=3;
                    Test someVar, \anotherOptionalParam ? someVar, anotherVar;
                ENDPROC
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_goto() {
        let input = r#"
            MODULE test
                PROC Test()
                    someLabel:
                    GOTO someLabel;
                ENDPROC
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_return_statement() {
        let input = r#"
            MODULE test
                PROC Test()
                    RETURN;
                ENDPROC
            ENDMODULE
        "#;
        parse_module(input).unwrap();
    }

    #[test]
    fn parse_if_statement() {
        let input = r#"
            IF counter > 100 THEN counter := 100;
            ELSEIF counter < 0 THEN counter := 0;
            ELSE
            counter := counter + 1;
            ENDIF
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_if_statement_placeholder() {
        let input = r#"
            IF counter > 100 THEN counter := 100;
            ELSEIF counter < 0 THEN <EIT>
            ELSE
            counter := counter + 1;
            ENDIF
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_compact_if_statement() {
        let input = r#"
            IF ERRNO = escape1 GOTO next;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_array_assignment_statement() {
        let input = r#"
            a{i} := b{i};
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_for_loop_statement() {
        let input = r#"
            FOR i FROM 10 TO 1 STEP -1 DO
                a{i} := b{i};
            ENDFOR
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_while_statement() {
        let input = r#"
            WHILE a < b DO
                a := a + 1;
            ENDWHILE
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_test_statement() {
        let input = r#"
            TEST choice
            CASE 1, 2, 3 :
                number := choice;
            CASE 4 :
                a := 4;
            DEFAULT:
                b := 10;
            ENDTEST
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_func_declaration() {
        let input = r#"
            FUNC num veclen(pos vector)
                RETURN sqrt(quad(vector.x) + quad(vector.y) + quad(vector.z));
            ERROR
                IF ERRNO = ERR_OVERFLOW THEN
                    RETURN maxnum;
                ENDIF
            ! propagate "unexpected" error
            RAISE;
            ENDFUNC
        "#;
        let result = rapid::RoutineDeclarationParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_simple_func_call_expression() {
        let input = r#"
            num result := veclen(vector);
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_nested_func_call() {
        let input = r#"
            num result := sqrt(quad(vector.x) + quad(vector.y));
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_func_call_two_parameters() {
        let input = r#"
            num result := polar(3.12, 0.785398);
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_func_call_with_names() {
        let input = r#"
            num result := polar(radius := 3.12, angle := 0.785398);
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_func_call_with_optional() {
        let input = r#"
            num result := polar(radius := 3.12, angle := 0.785398, \angle2 := 0.785398);
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_complex_string_literals() {
        let input = r#"
            someString := "This is a string with a ""quote"" in it";
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
        if let Ok(statement) = result {
            if let ast::Statement::Assignment(_, ast::Expr::Term(ast::Term::String(v))) = statement
            {
                assert_eq!(v, r#"This is a string with a "quote" in it"#);
            }
        }
    }

    #[test]
    fn parse_complex_string_with_hex() {
        let input = r#"
            someString := "This is a string with a BEL control character\07";
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
        if let Ok(statement) = result {
            if let ast::Statement::Assignment(_, ast::Expr::Term(ast::Term::String(v))) = statement
            {
                assert_eq!(v, "This is a string with a BEL control character\u{7}");
            }
        }
    }

    #[test]
    fn dont_parse_invalid_quotes() {
        let input = r#"
            VAR string someVar := "This is a string with a "";
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            lalrpop_util::ParseError::UnrecognizedEof {
                location: 62,
                expected: vec!["\"".to_string()]
            }
        );
    }

    #[test]
    fn parse_leading_dot_float_number() {
        let input = "VAR num someVar := .14;";
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_simple_not_expression() {
        let input = r#"
            VAR bool someVar := NOT true;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_simple_and_expression() {
        let input = r#"
            VAR bool someVar := true AND false;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_simple_or_expression() {
        let input = r#"
            VAR bool someVar := true OR false;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_simple_xor_expression() {
        let input = r#"
            VAR bool someVar := true XOR false;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_nested_or_and_expression() {
        let input = r#"
            VAR bool someVar := (true AND false) OR (false AND true);
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_single_nested_expression() {
        let input = r#"
            VAR bool someVar := (true);
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_late_binding_proc_call() {
        let input = r#"
            % "proc" + NumToStr(product_id, 0) % x, y, z;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_late_binding_proc_call_with_array() {
        let input = r#"
            % procname{product_id} % x, y, z;
        "#;
        let result = rapid::StatementParser::new().parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_complex_file_logger() {
        // Source from: https://raw.githubusercontent.com/robotics/open_abb/fuerte-devel/RAPID/LOGGER.mod
        let source = std::fs::read_to_string("data/LOGGER.mod").unwrap();
        let result = parse_module(&source);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_complex_server_module() {
        // Source from: https://raw.githubusercontent.com/robotics/open_abb/fuerte-devel/RAPID/SERVER.mod
        let source = std::fs::read_to_string("data/SERVER.mod").unwrap();
        let result = parse_module(&source);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
