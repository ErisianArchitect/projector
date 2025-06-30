use eframe::{
    egui::*
};
use crate::{ext::UiExt, settings::{Closer, OwnedCloser}};
use std::cell::RefCell;
use std::sync::{
    Arc,
    Mutex,
    MutexGuard,
};

pub fn centered_mbox_modal<R, F: FnOnce(&mut Ui) -> R>(ctx: &Context, add_contents: F) -> ModalResponse<R> {
    Modal::new(Id::new("centered_mbox_modal"))
        .area(
            Area::new(Id::new("centered_mbox_modal_area"))
                .constrain(true)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        )
        .frame(
            Frame::menu(&ctx.style())
        )
        .show(ctx, add_contents)
}

pub trait MessageBox<D> {
    fn show(&mut self, data: &mut D, closer: Closer, ui: &mut Ui);
}

impl<D, F> MessageBox<D> for F
where F: FnMut(&mut D, Closer, &mut Ui) {
    fn show(&mut self, data: &mut D, closer: Closer, ui: &mut Ui) {
        centered_mbox_modal(ui.ctx(), move |ui| {
            (*self)(data, closer, ui);
        });
    }
}

impl<D> MessageBox<D> for String {
    fn show(&mut self, _data: &mut D, closer: Closer, ui: &mut Ui) {
        centered_mbox_modal(ui.ctx(), move |ui| {
            ui.set_max_width(400.0);
            ui.vertical_centered_justified(|ui| {
                ui.with_inner_margin(Margin { top: 0, left: 0, right: 0, bottom: 4 }, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            Label::new(self.as_str())
                                .halign(Align::Min)
                                .selectable(false)
                                .wrap()
                        );
                    });
                });
                if ui.button("Close").clicked() {
                    closer.close();
                }
            });
        });
    }
}

impl<D> MessageBox<D> for &str {
    fn show(&mut self, _data: &mut D, closer: Closer, ui: &mut Ui) {
        centered_mbox_modal(ui.ctx(), move |ui| {
            ui.set_max_width(400.0);
            ui.vertical_centered_justified(|ui| {
                ui.with_inner_margin(Margin { top: 0, left: 0, right: 0, bottom: 4 }, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            Label::new(*self)
                                .halign(Align::Min)
                                .selectable(false)
                                .wrap()
                        );
                    });
                });
                if ui.button("Close").clicked() {
                    closer.close();
                }
            });
        });
    }
}

type MBoxMessage<D> = Option<Box<dyn MessageBox<D> + 'static>>;

pub struct MBox<D> {
    message: Arc<Mutex<MBoxMessage<D>>>,
}

impl<D> Clone for MBox<D> {
    fn clone(&self) -> Self {
        MBox {
            message: Arc::clone(&self.message),
        }
    }
}

impl<D> MBox<D> {
    pub fn new() -> Self {
        Self {
            message: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new_with<M: MessageBox<D> + 'static>(message: M) -> Self {
        Self {
            message: Arc::new(Mutex::new(Some(Box::new(message)))),
        }
    }

    fn lock<'a>(&'a self) -> MutexGuard<'a, MBoxMessage<D>> {
        self.message.lock().unwrap()
    }

    pub fn open<M: MessageBox<D> + 'static>(&self, message: M) {
        let mut message_lock = self.lock();
        *message_lock = Some(Box::new(message));
    }

    pub fn show(&self, data: &mut D, ui: &mut Ui) {
        let mut message_lock = self.lock();
        let msg_fn_opt = message_lock.take();
        drop(message_lock);
        if let Some(mut func) = msg_fn_opt {
            let close = OwnedCloser::new();
            func.show(data, close.make_closer(), ui);
            let mut message_lock = self.lock();
            if !close.is_closed() && message_lock.is_none() {
                *message_lock = Some(func);
            }
        }
    }
}