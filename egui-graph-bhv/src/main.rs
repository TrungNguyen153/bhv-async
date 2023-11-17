// use bhv_async::prelude::*;
use eframe::{egui, run_native, App, CreationContext};
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsStyle,
};
use petgraph::{stable_graph::StableGraph, Directed};

pub struct InteractiveApp {
    g: Graph<String, u32, Directed>,
}

impl InteractiveApp {
    fn new(_: &CreationContext<'_>) -> Self {
        let g = generate_graph();
        Self { g }
    }
}

impl App for InteractiveApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let interaction_settings = &SettingsInteraction::new()
                .with_dragging_enabled(true)
                .with_node_clicking_enabled(true)
                .with_node_selection_enabled(true)
                .with_node_selection_multi_enabled(true)
                .with_edge_clicking_enabled(true)
                .with_edge_selection_enabled(true)
                .with_edge_selection_multi_enabled(true);
            let style_settings = &SettingsStyle::new().with_labels_always(true);
            ui.add(
                &mut GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape<_>>::new(
                    &mut self.g,
                )
                .with_styles(style_settings)
                .with_interactions(interaction_settings),
            );
        });
    }
}

fn generate_graph() -> Graph<String, u32> {
    let mut g = StableGraph::new();

    let a = g.add_node("NodeA".to_string());
    let b = g.add_node("NodeB".into());
    let c = g.add_node("NodeC".into());

    // parent_node, node id, 1

    g.add_edge(a, b, 1);
    g.add_edge(b, c, 99);
    g.add_edge(c, a, 1000);

    Graph::from(&g)
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    run_native(
        "egui_graphs_interactive_demo",
        native_options,
        Box::new(|cc| Box::new(InteractiveApp::new(cc))),
    )
    .unwrap();
}
