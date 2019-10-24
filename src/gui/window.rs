use rg3d_core::{
    color::Color,
    pool::Handle,
    math::vec2::Vec2,
};
use crate::gui::{
    event::UIEventKind,
    border::BorderBuilder,
    node::UINode,
    UserInterface,
    grid::{GridBuilder, Column, Row},
    HorizontalAlignment,
    text::TextBuilder,
    Thickness,
    button::ButtonBuilder,
    scroll_viewer::ScrollViewerBuilder,
    Layout,
    widget::{Widget, WidgetBuilder, AsWidget},
    Draw,
    draw::DrawingContext,
    Update
};

/// Represents a widget looking as window in Windows - with title, minimize and close buttons.
/// It has scrollable region for content, content can be any desired node or even other window.
/// Window can be dragged by its title.
pub struct Window {
    widget: Widget,
    mouse_click_pos: Vec2,
    initial_position: Vec2,
    is_dragged: bool,
}

impl AsWidget for Window {
    fn widget(&self) -> &Widget {
        &self.widget
    }

    fn widget_mut(&mut self) -> &mut Widget {
        &mut self.widget
    }
}

impl Update for Window {
    fn update(&mut self, dt: f32) {
        self.widget.update(dt)
    }
}

impl Layout for Window {
    fn measure_override(&self, ui: &UserInterface, available_size: Vec2) -> Vec2 {
        self.widget.measure_override(ui, available_size)
    }

    fn arrange_override(&self, ui: &UserInterface, final_size: Vec2) -> Vec2 {
        self.widget.arrange_override(ui, final_size)
    }
}

impl Draw for Window {
    fn draw(&mut self, drawing_context: &mut DrawingContext) {
        self.widget.draw(drawing_context)
    }
}

pub struct WindowBuilder<'a> {
    widget_builder: WidgetBuilder,
    content: Handle<UINode>,
    title: Option<WindowTitle<'a>>,
}

/// Window title can be either text or node.
///
/// If `Text` is used, then builder will automatically create Text node with specified text,
/// but with default font.
///
/// If you need more flexibility (i.e. put a picture near text) then `Node` option is for you:
/// it allows to put any UI node hierarchy you want to.
pub enum WindowTitle<'a> {
    Text(&'a str),
    Node(Handle<UINode>),
}

impl<'a> WindowBuilder<'a> {
    pub fn new(widget_builder: WidgetBuilder) -> Self {
        Self {
            widget_builder,
            content: Handle::NONE,
            title: None,
        }
    }

    pub fn with_content(mut self, content: Handle<UINode>) -> Self {
        self.content = content;
        self
    }

    pub fn with_title(mut self, title: WindowTitle<'a>) -> Self {
        self.title = Some(title);
        self
    }

    pub fn build(self, ui: &mut UserInterface) -> Handle<UINode> {
        let header = BorderBuilder::new(WidgetBuilder::new()
            .with_color(Color::opaque(120, 120, 120))
            .with_horizontal_alignment(HorizontalAlignment::Stretch)
            .with_height(30.0)
            .with_event_handler(Box::new(|ui, handle, evt| {
                if evt.source == handle {
                    match evt.kind {
                        UIEventKind::MouseDown { pos, .. } => {
                            ui.capture_mouse(handle);
                            let window_node = ui.borrow_by_criteria_up_mut(handle, |node| node.is_window());
                            let initial_position = window_node.widget().actual_local_position.get();
                            let window = window_node.as_window_mut();
                            window.mouse_click_pos = pos;
                            window.initial_position = initial_position;
                            window.is_dragged = true;
                            evt.handled = true;
                        }
                        UIEventKind::MouseUp { .. } => {
                            ui.release_mouse_capture();
                            let window_node = ui.borrow_by_criteria_up_mut(handle, |node| node.is_window());
                            window_node.as_window_mut().is_dragged = false;
                            evt.handled = true;
                        }
                        UIEventKind::MouseMove { pos, .. } => {
                            let window = ui.borrow_by_criteria_up_mut(handle, |node| node.is_window()).as_window_mut();
                            let new_pos =
                                if window.is_dragged {
                                    window.initial_position + pos - window.mouse_click_pos
                                } else {
                                    return;
                                };

                            window.widget.set_desired_local_position(new_pos);
                            evt.handled = true;
                        }
                        _ => ()
                    }
                }
            }))
            .with_child(GridBuilder::new(WidgetBuilder::new()
                .with_child({
                    match self.title {
                        None => Handle::NONE,
                        Some(window_title) => {
                            match window_title {
                                WindowTitle::Node(node) => node,
                                WindowTitle::Text(text) => {
                                    TextBuilder::new(WidgetBuilder::new()
                                        .with_margin(Thickness::uniform(5.0))
                                        .on_row(0)
                                        .on_column(0))
                                        .with_text(text)
                                        .build(ui)
                                }
                            }
                        }
                    }
                })
                .with_child(ButtonBuilder::new(WidgetBuilder::new()
                    .on_row(0)
                    .on_column(1)
                    .with_margin(Thickness::uniform(2.0)))
                    .with_text("_")
                    .build(ui))
                .with_child(ButtonBuilder::new(WidgetBuilder::new()
                    .on_row(0)
                    .on_column(2)
                    .with_margin(Thickness::uniform(2.0)))
                    .with_text("X")
                    .build(ui)))
                .add_column(Column::stretch())
                .add_column(Column::strict(30.0))
                .add_column(Column::strict(30.0))
                .add_row(Row::stretch())
                .build(ui))
            .on_row(0)
        ).build(ui);

        let scroll_viewer = ScrollViewerBuilder::new(WidgetBuilder::new()
            .on_row(1))
            .with_content(self.content)
            .build(ui);

        let window = UINode::Window(Window {
            widget: self.widget_builder
                .with_child(BorderBuilder::new(WidgetBuilder::new()
                    .with_child(GridBuilder::new(WidgetBuilder::new()
                        .with_child(scroll_viewer)
                        .with_child(header))
                        .add_column(Column::stretch())
                        .add_row(Row::auto())
                        .add_row(Row::stretch())
                        .build(ui))
                    .with_color(Color::opaque(100, 100, 100)))
                    .build(ui))
                .build(),
            mouse_click_pos: Vec2::ZERO,
            initial_position: Vec2::ZERO,
            is_dragged: false,
        });
        ui.add_node(window)
    }
}