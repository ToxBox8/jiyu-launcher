//! Zona que avisa del clic derecho con la posición del cursor (para abrir el menú contextual donde
//! has pulsado). iced 0.13 no trae menús contextuales; `mouse_area` da el clic pero no dónde.

use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{overlay, Clipboard, Layout, Shell, Widget};
use iced::event::{self, Event};
use iced::{mouse, Element, Length, Point, Rectangle, Size, Vector};

pub struct ClicDerecho<'a, Message, Theme, Renderer> {
    contenido: Element<'a, Message, Theme, Renderer>,
    al_pulsar: Box<dyn Fn(Point) -> Message + 'a>,
    boton: mouse::Button,
}

pub fn clic_derecho<'a, Message, Theme, Renderer>(
    contenido: impl Into<Element<'a, Message, Theme, Renderer>>,
    al_pulsar: impl Fn(Point) -> Message + 'a,
) -> ClicDerecho<'a, Message, Theme, Renderer> {
    ClicDerecho { contenido: contenido.into(), al_pulsar: Box::new(al_pulsar), boton: mouse::Button::Right }
}

/// Igual, pero con el clic normal (abrir el menú de un grupo al pulsarlo).
pub fn clic_izquierdo<'a, Message, Theme, Renderer>(
    contenido: impl Into<Element<'a, Message, Theme, Renderer>>,
    al_pulsar: impl Fn(Point) -> Message + 'a,
) -> ClicDerecho<'a, Message, Theme, Renderer> {
    ClicDerecho { contenido: contenido.into(), al_pulsar: Box::new(al_pulsar), boton: mouse::Button::Left }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ClicDerecho<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.contenido)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.contenido));
    }
    fn size(&self) -> Size<Length> {
        self.contenido.as_widget().size()
    }
    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.contenido.as_widget().layout(&mut tree.children[0], renderer, limits)
    }
    fn operate(&self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.contenido.as_widget().operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.contenido.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }
        if let Event::Mouse(mouse::Event::ButtonPressed(b)) = event {
            if b != self.boton {
                return event::Status::Ignored;
            }
            if let Some(p) = cursor.position_over(layout.bounds()) {
                shell.publish((self.al_pulsar)(p));
                return event::Status::Captured;
            }
        }
        event::Status::Ignored
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.contenido.as_widget().mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.contenido.as_widget().draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.contenido.as_widget_mut().overlay(&mut tree.children[0], layout, renderer, translation)
    }
}

impl<'a, Message: 'a, Theme: 'a, Renderer: renderer::Renderer + 'a> From<ClicDerecho<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
{
    fn from(w: ClicDerecho<'a, Message, Theme, Renderer>) -> Self {
        Element::new(w)
    }
}
