use druid::im::Vector;
use druid::{
    widget::{Flex, Label, List, RadioGroup},
    AppLauncher, Data, Lens, PlatformError, Widget, WidgetExt, WindowDesc,
};

#[derive(Clone, Data, PartialEq)]
enum Category {
    One,
    Two,
    Three,
}

#[derive(Lens, Data, Clone)]
struct ListItem {
    text: String,
    category: Category,
}
impl ListItem {
    fn new(text: &str, category: Category) -> Self {
        Self {
            text: text.into(),
            category,
        }
    }
}

#[derive(Lens, Data, Clone)]
struct AppState {
    category: Category,
    items: Vector<ListItem>,
}
impl AppState {
    fn new() -> Self {
        let mut items = Vector::new();
        items.push_front(ListItem::new("black", Category::One));
        items.push_front(ListItem::new("white", Category::One));
        items.push_front(ListItem::new("red", Category::Two));
        items.push_front(ListItem::new("blue", Category::Two));
        items.push_front(ListItem::new("green", Category::Three));
        items.push_front(ListItem::new("purple", Category::Three));
        Self {
            category: Category::One,
            items,
        }
    }
}

fn build_ui() -> impl Widget<AppState> {
    let filtering = RadioGroup::row(vec![
        ("one", Category::One),
        ("two", Category::Two),
        ("three", Category::Three),
    ])
    .lens(AppState::category);
    let items = List::new(|| Label::raw().lens(ListItem::text)).lens(AppState::items);
    Flex::column().with_child(filtering).with_child(items)
}

fn main() -> Result<(), PlatformError> {
    let window = WindowDesc::new(build_ui())
        .title("my app")
        .window_size((1000., 500.));
    let inital_state = AppState::new();
    AppLauncher::with_window(window).launch(inital_state)?;
    Ok(())
}
