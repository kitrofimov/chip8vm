use eframe::{egui, NativeOptions};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::collections::HashMap;

fn main() -> eframe::Result {
    let options = NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct TabViewer {
    tab_contents: HashMap<String, String>,
}

impl Default for TabViewer {
    fn default() -> Self {
        let mut tab_contents = HashMap::new();
        // Default content for each tab
        tab_contents.insert("tab1".to_string(), "// Tab 1 content".to_string());
        tab_contents.insert("tab2".to_string(), "// Tab 2 content".to_string());
        tab_contents.insert("tab3".to_string(), "// Tab 3 content".to_string());
        tab_contents.insert("tab4".to_string(), "// Tab 4 content".to_string());
        tab_contents.insert("tab5".to_string(), "// Tab 5 content".to_string());
        Self { tab_contents }
    }
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let content = self.tab_contents.entry(tab.clone()).or_insert_with(String::new);
        egui::TextEdit::multiline(content)
            .code_editor()
            .desired_width(f32::INFINITY)
            .show(ui);
    }
}

struct MyApp {
    tree: DockState<String>,
    tab_viewer: TabViewer,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut tree = DockState::new(vec!["tab1".to_owned(), "tab2".to_owned()]);

        // Example
        let [a, b] =
            tree.main_surface_mut()
                .split_left(NodeIndex::root(), 0.3, vec!["tab3".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(a, 0.7, vec!["tab4".to_owned()]);
        let [_, _] = tree
            .main_surface_mut()
            .split_below(b, 0.5, vec!["tab5".to_owned()]);

        Self { 
            tree,
            tab_viewer: TabViewer::default(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);
    }
}
