use {
    // Use volt to resolve items in derive macro.
    crate as volt,
    crate::*,
    crate::parser::*,
    crate::tree::*,
    volt_derive::VoltModuleDefinition,
    speculate::speculate,
};

speculate!{
    before {
        let volt = &mut Volt::new();
        volt.add_module(TestModule::new());

        let assert_ast = |input: &str, rule_id: &str, expected: ParserResult|
            assert_eq!(Parser::parse(volt, input, &RuleId(rule_id.to_string())), expected);

        #[allow(unused)]
        let expect_success = |input: &str, rule_id: &str, expected: SyntaxTree|
            assert_ast(input, rule_id, Ok(expected));

        #[allow(unused)]
        let expect_failure = |input: &str, rule_id: &str, expected: ParserError|
            assert_ast(input, rule_id, Err(expected));
    }

    describe "input index" {
        it "generates line start indexes" {
            assert_eq!(InputPositionCounter::from(""), (
                InputPositionCounter {
                    lines: vec![(0, 0)],
                }
            ));

            assert_eq!(InputPositionCounter::from("a\nあ\n"), (
                InputPositionCounter {
                    lines: vec![(0, 2), (2, 2), (4, 0)],
                }
            ));
        }

        it "gets input position" {
            expect_success("a\na\n", "TestModule::input_index", tree!(
                node!("TestModule::input_index" => [
                    leaf!(pos!(0, 0, 0), "a"),
                    leaf!(pos!(1, 0, 1), "\n"),
                    leaf!(pos!(2, 1, 0), "a"),
                    leaf!(pos!(3, 1, 1), "\n"),
                ])
            ));
        }
    }

    // it "detect max recursion excess" {
    //     expect_failure("", "TestModule::left_recursion", ParserError::ExceededMaxRecursion);
    // }

    describe "choice element" {
        it "choice consumes characters as much its children 1" {
            expect_failure("", "TestModule::choice", ParserError::NoMatchedRule);
        }

        it "choice consumes characters as much its children 2" {
            expect_failure("ab", "TestModule::choice", ParserError::NoMatchedRule);
        }

        it "choices first choice when match" {
            expect_success("a", "TestModule::choice", tree!(
                node!("TestModule::choice" => [
                    leaf!("a"),
                ])
            ));
        }

        it "choices the next choice when first choice doesn't match" {
            expect_success("b", "TestModule::choice", tree!(
                node!("TestModule::choice" => [
                    leaf!("b"),
                ])
            ));
        }

        it "choice doesn't match element not exist in children" {
            expect_failure("c", "TestModule::choice", ParserError::NoMatchedRule);
        }
    }

    describe "sequence element" {
        it "sequence consumes characters as much its children 1" {
            expect_failure("a", "TestModule::sequence", ParserError::NoMatchedRule);
        }

        it "sequence consumes characters as much its children 2" {
            expect_failure("abc", "TestModule::sequence", ParserError::NoMatchedRule);
        }

        it "sequence matches completely same input 1" {
            expect_success("ab", "TestModule::sequence", tree!(
                node!("TestModule::sequence" => [
                    leaf!("a"),
                    leaf!("b"),
                ])
            ));
        }

        it "sequence matches completely same input 2" {
            expect_failure("ac", "TestModule::sequence", ParserError::NoMatchedRule);
        }
    }

    describe "loop element" {
        describe "n times" {
            it "repeats for the number of times in the specified range 1-1" {
                expect_failure("a", "TestModule::loop_range1", ParserError::NoMatchedRule);
            }

            it "repeats for the number of times in the specified range 1-2" {
                expect_success("aa", "TestModule::loop_range1", tree!(
                    node!("TestModule::loop_range1" => [
                        leaf!("a"),
                        leaf!("a"),
                    ])
                ));
            }

            it "repeats for the number of times in the specified range 1-3" {
                expect_failure("aaa", "TestModule::loop_range1", ParserError::NoMatchedRule);
            }
        }

        describe "min" {
            it "repeats for the number of times in the specified range 2-1" {
                expect_failure("", "TestModule::loop_range2", ParserError::NoMatchedRule);
            }

            it "repeats for the number of times in the specified range 2-2" {
                expect_success("a", "TestModule::loop_range2", tree!(
                    node!("TestModule::loop_range2" => [
                        leaf!("a"),
                    ])
                ));
            }

            it "repeats for the number of times in the specified range 2-3" {
                expect_success("aa", "TestModule::loop_range2", tree!(
                    node!("TestModule::loop_range2" => [
                        leaf!("a"),
                        leaf!("a"),
                    ])
                ));
            }
        }

        describe "max" {
            it "repeats for the number of times in the specified range 3-1" {
                expect_success("", "TestModule::loop_range3", tree!(
                    node!("TestModule::loop_range3" => [])
                ));
            }

            it "repeats for the number of times in the specified range 3-2" {
                expect_success("a", "TestModule::loop_range3", tree!(
                    node!("TestModule::loop_range3" => [
                        leaf!("a"),
                    ])
                ));
            }

            it "repeats for the number of times in the specified range 3-3" {
                expect_failure("aa", "TestModule::loop_range3", ParserError::NoMatchedRule);
            }
        }
    }

    describe "positive lookahead element" {
        it "doesn't change input index 1" {
            expect_success("a", "TestModule::poslook", tree!(
                node!("TestModule::poslook" => [
                    leaf!("a"),
                ])
            ));
        }

        it "doesn't change input index 2" {
            expect_failure("b", "TestModule::poslook", ParserError::NoMatchedRule);
        }
    }

    describe "negative lookahead element" {
        it "doesn't change input index 1" {
            expect_failure("a", "TestModule::neglook", ParserError::NoMatchedRule);
        }

        it "doesn't change input index 2" {
            expect_success("b", "TestModule::neglook", tree!(
                node!("TestModule::neglook" => [
                    leaf!("b"),
                ])
            ));
        }
    }

    describe "error element" {
        it "generates error when matched" {
            expect_success("a", "TestModule::error", tree!(
                node!("TestModule::error" => [
                    error!("msg", [
                        leaf!("a"),
                    ]),
                ])
            ));
        }

        it "doesn't add input index when not matched" {
            expect_success("", "TestModule::error", tree!(
                node!("TestModule::error" => [])
            ));
        }
    }

    describe "catch element" {
        it "successes parsing normally" {
            expect_success("aa", "TestModule::catch", tree!(
                node!("TestModule::catch" => [
                    leaf!("a"),
                    leaf!("a"),
                ])
            ));
        }

        it "doesn't add input index on failure" {
            expect_success("b", "TestModule::catch", tree!(
                node!("TestModule::catch" => [
                    error!("msg", []),
                    leaf!("b"),
                ])
            ));
        }
    }

    describe "catch skip element" {
        it "successes parsing normally" {
            expect_success("a;", "TestModule::catch_to", tree!(
                node!("TestModule::catch_to" => [
                    leaf!("a"),
                    leaf!(";"),
                ])
            ));
        }

        it "adds input index until end string on failure" {
            expect_success("b;", "TestModule::catch_to", tree!(
                node!("TestModule::catch_to" => [
                    error!("msg", [
                        leaf!(";"),
                    ]),
                ])
            ));
        }

        it "try parsing until end of input" {
            expect_failure("b", "TestModule::catch_to", ParserError::NoMatchedRule);
        }
    }

    describe "tree reduction element" {
        it "modify the result when input matched syntax rule" {
            expect_success("a", "TestModule::tree_reduction", tree!(
                node!("TestModule::tree_reduction" => [
                    leaf!(InputPosition::new(0, 0, 0), "reduced"),
                ])
            ));
        }

        it "doesn't modify the result when input didn't match syntax rule" {
            expect_failure("b", "TestModule::tree_reduction", ParserError::NoMatchedRule);
        }
    }

    describe "group element" {
        it "group a sequence" {
            expect_success("aa", "TestModule::sequence_group", tree!(
                node!("TestModule::sequence_group" => [
                    node!("group" => [
                        leaf!("a"),
                        leaf!("a"),
                    ])
                ])
            ));
        }

        it "group an expression" {
            expect_success("a", "TestModule::expression_group", tree!(
                node!("TestModule::expression_group" => [
                    node!("group" => [
                        leaf!("a"),
                    ])
                ])
            ));
        }
    }

    describe "expansion element" {
        it "expands children at all levels of hierarchy" {
            expect_success("abc", "TestModule::expansion", tree!(
                node!("TestModule::expansion" => [
                    leaf!("a"),
                    leaf!("b"),
                    leaf!("c"),
                ])
            ));
        }

        it "expands children at the first level of hierarchy" {
            expect_success("abc", "TestModule::expansion_once", tree!(
                node!("TestModule::expansion_once" => [
                    leaf!("a"),
                    leaf!("b"),
                    node!("group_b" => [
                        leaf!("c"),
                    ]),
                ])
            ));
        }
    }

    describe "join element" {
        it "a" {
            expect_success("aa", "TestModule::join", tree!(
                node!("TestModule::join" => [
                    leaf!("aa")
                ])
            ));
        }

        it "b" {
            expect_success("aaa", "TestModule::errors_in_join", tree!(
                node!("TestModule::errors_in_join" => [
                    leaf!("a"),
                    error!("e1", [
                        leaf!("a"),
                    ]),
                    error!("e2", [
                        leaf!("a"),
                    ]),
                ])
            ));
        }
    }

    describe "hidden element" {
        it "element shouldn't reflected in AST" {
            expect_success("a", "TestModule::hidden", tree!(
                node!("TestModule::hidden" => [])
            ));
        }
    }

    describe "around element" {
        it "should have one item" {
            expect_failure("", "TestModule::around", ParserError::NoMatchedRule);
        }

        it "should have enclosure at both side" {
            expect_failure("a", "TestModule::around", ParserError::NoMatchedRule);

            expect_success("'a'", "TestModule::around", tree!(
                node!("TestModule::around" => [
                    leaf!("'"),
                    leaf!("a"),
                    leaf!("'"),
                ])
            ));
        }
    }

    describe "separated element" {
        it "should have at least one item" {
            expect_failure("", "TestModule::separated", ParserError::NoMatchedRule);
        }

        it "can put single item" {
            expect_success("a", "TestModule::separated", tree!(
                node!("TestModule::separated" => [
                    leaf!("a"),
                ])
            ));
        }

        it "can put a separator at the last of single item" {
            expect_success("a,", "TestModule::separated", tree!(
                node!("TestModule::separated" => [
                    leaf!("a"),
                    leaf!(","),
                ])
            ));
        }

        it "can put multiple items" {
            expect_success("a,a", "TestModule::separated", tree!(
                node!("TestModule::separated" => [
                    leaf!("a"),
                    leaf!(","),
                    leaf!("a"),
                ])
            ));
        }

        it "can put a separator at the last of multiple items" {
            expect_success("a,a,", "TestModule::separated", tree!(
                node!("TestModule::separated" => [
                    leaf!("a"),
                    leaf!(","),
                    leaf!("a"),
                    leaf!(","),
                ])
            ));
        }

        it "separators can be hidden" {
            expect_success("a,a", "TestModule::separated_with_hidden_separator", tree!(
                node!("TestModule::separated_with_hidden_separator" => [
                    leaf!("a"),
                    leaf!("a"),
                ])
            ));
        }

        it "accept first separator optionally when separated around" {
            expect_success("a", "TestModule::separated_around", tree!(
                node!("TestModule::separated_around" => [
                    leaf!("a"),
                ])
            ));

            expect_success(",a", "TestModule::separated_around", tree!(
                node!("TestModule::separated_around" => [
                    leaf!(","),
                    leaf!("a"),
                ])
            ));
        }
    }

    describe "string expression" {
        it "string consumes characters as much its length 1" {
            expect_failure("a", "TestModule::string", ParserError::NoMatchedRule);
        }

        it "string consumes characters as much its length 2" {
            expect_failure("abc", "TestModule::string", ParserError::NoMatchedRule);
        }

        it "string generates single leaf" {
            expect_success("ab", "TestModule::string", tree!(
                node!("TestModule::string" => [
                    leaf!("ab"),
                ])
            ));
        }

        it "string supports multibyte characters" {
            expect_success("あい", "TestModule::multibyte_string", tree!(
                node!("TestModule::multibyte_string" => [
                    leaf!("あい"),
                ])
            ));
        }
    }

    describe "character class expression" {
        it "matches a specified character 1" {
            expect_success("a", "TestModule::character_class1", tree!(
                node!("TestModule::character_class1" => [
                    leaf!("a"),
                ])
            ));
        }

        it "matches a specified character 2" {
            expect_success("b", "TestModule::character_class1", tree!(
                node!("TestModule::character_class1" => [
                    leaf!("b"),
                ])
            ));
        }

        it "matches a specified character 3" {
            expect_failure("c", "TestModule::character_class1", ParserError::NoMatchedRule);
        }

        it "consumes only one character" {
            expect_failure("aa", "TestModule::character_class1", ParserError::NoMatchedRule);
        }

        it "supports number specification" {
            expect_success("0", "TestModule::character_class2", tree!(
                node!("TestModule::character_class2" => [
                    leaf!("0"),
                ])
            ));
        }

        it "supports regex pattern enclosure" {
            expect_success("[", "TestModule::character_class3", tree!(
                node!("TestModule::character_class3" => [
                    leaf!("["),
                ])
            ));
        }
    }

    describe "wildcard expression" {
        it "wildcard consumes single character 1" {
            expect_failure("", "TestModule::wildcard", ParserError::NoMatchedRule);
        }

        it "wildcard consumes single character 2" {
            expect_failure("aa", "TestModule::wildcard", ParserError::NoMatchedRule);
        }

        it "wildcard generates single leaf" {
            expect_success("a", "TestModule::wildcard", tree!(
                node!("TestModule::wildcard" => [
                    leaf!("a"),
                ])
            ));
        }

        it "wildcard treats single multibyte character as a character" {
            expect_success("あ", "TestModule::wildcard", tree!(
                node!("TestModule::wildcard" => [
                    leaf!("あ"),
                ])
            ));
        }
    }
}

#[derive(VoltModuleDefinition)]
struct TestModule {
    input_index: Element,
    // left_recursion: Element,
    choice: Element,
    sequence: Element,
    loop_range1: Element,
    loop_range2: Element,
    loop_range3: Element,
    poslook: Element,
    neglook: Element,
    error: Element,
    catch: Element,
    catch_to: Element,
    tree_reduction: Element,
    sequence_group: Element,
    expression_group: Element,
    expansion: Element,
    expansion_once: Element,
    join: Element,
    errors_in_join: Element,
    hidden: Element,
    around: Element,
    separated: Element,
    separated_with_hidden_separator: Element,
    separated_around: Element,
    string: Element,
    multibyte_string: Element,
    character_class1: Element,
    character_class2: Element,
    character_class3: Element,
    wildcard: Element,
}

impl VoltModule for TestModule {
    fn new() -> TestModule {
        define_rules!{
            input_index := seq![str("a"), str("\n"), str("a"), str("\n")];
            // left_recursion := TestModule::left_recursion();
            choice := choice![str("a"), str("b")];
            sequence := seq![str("a"), str("b")];
            loop_range1 := seq![wildcard().times(2)];
            loop_range2 := seq![wildcard().min(1)];
            loop_range3 := seq![wildcard().max(1)];
            poslook := seq![str("a").poslook(), wildcard()];
            neglook := seq![str("a").neglook(), wildcard()];
            error := str("a").err("msg");
            catch := seq![str("a").catch("msg"), wildcard()];
            catch_to := seq![str("a"), str(";")].catch_to("msg", str(";"));
            tree_reduction := str("a").reduce(|v| vec![SyntaxChild::leaf(v.get_start_position().unwrap(), "reduced".to_string())]);
            sequence_group := seq![wildcard(), wildcard()].group("group");
            expression_group := wildcard().group("group");
            expansion := seq![wildcard(), seq![wildcard(), seq![wildcard()].group("group_b")].group("group_a").expand()];
            expansion_once := seq![wildcard(), seq![wildcard(), seq![wildcard()].group("group_b")].group("group_a").expand_once()];
            join := seq![wildcard(), seq![wildcard()].group("g")].join();
            errors_in_join := seq![wildcard(), wildcard().err("e1"), seq![wildcard().err("e2")].group("g")].join();
            hidden := wildcard().hide();
            around := wildcard().around(str("'"));
            separated := wildcard().separate(str(","));
            separated_with_hidden_separator := wildcard().separate(str(",").hide());
            separated_around := wildcard().separate_around(str(","));
            string := str("ab");
            multibyte_string := str("あい");
            character_class1 := chars("ab");
            character_class2 := chars(r"\d");
            character_class3 := chars("[");
            wildcard := wildcard();
        }
    }
}
