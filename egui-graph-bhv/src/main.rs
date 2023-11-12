use bhv_async::prelude::*;
use eframe::egui;
use egui_node_graph::*;

fn main() {
    use eframe::egui::Visuals;

    eframe::run_native(
        "Egui node graph example",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            Box::<DemoApp>::default()
        }),
    )
    .expect("Failed to run native example");
}

/// Note query state
#[derive(Clone)]
enum BhvNodeTemplate {}
struct BhvUserState {
    pub active_node: Option<NodeId>,
}

/// Node Info
#[derive(Default, Clone)]
struct BhvNodeData {
    pub composite: Option<Composite>,
}
/// process input data for node
#[derive(PartialEq, Eq)]
enum BhvDataType {}
struct BhvValueType {}
type BhvGraph = Graph<BhvNodeData, BhvDataType, BhvValueType>;
type BhvEditorState =
    GraphEditorState<BhvNodeData, BhvDataType, BhvValueType, BhvNodeTemplate, BhvUserState>;
// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<BhvUserState> for BhvDataType {
    fn name(&self) -> std::borrow::Cow<str> {
        std::borrow::Cow::Borrowed("What is this kind")
    }

    fn data_type_color(&self, user_state: &mut BhvUserState) -> egui::Color32 {
        egui::Color32::from_rgb(38, 109, 211)
    }
}

impl NodeTemplateTrait for BhvNodeTemplate {
    type NodeData = BhvNodeData;

    type DataType = BhvDataType;

    type ValueType = BhvValueType;

    type UserState = BhvUserState;

    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        std::borrow::Cow::Borrowed("This is node_finder_label")
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        "This is node_graph_label".into()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        BhvNodeData::default()
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // empty
    }
}

#[derive(Default)]
pub struct DemoApp;

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        todo!()
    }
}
