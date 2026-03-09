
use eframe::{
    egui::*
};
use crate::dgui::mbox::MessageBox;

use super::app::*;
use crossbeam_channel::{
    bounded, unbounded, Receiver, SendError, Sender
};

pub enum WorkResponse {
    Callback(Box<dyn FnOnce(&mut ProjectorApp, &Context) + Send + 'static>),
    FallibleCallback(Box<dyn FnOnce(&mut ProjectorApp, &Context) -> Result<(), crate::error::Error> + Send + 'static>),
}

impl WorkResponse {
    #[must_use]
    #[inline]
    pub fn callback<F: FnOnce(&mut ProjectorApp, &Context) + Send + 'static>(callback: F) -> WorkResponse {
        WorkResponse::Callback(Box::new(callback))
    }

    #[must_use]
    #[inline]
    pub fn fallible_callback<F: FnOnce(&mut ProjectorApp, &Context) -> Result<(), crate::error::Error> + Send + 'static>(callback: F) -> WorkResponse {
        WorkResponse::FallibleCallback(Box::new(callback))
    }

    #[must_use]
    #[inline]
    pub fn show_message<M: MessageBox<ProjectorApp> + Send + 'static>(message: M) -> WorkResponse {
        let msg: Box<dyn MessageBox<ProjectorApp> + Send + 'static> = Box::new(message);
        Self::callback(move |app, _| {
            app.show_message_boxed(msg);
        })
    }
}

#[must_use]
#[inline]
pub fn callback<F: FnOnce(&mut ProjectorApp, &Context) + Send + 'static>(callback: F) -> WorkResponse {
    WorkResponse::callback(callback)
}

#[must_use]
#[inline]
pub fn fallible_callback<F: FnOnce(&mut ProjectorApp, &Context) -> Result<(), crate::error::Error> + Send + 'static>(callback: F) -> WorkResponse {
    WorkResponse::fallible_callback(callback)
}

pub struct WorkPool {
    sender: Sender<WorkResponse>,
    receiver: Receiver<WorkResponse>,
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Responder {
    sender: Sender<WorkResponse>,
}

impl Responder {
    #[must_use]
    #[inline(always)]
    pub fn send(&self, response: WorkResponse) -> Result<(), SendError<WorkResponse>> {
        self.sender.send(response)
    }

    #[must_use]
    #[inline(always)]
    pub fn send_callback<F: FnOnce(&mut ProjectorApp, &Context) + Send + 'static>(&self, callback: F) -> Result<(), SendError<WorkResponse>> {
        self.send(WorkResponse::Callback(Box::new(callback)))
    }

    #[must_use]
    #[inline(always)]
    pub fn send_fallible_callback<F: FnOnce(&mut ProjectorApp, &Context) -> Result<(), crate::error::Error> + Send + 'static>(&self, callback: F) -> Result<(), SendError<WorkResponse>> {
        self.send(WorkResponse::FallibleCallback(Box::new(callback)))
    }

    #[must_use]
    #[inline(always)]
    pub fn show_message<M: MessageBox<ProjectorApp> + Send + 'static>(&self, message: M) -> Result<(), SendError<WorkResponse>> {
        self.send(WorkResponse::show_message(message))
    }
}

impl WorkPool {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            sender,
            receiver,
        }
    }

    #[inline]
    pub fn spawn<F: FnOnce(Responder) + Send + 'static>(&self, task: F) {
        let sender = self.sender.clone();
        let responder = Responder {
            sender,
        };
        rayon::spawn(move || {
            task(responder);
        });
    }

    #[must_use]
    #[inline]
    pub fn receiver(&self) -> &Receiver<WorkResponse> {
        &self.receiver
    }

}