use std::path::PathBuf;

use crate::application::adapters::ApplicationAdapters;
use crate::application::service::{ActionExecutor, ApplicationService};
use crate::application::types::{ApplicationOutcome, CommandAction, GenericSuccess};
use crate::domain::graph::{
    GraphTraversal, QualifiedSymbolName, SymbolName, TraversalDirection, TraversalFilter,
    TypeGraph,
};
use crate::ux_model::intent::{UserCommandContext, UserCommandFrom, UserGoal, UserGoalType, UserPromise, UserPromiseType};
use crate::ux_model::result::{UserResult, UserResultContext, UserSummary, UserNextStep, UserLimitation};

impl CommandAction for UserCommandFrom {
    type Success = GenericSuccess;
    type Partial = crate::application::types::GenericPartial;
    type Failure = crate::application::types::GenericFailure;
}

impl<A: ApplicationAdapters> ActionExecutor<UserCommandFrom> for ApplicationService<A> {
    async fn execute_action(
        &self,
        action: UserCommandFrom,
        context: UserCommandContext,
    ) -> ApplicationOutcome<UserCommandFrom> {
        // For now, create a mock graph until we have LSP integration
        // TODO: Replace with actual LSP queries to build real graph
        let graph = create_mock_graph();

        // Extract symbol name from target
        let target_str = match &action.target {
            crate::ux_model::intent::FromTarget::Symbol(s) => s.clone(),
            crate::ux_model::intent::FromTarget::File(_) 
            | crate::ux_model::intent::FromTarget::Module(_)
            | crate::ux_model::intent::FromTarget::PublicExports => {
                return UserResult::Failure(crate::application::types::GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreExplicit),
                    ],
                    summary: UserSummary("Only symbol-based traversal is currently implemented".to_string()),
                    limitations: vec![UserLimitation("File, module, and public exports traversal are not yet supported".to_string())],
                    next_step: UserNextStep("Use a symbol name instead, e.g., 'typeglass from MyType'".to_string()),
                    context: UserResultContext {
                        command_context: context,
                    },
                });
            }
        };

        // Parse the target symbol
        let symbol_name = match SymbolName::new(target_str.clone()) {
            Ok(name) => name,
            Err(_) => {
                return UserResult::Failure(crate::application::types::GenericFailure {
                    promises: vec![
                        UserPromise(UserPromiseType::NeverSilentWrong),
                        UserPromise(UserPromiseType::ErrorsAreExplicit),
                    ],
                    summary: UserSummary(format!("Invalid symbol name: {}", target_str)),
                    limitations: vec![UserLimitation("Symbol names cannot be empty or contain whitespace".to_string())],
                    next_step: UserNextStep("Provide a valid symbol name".to_string()),
                    context: UserResultContext {
                        command_context: context,
                    },
                });
            }
        };

        // Create qualified symbol name (mock module path for now)
        let entry_point = QualifiedSymbolName {
            module_path: PathBuf::from("mock.ts"),
            symbol: symbol_name,
        };

        // Use default direction (Both) and filter
        let direction = TraversalDirection::Both;
        let filter = TraversalFilter::default();

        // Execute traversal
        let max_depth = action.depth.unwrap_or(5);
        let mut traversal = GraphTraversal::new(max_depth);
        let result = traversal.traverse(&graph, &entry_point, direction, &filter);

        // Format results
        let node_count = result.graph.nodes().len();
        let edge_count = result.graph.edges().len();
        let completeness = if result.graph.is_complete() {
            "complete"
        } else {
            "partial"
        };

        let summary = format!(
            "Found {} nodes and {} edges from '{}' ({} graph, max depth: {})",
            node_count, edge_count, target_str, completeness, result.max_depth_reached
        );

        UserResult::Success(GenericSuccess {
            goal: UserGoal(UserGoalType::UnderstandCodebaseDomain),
            promises: vec![
                UserPromise(UserPromiseType::NeverSilentWrong),
                UserPromise(UserPromiseType::FastByDefault),
                UserPromise(UserPromiseType::PartialResultsAreExplicit),
            ],
            summary: UserSummary(summary),
            next_step: Some(UserNextStep(
                "Graph traversal complete. Use --format json for detailed output.".to_string(),
            )),
            context: UserResultContext {
                command_context: context,
            },
        })
    }
}

// Mock graph builder - TODO: Replace with real LSP-based graph builder
fn create_mock_graph() -> TypeGraph {
    use crate::domain::graph::{EdgeKind, SourceLocation, SymbolKind, SymbolOrigin, TypeEdge, TypeNode};
    use crate::domain::language::Language;

    let mut graph = TypeGraph::empty();

    // Add some mock nodes
    let node_a = TypeNode {
        id: QualifiedSymbolName {
            module_path: PathBuf::from("mock.ts"),
            symbol: SymbolName("TypeA".to_string()),
        },
        name: SymbolName("TypeA".to_string()),
        kind: SymbolKind::Struct,
        origin: SymbolOrigin::Canonical,
        location: SourceLocation {
            file: PathBuf::from("mock.ts"),
            line: 10,
            column: 1,
        },
        language: Language::TypeScript,
        generic_parameters: vec![],
    };

    let node_b = TypeNode {
        id: QualifiedSymbolName {
            module_path: PathBuf::from("mock.ts"),
            symbol: SymbolName("TypeB".to_string()),
        },
        name: SymbolName("TypeB".to_string()),
        kind: SymbolKind::Struct,
        origin: SymbolOrigin::Canonical,
        location: SourceLocation {
            file: PathBuf::from("mock.ts"),
            line: 20,
            column: 1,
        },
        language: Language::TypeScript,
        generic_parameters: vec![],
    };

    graph.add_node(node_a);
    graph.add_node(node_b);
    graph.add_edge(TypeEdge {
        from: SymbolName("TypeA".to_string()),
        to: SymbolName("TypeB".to_string()),
        kind: EdgeKind::Contains,
    });

    graph
}
