use druid::im::Vector;
use druid::widget::Controller;
use druid::{
    widget::{Button, Checkbox, Flex, Label, List, Radio, RadioGroup, TextBox, ListIter},
    AppDelegate, AppLauncher, Data, Env, Lens, PlatformError, Selector, Widget, WidgetExt,
    WindowDesc,
};
use druid::{Command, DelegateCtx, EventCtx, Handled, Target, UpdateCtx};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SAVE: Selector = Selector::new("todo.save");
const DELETE: Selector<Uuid> = Selector::new("todo.delete");

#[derive(Clone, Data, PartialEq, Debug)]
enum MyRadio {
    GaGa,
    GuGu,
    BaaBaa,
}

#[derive(Lens, Data, Clone, Debug)]
struct AppState {
    radio: MyRadio,
    new_item_text: String,
    items: Vector<TodoItem>,
}
impl AppState {
    fn new() -> Self {
        Self {
            radio: MyRadio::GaGa,
            new_item_text: "".to_string(),
            items: Vector::new(),
        }
    }
    fn click_add_new_item(ctx: &mut EventCtx, data: &mut Self, env: &Env) {
        data.items.push_back(TodoItem::new(&data.new_item_text));
        data.new_item_text.clear();
        ctx.submit_command(SAVE);
    }
    fn delete_item(&mut self, id: &Uuid) {
        self.items.retain(|item| item.id != *id);
    }
}

struct MyAppDelegate {}
impl AppDelegate<AppState> for MyAppDelegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: Target,
        cmd: &Command,
        data: &mut AppState,
        env: &Env,
    ) -> Handled {
        if cmd.is(SAVE) {
            println!("save {:?}", data);
            Handled::Yes
        } else if let Some(id) = cmd.get(DELETE) {
            data.delete_item(id);
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

#[derive(Lens, Data, Clone, Debug, Serialize, Deserialize)]
struct TodoItem {
    #[data(same_fn = "PartialEq::eq")]
    id: Uuid,
    text: String,
    completed: bool,
}
impl TodoItem {
    fn new(text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            text: text.into(),
            completed: false,
        }
    }
    fn click_delete(ctx: &mut EventCtx, data: &mut Self, _env: &Env) {
        ctx.submit_command(DELETE.with(data.id));
        ctx.submit_command(SAVE);
    }
}

struct TodoItemController {}
impl<W: Widget<TodoItem>> Controller<TodoItem, W> for TodoItemController {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &TodoItem,
        data: &TodoItem,
        env: &Env,
    ) {
        if old_data.completed != data.completed {
            ctx.submit_command(SAVE);
        }
        child.update(ctx, old_data, data, env)
    }
}

fn todo_item() -> impl Widget<TodoItem> {
    let text = Label::raw().lens(TodoItem::text);
    let completed = Checkbox::new("completed").lens(TodoItem::completed);
    let delete_button = Button::new("X").on_click(TodoItem::click_delete);
    Flex::row()
        .with_child(text)
        .with_child(completed)
        .with_child(delete_button)
        .controller(TodoItemController {})
}

fn build_ui() -> impl Widget<AppState> {
    let new_item_textbox = TextBox::new().lens(AppState::new_item_text);
    let add_new_item = Button::new("add").on_click(AppState::click_add_new_item);
    let filtering = RadioGroup::column(vec![
        ("radio gaga", MyRadio::GaGa),
        ("radio gugu", MyRadio::GuGu),
        ("radio baabaa", MyRadio::BaaBaa),
    ])
    .lens(AppState::radio);
    let items = List::new(todo_item).lens(AppState::items);
    Flex::column()
        .with_child(new_item_textbox)
        .with_child(add_new_item)
        .with_child(filtering)
        .with_child(items)
}

fn main() -> Result<(), PlatformError> {
    let window = WindowDesc::new(build_ui())
        .title("my cool app")
        .window_size((1000., 500.));
    let inital_state = AppState::new();
    AppLauncher::with_window(window)
        .delegate(MyAppDelegate {})
        .launch(inital_state)?;
    Ok(())
}
