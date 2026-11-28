//! TextArea (`blueprint-textarea`): a field of several lines, such as a check's done-when or the
//! reason for overturning a decision. Its text is edited by gpui-component's multi-line input,
//! held in the window between frames under the field's id; its box is the design system's.

use std::rc::Rc;

use gpui::{
    App, AppContext, Focusable, Context, ElementId, Entity, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Subscription, Window, div, px,
};
use gpui_base::input::InputBaseState;
use gpui_component::input::{InputEvent, InputModeKind, Textarea, TextareaState};
use gpui_component::{Sizable, Size};

use super::{Class, apply, box_of, type_of};
use crate::theme::look::{self, Look, State};
use crate::theme::text;

/// Called with a field's text each time the person changes it.
pub(crate) type OnChange = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A field's input between frames, and the handler its changes go to: the one the field was
/// last drawn with.
pub(crate) struct Field<M: InputModeKind> {
    pub(crate) input: Entity<InputBaseState<M>>,
    on_change: Option<OnChange>,
    _change: Subscription,
}

impl<M: InputModeKind> Field<M> {
    /// The field held under `id`, made with `make` the first time it is drawn. Each frame it
    /// takes `on_change`, and `value` when the field's text is not already that.
    pub(crate) fn keyed(
        id: ElementId,
        value: Option<SharedString>,
        on_change: Option<OnChange>,
        window: &mut Window,
        cx: &mut App,
        make: impl FnOnce(&mut Window, &mut Context<InputBaseState<M>>) -> InputBaseState<M>,
    ) -> Entity<InputBaseState<M>> {
        let start = value.clone().unwrap_or_default();
        let field = window.use_keyed_state(id, cx, |window, cx: &mut Context<Self>| {
            let input = cx.new(|cx| make(window, cx).default_value(start));
            let change = cx.subscribe_in(
                &input,
                window,
                |this: &mut Self, input, event: &InputEvent, window, cx| {
                    if matches!(event, InputEvent::Change)
                        && let Some(handler) = this.on_change.clone()
                    {
                        let text = input.read(cx).value();
                        handler(&text, window, cx);
                    }
                },
            );
            Self {
                input,
                on_change: None,
                _change: change,
            }
        });
        field.update(cx, |f, _| f.on_change = on_change);
        let input = field.read(cx).input.clone();
        if let Some(value) = value
            && input.read(cx).value() != value
        {
            input.update(cx, |s, cx| s.set_value(value, window, cx));
        }
        input
    }
}

/// The room gpui-base's text editor keeps free at the right of a wrapped line (its private
/// `RIGHT_MARGIN`), which a browser's text area does not: the editor reaches this far past the
/// field's padding, so its lines wrap where the browser wraps them. A library value, not a
/// design one.
const WRAP_MARGIN: f32 = 10.;

/// How far gpui-component's smallest input already pads its text from the field's edges, at
/// the sides and at the top and bottom; the field's box pads the rest.
pub(crate) fn inner_pad() -> (gpui::Pixels, gpui::Pixels) {
    (Size::XSmall.input_px(), Size::XSmall.input_py())
}

/// A field's box in its look, less the padding the input draws inside it.
pub(crate) fn field_box<E: Styled>(el: E, look: &Look) -> E {
    let (side, top) = inner_pad();
    box_of(type_of(el, look), look)
        .pt(look.padding.top - top)
        .pb(look.padding.bottom - top)
        .pl(look.padding.left - side)
        .pr(look.padding.right - side)
}

#[derive(IntoElement)]
pub struct TextArea {
    id: ElementId,
    value: Option<SharedString>,
    default_value: SharedString,
    fill: bool,
    auto_resize: bool,
    disabled: bool,
    class_name: Option<Class>,
    on_change: Option<OnChange>,
}

impl TextArea {
    /// A field. `id` is its identity in the window, which keeps its text and caret between
    /// frames.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: SharedString::default(),
            fill: false,
            auto_resize: false,
            disabled: false,
            class_name: None,
            on_change: None,
        }
    }

    /// The text, kept by the caller from `on_change`: when it differs from the field's, the
    /// field shows it.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The text the field starts with, when the caller does not keep it.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Takes the full width of its container.
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Grows and shrinks with its text, one line at least; otherwise it is two lines tall.
    pub fn auto_resize(mut self, auto_resize: bool) -> Self {
        self.auto_resize = auto_resize;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A class of the design system's own: `Mono` sets the text in the code face, as
    /// RefineDialog's proven-by (`.tm-refine-row textarea.tm-mono`).
    pub fn class_name(mut self, class: Class) -> Self {
        self.class_name = Some(class);
        self
    }

    /// Called with the text each time the person changes it.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TextArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let auto = self.auto_resize;
        let value = self.value.or(Some(self.default_value).filter(|v| !v.is_empty()));
        let input: Entity<TextareaState> = Field::keyed(
            self.id.clone(),
            value,
            self.on_change,
            window,
            cx,
            move |window, cx| {
                let state = TextareaState::new(window, cx);
                // A text area is two lines tall, as HTML's `rows`; Blueprint's `autoResize`
                // sizes it to its text.
                if auto {
                    state.auto_grow(1, usize::MAX)
                } else {
                    state.rows(2)
                }
            },
        );
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let state = if self.disabled {
            State::Disabled
        } else if focused {
            State::Focus
        } else {
            State::Rest
        };
        // The export measured the filled text area at rest and in focus; a disabled one takes
        // the disabled input's look, which Blueprint draws with the same rules
        // (`.bp6-input:disabled`).
        let look = match state {
            State::Disabled => look::get(cx, "InputGroup", "default", State::Disabled),
            State::Focus => look::get(cx, "TextArea", "fill", State::Focus),
            _ => look::get(cx, "TextArea", "fill", State::Rest),
        };
        // `textarea.bp6-input { line-height: inherit }`: the body's line, at the field's size.
        let mut field = Textarea::new(&input)
            .appearance(false)
            .with_size(Size::XSmall)
            .disabled(self.disabled)
            .text_color(look.text)
            .text_size(look.font_size)
            .font_family(text::family(cx))
            .line_height(text::line_height(cx));
        if let Some(class) = self.class_name {
            field = apply(field, class, cx);
        }
        let selector = format!("{}", self.id);
        let mut el = field_box(div().id(self.id), &look)
            .debug_selector(|| selector)
            .min_w_0();
        if self.fill {
            el = el.w_full();
        }
        el.flex()
            .flex_col()
            .child(div().mr(-px(WRAP_MARGIN)).child(field))
    }
}
